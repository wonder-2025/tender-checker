use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use chrono::Utc;

/// LLM提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub name: String,
    pub display_name: String,
    pub models: Vec<String>,
    pub default_model: String,
    pub requires_secret_key: bool,
}

/// 多LLM管理器
pub struct MultiLlmManager {
    providers: HashMap<String, LlmProvider>,
    api_keys: Arc<Mutex<HashMap<String, String>>>,
    health_status: Arc<Mutex<HashMap<String, bool>>>,
    current_provider: Arc<Mutex<String>>,
}

impl MultiLlmManager {
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        
        // 百度千帆
        providers.insert("baidu".to_string(), LlmProvider {
            name: "baidu".to_string(),
            display_name: "百度千帆".to_string(),
            models: vec![
                "ERNIE-4.0-8K".to_string(),
                "ERNIE-4.0".to_string(),
                "ERNIE-3.5-8K".to_string(),
                "ERNIE-3.5".to_string(),
                "ERNIE-Speed-8K".to_string(),
                "ERNIE-Speed".to_string(),
            ],
            default_model: "ERNIE-3.5-8K".to_string(),
            requires_secret_key: true,
        });
        
        // 阿里通义
        providers.insert("aliyun".to_string(), LlmProvider {
            name: "aliyun".to_string(),
            display_name: "阿里通义".to_string(),
            models: vec![
                "qwen-max".to_string(),
                "qwen-plus".to_string(),
                "qwen-turbo".to_string(),
            ],
            default_model: "qwen-plus".to_string(),
            requires_secret_key: false,
        });
        
        // OpenAI
        providers.insert("openai".to_string(), LlmProvider {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            models: vec![
                "gpt-4-turbo".to_string(),
                "gpt-4".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            default_model: "gpt-3.5-turbo".to_string(),
            requires_secret_key: false,
        });
        
        // DeepSeek
        providers.insert("deepseek".to_string(), LlmProvider {
            name: "deepseek".to_string(),
            display_name: "DeepSeek".to_string(),
            models: vec![
                "deepseek-chat".to_string(),
                "deepseek-coder".to_string(),
            ],
            default_model: "deepseek-chat".to_string(),
            requires_secret_key: false,
        });
        
        Self {
            providers,
            api_keys: Arc::new(Mutex::new(HashMap::new())),
            health_status: Arc::new(Mutex::new(HashMap::new())),
            current_provider: Arc::new(Mutex::new("baidu".to_string())),
        }
    }
    
    /// 获取所有提供商
    pub fn get_providers(&self) -> Vec<&LlmProvider> {
        self.providers.values().collect()
    }
    
    /// 获取指定提供商
    pub fn get_provider(&self, name: &str) -> Option<&LlmProvider> {
        self.providers.get(name)
    }
    
    /// 设置API密钥
    pub fn set_api_key(&self, provider: &str, api_key: &str, secret_key: Option<&str>) {
        let key = if let Some(sk) = secret_key {
            format!("{}:{}", api_key, sk)
        } else {
            api_key.to_string()
        };
        
        self.api_keys.lock().insert(provider.to_string(), key);
    }
    
    /// 获取API密钥
    pub fn get_api_key(&self, provider: &str) -> Option<(String, Option<String>)> {
        let keys = self.api_keys.lock();
        keys.get(provider).map(|k| {
            if k.contains(':') {
                let parts: Vec<&str> = k.split(':').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (k.clone(), None)
            }
        })
    }
    
    /// 测试连接
    pub async fn test_connection(&self, provider: &str, model: &str) -> Result<bool, String> {
        let config = self.build_config(provider, model)?;
        
        match super::llm_client::chat(&config, "你好".to_string()).await {
            Ok(_) => {
                self.health_status.lock().insert(provider.to_string(), true);
                Ok(true)
            }
            Err(e) => {
                self.health_status.lock().insert(provider.to_string(), false);
                Err(e)
            }
        }
    }
    
    /// 构建配置
    pub fn build_config(&self, provider: &str, model: &str) -> Result<super::llm_client::LlmConfig, String> {
        let (api_key, secret_key) = self.get_api_key(provider)
            .ok_or_else(|| format!("未配置{}的API密钥", provider))?;
        
        let base_url = match provider {
            "baidu" => "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat".to_string(),
            "aliyun" => "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            _ => return Err(format!("未知的提供商: {}", provider)),
        };
        
        Ok(super::llm_client::LlmConfig {
            provider: provider.to_string(),
            api_key,
            secret_key,
            model: model.to_string(),
            base_url: Some(base_url),
        })
    }
    
    /// 智能选择模型（根据成本和质量）
    pub fn select_optimal_model(&self, task_type: &str) -> (String, String) {
        match task_type {
            // 简单任务使用快速模型
            "quick_check" => {
                if self.is_provider_available("baidu") {
                    ("baidu".to_string(), "ERNIE-Speed-8K".to_string())
                } else if self.is_provider_available("openai") {
                    ("openai".to_string(), "gpt-3.5-turbo".to_string())
                } else {
                    ("baidu".to_string(), "ERNIE-3.5-8K".to_string())
                }
            }
            // 复杂任务使用强力模型
            "complex_check" => {
                if self.is_provider_available("baidu") {
                    ("baidu".to_string(), "ERNIE-4.0-8K".to_string())
                } else if self.is_provider_available("openai") {
                    ("openai".to_string(), "gpt-4-turbo".to_string())
                } else {
                    ("baidu".to_string(), "ERNIE-4.0".to_string())
                }
            }
            _ => ("baidu".to_string(), "ERNIE-3.5-8K".to_string()),
        }
    }
    
    /// 检查提供商是否可用
    fn is_provider_available(&self, provider: &str) -> bool {
        self.health_status.lock().get(provider).copied().unwrap_or(false)
    }
    
    /// 自动切换备用API
    pub async fn auto_failover(&self, failed_provider: &str) -> Option<(String, String)> {
        let fallback_order = match failed_provider {
            "baidu" => vec!["aliyun", "openai", "deepseek"],
            "aliyun" => vec!["baidu", "openai", "deepseek"],
            "openai" => vec!["baidu", "aliyun", "deepseek"],
            _ => vec!["baidu", "aliyun", "openai"],
        };
        
        for provider in fallback_order {
            if self.is_provider_available(provider) {
                if let Some(p) = self.get_provider(provider) {
                    return Some((provider.to_string(), p.default_model.clone()));
                }
            }
        }
        
        None
    }
}

/// 检查结果缓存
pub struct LlmCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    ttl_seconds: u64,
}

#[derive(Clone)]
struct CacheEntry {
    response: String,
    timestamp: i64,
}

impl LlmCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl_seconds,
        }
    }
    
    /// 生成缓存键
    fn make_key(provider: &str, model: &str, prompt: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(provider.as_bytes());
        hasher.update(model.as_bytes());
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// 获取缓存
    pub fn get(&self, provider: &str, model: &str, prompt: &str) -> Option<String> {
        let key = Self::make_key(provider, model, prompt);
        let cache = self.cache.lock();
        
        cache.get(&key).and_then(|entry| {
            let now = Utc::now().timestamp();
            if (now - entry.timestamp) as u64 <= self.ttl_seconds {
                Some(entry.response.clone())
            } else {
                None
            }
        })
    }
    
    /// 设置缓存
    pub fn set(&self, provider: &str, model: &str, prompt: &str, response: &str) {
        let key = Self::make_key(provider, model, prompt);
        let mut cache = self.cache.lock();
        
        cache.insert(key, CacheEntry {
            response: response.to_string(),
            timestamp: Utc::now().timestamp(),
        });
    }
    
    /// 清理过期缓存
    pub fn cleanup(&self) {
        let mut cache = self.cache.lock();
        let now = Utc::now().timestamp();
        
        cache.retain(|_, entry| {
            (now - entry.timestamp) as u64 <= self.ttl_seconds
        });
    }
}

/// Tauri命令：获取提供商列表
#[tauri::command]
pub fn get_llm_providers() -> Vec<LlmProvider> {
    let manager = MultiLlmManager::new();
    manager.get_providers().into_iter().cloned().collect()
}

/// Tauri命令：测试连接
#[tauri::command]
pub async fn test_llm_connection(
    provider: String,
    model: String,
    api_key: String,
    secret_key: Option<String>,
) -> Result<bool, String> {
    let manager = MultiLlmManager::new();
    manager.set_api_key(&provider, &api_key, secret_key.as_deref());
    manager.test_connection(&provider, &model).await
}

/// Tauri命令：智能选择模型
#[tauri::command]
pub fn select_optimal_llm(task_type: String) -> (String, String) {
    let manager = MultiLlmManager::new();
    manager.select_optimal_model(&task_type)
}
