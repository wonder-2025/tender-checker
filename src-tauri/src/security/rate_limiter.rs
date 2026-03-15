use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 频率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 每小时最大检查次数
    pub max_per_hour: u32,
    /// 每日最大检查次数
    pub max_per_day: u32,
    /// 两次检查最小间隔（秒）
    pub min_interval_secs: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_per_hour: 20,
            max_per_day: 50,
            min_interval_secs: 10,
        }
    }
}

/// 使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: i64,
    pub action: String,
    pub resource: String,
    pub success: bool,
}

/// 频率限制器
pub struct RateLimiter {
    config: RateLimitConfig,
    check_times: Arc<Mutex<Vec<Instant>>>,
    last_check: Arc<Mutex<Option<Instant>>>,
    usage_records: Arc<Mutex<Vec<UsageRecord>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            check_times: Arc::new(Mutex::new(Vec::new())),
            last_check: Arc::new(Mutex::new(None)),
            usage_records: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// 检查是否可以执行操作
    pub fn check_rate_limit(&self, action: &str) -> Result<(), String> {
        let now = Instant::now();
        
        // 检查最小间隔
        {
            let last = self.last_check.lock().expect("获取锁失败: last_check");
            if let Some(last_time) = *last {
                let elapsed = now.duration_since(last_time).as_secs();
                if elapsed < self.config.min_interval_secs as u64 {
                    return Err(format!(
                        "操作过于频繁，请等待 {} 秒",
                        self.config.min_interval_secs - elapsed as u32
                    ));
                }
            }
        }
        
        // 清理过期记录
        {
            let mut times = self.check_times.lock().expect("获取锁失败: check_times");
            times.retain(|&t| now.duration_since(t).as_secs() < 86400);
        }
        
        // 检查日限制
        {
            let times = self.check_times.lock().expect("获取锁失败: check_times");
            if times.len() >= self.config.max_per_day as usize {
                return Err(format!(
                    "已达到每日操作上限（{}次），请明天再试",
                    self.config.max_per_day
                ));
            }
        }
        
        // 检查小时限制
        {
            let times = self.check_times.lock().expect("获取锁失败: check_times");
            let hour_ago = now - std::time::Duration::from_secs(3600);
            let hour_count = times.iter().filter(|&&t| t > hour_ago).count();
            
            if hour_count >= self.config.max_per_hour as usize {
                return Err(format!(
                    "每小时操作次数已达上限（{}次），请稍后再试",
                    self.config.max_per_hour
                ));
            }
        }
        
        Ok(())
    }
    
    /// 记录操作
    pub fn record_action(&self, action: &str, resource: &str, success: bool) {
        let now = Instant::now();
        
        // 记录时间
        {
            let mut times = self.check_times.lock().expect("获取锁失败: check_times");
            times.push(now);
        }
        
        // 更新最后操作时间
        {
            let mut last = self.last_check.lock().expect("获取锁失败: last_check");
            *last = Some(now);
        }
        
        // 记录详细日志
        {
            let mut records = self.usage_records.lock().expect("获取锁失败: usage_records");
            records.push(UsageRecord {
                timestamp: chrono::Utc::now().timestamp(),
                action: action.to_string(),
                resource: resource.to_string(),
                success,
            });
            
            // 只保留最近1000条
            if records.len() > 1000 {
                let start = records.len() - 1000;
                let new_records: Vec<_> = records.drain(start..).collect();
                *records = new_records;
            }
        }
    }
    
    /// 获取今日使用统计
    pub fn get_today_stats(&self) -> UsageStats {
        let times = self.check_times.lock().expect("获取锁失败: check_times");
        let now = Instant::now();
        let day_ago = now - std::time::Duration::from_secs(86400);
        let hour_ago = now - std::time::Duration::from_secs(3600);
        
        let today_count = times.iter().filter(|&&t| t > day_ago).count();
        let hour_count = times.iter().filter(|&&t| t > hour_ago).count();
        
        UsageStats {
            today_count: today_count as u32,
            hour_count: hour_count as u32,
            max_per_day: self.config.max_per_day,
            max_per_hour: self.config.max_per_hour,
            remaining_today: self.config.max_per_day.saturating_sub(today_count as u32),
            remaining_hour: self.config.max_per_hour.saturating_sub(hour_count as u32),
        }
    }
    
    /// 获取使用历史
    pub fn get_usage_history(&self, limit: usize) -> Vec<UsageRecord> {
        let records = self.usage_records.lock().expect("获取锁失败: usage_records");
        records.iter().rev().take(limit).cloned().collect()
    }
    
    /// 重置计数（用于测试或管理员操作）
    pub fn reset(&self) {
        let mut times = self.check_times.lock().expect("获取锁失败: check_times");
        times.clear();
        
        let mut last = self.last_check.lock().expect("获取锁失败: last_check");
        *last = None;
    }
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub today_count: u32,
    pub hour_count: u32,
    pub max_per_day: u32,
    pub max_per_hour: u32,
    pub remaining_today: u32,
    pub remaining_hour: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            max_per_hour: 3,
            max_per_day: 5,
            min_interval_secs: 0,
        };
        
        let limiter = RateLimiter::new(config);
        
        // 前3次应该成功
        for i in 0..3 {
            assert!(limiter.check_rate_limit("check").is_ok());
            limiter.record_action("check", &format!("file_{}.docx", i), true);
        }
        
        // 第4次应该失败（小时限制）
        assert!(limiter.check_rate_limit("check").is_err());
        
        let stats = limiter.get_today_stats();
        assert_eq!(stats.today_count, 3);
    }
}
