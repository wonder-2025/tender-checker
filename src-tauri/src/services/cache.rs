use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use rusqlite::{Connection, params};

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub hit_count: u32,
    pub size_bytes: usize,
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

/// 缓存管理器
pub struct CacheManager {
    memory_cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    db_path: PathBuf,
    max_memory_size: usize,  // 最大内存缓存大小 (bytes)
    default_ttl: Duration,    // 默认过期时间
    stats: Arc<Mutex<CacheStats>>,
}

impl CacheManager {
    pub fn new(db_path: PathBuf, max_memory_mb: usize, default_ttl_hours: i64) -> Self {
        let max_memory_size = max_memory_mb * 1024 * 1024;
        let default_ttl = Duration::hours(default_ttl_hours);
        
        let manager = Self {
            memory_cache: Arc::new(Mutex::new(HashMap::new())),
            db_path,
            max_memory_size,
            default_ttl,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        };
        
        // 初始化数据库
        manager.init_database().ok();
        
        // 从数据库加载热点数据到内存
        manager.load_hot_entries().ok();
        
        manager
    }
    
    /// 初始化数据库
    fn init_database(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                hit_count INTEGER DEFAULT 0,
                size_bytes INTEGER DEFAULT 0
            )",
            [],
        ).map_err(|e| format!("创建表失败: {}", e))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_expires_at ON cache(expires_at)",
            [],
        ).map_err(|e| format!("创建索引失败: {}", e))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hit_count ON cache(hit_count DESC)",
            [],
        ).map_err(|e| format!("创建索引失败: {}", e))?;
        
        Ok(())
    }
    
    /// 加载热点数据到内存
    fn load_hot_entries(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        let now = Utc::now().to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT key, value, created_at, expires_at, hit_count, size_bytes 
             FROM cache 
             WHERE expires_at > ? 
             ORDER BY hit_count DESC 
             LIMIT 100"
        ).map_err(|e| format!("查询失败: {}", e))?;
        
        let entries: Vec<CacheEntry> = stmt.query_map([&now], |row| {
            Ok(CacheEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                created_at: row.get::<_, String>(2)?.parse().unwrap_or(Utc::now()),
                expires_at: row.get::<_, String>(3)?.parse().unwrap_or(Utc::now()),
                hit_count: row.get(4)?,
                size_bytes: row.get(5)?,
            })
        }).map_err(|e| format!("读取失败: {}", e))?
        .filter_map(|e| e.ok())
        .collect();
        
        let mut cache = self.memory_cache.lock();
        for entry in entries {
            cache.insert(entry.key.clone(), entry);
        }
        
        Ok(())
    }
    
    /// 生成缓存键
    pub fn generate_key(prefix: &str, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prefix.as_bytes());
        hasher.update(data.as_bytes());
        format!("{}:{:x}", prefix, hasher.finalize())
    }
    
    /// 获取缓存
    pub fn get(&self, key: &str) -> Option<String> {
        // 先查内存缓存
        {
            let mut cache = self.memory_cache.lock();
            if let Some(entry) = cache.get_mut(key) {
                if entry.expires_at > Utc::now() {
                    entry.hit_count += 1;
                    self.stats.lock().hits += 1;
                    return Some(entry.value.clone());
                } else {
                    // 过期，删除
                    cache.remove(key);
                }
            }
        }
        
        // 查数据库
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = Utc::now().to_rfc3339();
            let mut stmt = conn.prepare(
                "SELECT value, created_at, expires_at, hit_count, size_bytes 
                 FROM cache 
                 WHERE key = ? AND expires_at > ?"
            ).ok()?;
            
            let result = stmt.query_row(params![key, now], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, usize>(4)?,
                ))
            }).ok()?;
            
            // 更新命中次数
            conn.execute(
                "UPDATE cache SET hit_count = hit_count + 1 WHERE key = ?",
                [key]
            ).ok()?;
            
            // 添加到内存缓存
            let entry = CacheEntry {
                key: key.to_string(),
                value: result.0.clone(),
                created_at: result.1.parse().unwrap_or(Utc::now()),
                expires_at: result.2.parse().unwrap_or(Utc::now()),
                hit_count: result.3 + 1,
                size_bytes: result.4,
            };
            
            self.memory_cache.lock().insert(key.to_string(), entry);
            self.stats.lock().hits += 1;
            
            return Some(result.0);
        }
        
        self.stats.lock().misses += 1;
        None
    }
    
    /// 设置缓存
    pub fn set(&self, key: &str, value: &str, ttl_hours: Option<i64>) -> Result<(), String> {
        let ttl = Duration::hours(ttl_hours.unwrap_or(self.default_ttl.num_hours() as i64));
        let now = Utc::now();
        let expires_at = now + ttl;
        let size_bytes = value.len();
        
        let entry = CacheEntry {
            key: key.to_string(),
            value: value.to_string(),
            created_at: now,
            expires_at,
            hit_count: 0,
            size_bytes,
        };
        
        // 存入内存
        {
            let mut cache = self.memory_cache.lock();
            
            // 检查内存限制
            let current_size: usize = cache.values().map(|e| e.size_bytes).sum();
            if current_size + size_bytes > self.max_memory_size {
                // 清理最少使用的
                let entries: Vec<_> = cache.iter()
                    .map(|(k, e)| (k.clone(), e.hit_count, e.size_bytes))
                    .collect();
                let mut entries = entries;
                entries.sort_by_key(|(_, hit, _)| *hit);
                
                let mut freed = 0;
                for (k, _, size) in entries {
                    if current_size - freed + size_bytes <= self.max_memory_size {
                        break;
                    }
                    freed += size;
                    cache.remove(&k);
                    self.stats.lock().evictions += 1;
                }
            }
            
            cache.insert(key.to_string(), entry.clone());
        }
        
        // 存入数据库
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, created_at, expires_at, hit_count, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                key,
                value,
                now.to_rfc3339(),
                expires_at.to_rfc3339(),
                entry.hit_count,
                size_bytes
            ],
        ).map_err(|e| format!("插入失败: {}", e))?;
        
        Ok(())
    }
    
    /// 删除缓存
    pub fn delete(&self, key: &str) -> Result<(), String> {
        self.memory_cache.lock().remove(key);
        
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        conn.execute("DELETE FROM cache WHERE key = ?", [key])
            .map_err(|e| format!("删除失败: {}", e))?;
        
        Ok(())
    }
    
    /// 清理过期缓存
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("打开数据库失败: {}", e))?;
        
        let now = Utc::now().to_rfc3339();
        let count = conn.execute("DELETE FROM cache WHERE expires_at <= ?", [&now])
            .map_err(|e| format!("清理失败: {}", e))?;
        
        // 同时清理内存缓存
        self.memory_cache.lock().retain(|_, e| e.expires_at > Utc::now());
        
        Ok(count)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().clone();
        
        // 从数据库获取更准确的统计
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = Utc::now().to_rfc3339();
            
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM cache WHERE expires_at > ?",
                [&now],
                |row| row.get::<_, usize>(0)
            ) {
                stats.total_entries = count;
            }
            
            if let Ok(size) = conn.query_row(
                "SELECT COALESCE(SUM(size_bytes), 0) FROM cache WHERE expires_at > ?",
                [&now],
                |row| row.get::<_, usize>(0)
            ) {
                stats.total_size_bytes = size;
            }
        }
        
        stats
    }
    
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.lock();
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }
}

/// 检查结果缓存
pub struct CheckResultCache {
    manager: CacheManager,
}

impl CheckResultCache {
    pub fn new(data_dir: PathBuf) -> Self {
        let db_path = data_dir.join("cache.db");
        Self {
            manager: CacheManager::new(db_path, 100, 24), // 100MB, 24小时
        }
    }
    
    /// 缓存检查结果
    pub fn cache_result(&self, file_hash: &str, check_type: &str, result: &str) -> Result<(), String> {
        let key = CacheManager::generate_key("check", &format!("{}:{}", file_hash, check_type));
        self.manager.set(&key, result, None)
    }
    
    /// 获取缓存的检查结果
    pub fn get_cached_result(&self, file_hash: &str, check_type: &str) -> Option<String> {
        let key = CacheManager::generate_key("check", &format!("{}:{}", file_hash, check_type));
        self.manager.get(&key)
    }
    
    /// 缓存LLM响应
    pub fn cache_llm_response(&self, provider: &str, model: &str, prompt_hash: &str, response: &str) -> Result<(), String> {
        let key = CacheManager::generate_key("llm", &format!("{}:{}:{}", provider, model, prompt_hash));
        self.manager.set(&key, response, Some(72)) // 72小时
    }
    
    /// 获取缓存的LLM响应
    pub fn get_llm_response(&self, provider: &str, model: &str, prompt_hash: &str) -> Option<String> {
        let key = CacheManager::generate_key("llm", &format!("{}:{}:{}", provider, model, prompt_hash));
        self.manager.get(&key)
    }
    
    /// 获取统计
    pub fn get_stats(&self) -> CacheStats {
        self.manager.get_stats()
    }
    
    /// 清理过期缓存
    pub fn cleanup(&self) -> Result<usize, String> {
        self.manager.cleanup_expired()
    }
}

/// Tauri命令：获取缓存统计
#[tauri::command]
pub fn get_cache_stats(data_dir: String) -> Result<CacheStats, String> {
    let cache = CheckResultCache::new(PathBuf::from(data_dir));
    Ok(cache.get_stats())
}

/// Tauri命令：清理缓存
#[tauri::command]
pub fn cleanup_cache(data_dir: String) -> Result<usize, String> {
    let cache = CheckResultCache::new(PathBuf::from(data_dir));
    cache.cleanup()
}

/// Tauri命令：清除所有缓存
#[tauri::command]
pub fn clear_all_cache(data_dir: String) -> Result<(), String> {
    let db_path = PathBuf::from(data_dir).join("cache.db");
    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .map_err(|e| format!("删除缓存失败: {}", e))?;
    }
    Ok(())
}
