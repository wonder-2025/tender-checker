use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use crate::security::log_sanitizer::{LogSanitizer, sanitize_file_path};

/// 审计日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// 时间戳
    pub timestamp: i64,
    /// 事件ID
    pub event_id: String,
    /// 用户标识
    pub user_id: String,
    /// 用户名
    pub user_name: String,
    /// 操作类型
    pub action: String,
    /// 操作资源（已脱敏）
    pub resource: String,
    /// 操作结果
    pub result: String,
    /// 错误信息（已脱敏）
    pub error_message: Option<String>,
    /// 设备指纹
    pub device_fp: String,
    /// 许可证ID
    pub license_id: String,
    /// 客户端IP（如果有）
    pub client_ip: Option<String>,
    /// 附加信息（已脱敏）
    pub extra: Option<serde_json::Value>,
}

/// 审计日志管理器
pub struct AuditLogger {
    log_path: PathBuf,
    current_user: String,
    device_fp: String,
    license_id: String,
}

impl AuditLogger {
    pub fn new(app_data_dir: &std::path::Path, device_fp: String) -> Self {
        Self {
            log_path: app_data_dir.join("audit.log"),
            current_user: "unknown".to_string(),
            device_fp,
            license_id: "unknown".to_string(),
        }
    }
    
    /// 设置当前用户信息
    pub fn set_user(&mut self, user_name: String, license_id: String) {
        self.current_user = user_name;
        self.license_id = license_id;
    }
    
    /// 记录审计日志（自动脱敏）
    pub fn log(
        &self,
        action: &str,
        resource: &str,
        result: &str,
        error_message: Option<String>,
        extra: Option<serde_json::Value>,
    ) {
        // 脱敏处理
        let sanitized_resource = sanitize_file_path(resource);
        let sanitized_error = error_message.as_ref().map(|e| LogSanitizer::sanitize(e));
        
        let log_entry = AuditLog {
            timestamp: Utc::now().timestamp(),
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id: self.device_fp.clone(), // 使用设备指纹作为用户ID
            user_name: self.current_user.clone(),
            action: action.to_string(),
            resource: sanitized_resource,
            result: result.to_string(),
            error_message: sanitized_error,
            device_fp: self.device_fp.clone(),
            license_id: self.license_id.clone(),
            client_ip: None,
            extra,
        };
        
        // 写入日志文件
        self.write_log(&log_entry);
        
        // 同时打印到控制台（开发阶段，已脱敏）
        log::info!(
            "[AUDIT] {} | {} | {} | {} | {}",
            log_entry.timestamp,
            log_entry.action,
            log_entry.resource,
            log_entry.result,
            log_entry.user_name
        );
    }
    
    /// 记录成功操作
    pub fn log_success(&self, action: &str, resource: &str) {
        self.log(action, resource, "success", None, None);
    }
    
    /// 记录失败操作
    pub fn log_failure(&self, action: &str, resource: &str, error: &str) {
        self.log(action, resource, "failed", Some(error.to_string()), None);
    }
    
    /// 记录带详细信息的操作
    pub fn log_with_extra(&self, action: &str, resource: &str, extra: serde_json::Value) {
        self.log(action, resource, "success", None, Some(extra));
    }
    
    /// 写入日志文件
    fn write_log(&self, log: &AuditLog) {
        let log_line = serde_json::to_string(log).unwrap_or_default();
        
        // 追加写入
        use std::fs::OpenOptions;
        use std::io::Write;
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(file, "{}", log_line);
        }
    }
    
    /// 读取审计日志
    pub fn read_logs(&self, limit: usize) -> Vec<AuditLog> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        
        let mut logs = Vec::new();
        
        if let Ok(file) = File::open(&self.log_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().rev().take(limit).flatten() {
                if let Ok(log) = serde_json::from_str::<AuditLog>(&line) {
                    logs.push(log);
                }
            }
        }
        
        logs
    }
    
    /// 获取统计信息
    pub fn get_stats(&self, days: u32) -> AuditStats {
        let logs = self.read_logs(10000);
        let cutoff = Utc::now().timestamp() - (days as i64 * 86400);
        
        let recent: Vec<_> = logs.iter()
            .filter(|l| l.timestamp > cutoff)
            .collect();
        
        let total_count = recent.len() as u32;
        let success_count = recent.iter().filter(|l| l.result == "success").count() as u32;
        let failed_count = recent.iter().filter(|l| l.result == "failed").count() as u32;
        
        let mut action_counts = std::collections::HashMap::new();
        for log in &recent {
            *action_counts.entry(log.action.clone()).or_insert(0) += 1;
        }
        
        AuditStats {
            total_count,
            success_count,
            failed_count,
            action_counts,
        }
    }
    
    /// 清理旧日志
    pub fn cleanup_old_logs(&self, keep_days: u32) {
        let cutoff = Utc::now().timestamp() - (keep_days as i64 * 86400);
        
        let logs = self.read_logs(100000);
        
        use std::fs::OpenOptions;
        use std::io::Write;
        
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.log_path)
        {
            for log in logs.iter().filter(|l| l.timestamp > cutoff) {
                if let Ok(line) = serde_json::to_string(log) {
                    let _ = writeln!(file, "{}", line);
                }
            }
        }
    }
}

/// 审计统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_count: u32,
    pub success_count: u32,
    pub failed_count: u32,
    pub action_counts: std::collections::HashMap<String, u32>,
}

/// 预定义操作类型
pub mod actions {
    pub const APP_START: &str = "app_start";
    pub const APP_CLOSE: &str = "app_close";
    pub const LICENSE_IMPORT: &str = "license_import";
    pub const FILE_UPLOAD: &str = "file_upload";
    pub const TENDER_PARSE: &str = "tender_parse";
    pub const CHECK_START: &str = "check_start";
    pub const CHECK_COMPLETE: &str = "check_complete";
    pub const REPORT_EXPORT: &str = "report_export";
    pub const CONFIG_CHANGE: &str = "config_change";
    pub const CUSTOM_RULE_CREATE: &str = "custom_rule_create";
    pub const CUSTOM_RULE_DELETE: &str = "custom_rule_delete";
    pub const API_TEST: &str = "api_test";
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_audit_logger() {
        let dir = tempdir().unwrap();
        let logger = AuditLogger::new(dir.path(), "test-device-fp".to_string());
        
        logger.log_success("check", "test.docx");
        logger.log_failure("export", "report.pdf", "权限不足");
        
        let logs = logger.read_logs(10);
        assert_eq!(logs.len(), 2);
    }
}
