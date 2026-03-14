// 日志记录服务
// 设计者: wonder-宏 & JARVIS AI Assistant

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// 日志级别
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

/// 日志类型
#[derive(Debug, Clone)]
pub enum LogCategory {
    Desensitize,
    ApiRequest,
    ApiResponse,
    FileOperation,
    CheckProcess,
    FormatChange,
}

impl LogCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Desensitize => "DESENSITIZE",
            LogCategory::ApiRequest => "API_REQUEST",
            LogCategory::ApiResponse => "API_RESPONSE",
            LogCategory::FileOperation => "FILE_OPERATION",
            LogCategory::CheckProcess => "CHECK_PROCESS",
            LogCategory::FormatChange => "FORMAT_CHANGE",
        }
    }
}

/// 全局日志配置
pub struct LoggerConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub enabled_categories: Vec<String>,
    pub log_dir: PathBuf,
}

static LOGGER_CONFIG: Lazy<Mutex<Option<LoggerConfig>>> = Lazy::new(|| Mutex::new(None));

/// 初始化日志器
pub fn init_logger(data_dir: &str, enabled: bool, level: &str, log_items: &[String]) {
    let config = LoggerConfig {
        enabled,
        level: LogLevel::from(level),
        enabled_categories: log_items.to_vec(),
        log_dir: PathBuf::from(data_dir).join("logs"),
    };
    
    // 确保日志目录存在
    if !config.log_dir.exists() {
        let _ = fs::create_dir_all(&config.log_dir);
    }
    
    *LOGGER_CONFIG.lock().unwrap() = Some(config);
}

/// 检查是否应该记录该类型的日志
fn should_log(category: &LogCategory) -> bool {
    let config = LOGGER_CONFIG.lock().unwrap();
    
    if let Some(ref cfg) = *config {
        if !cfg.enabled {
            return false;
        }
        
        let category_str = match category {
            LogCategory::Desensitize => "desensitize",
            LogCategory::ApiRequest => "apiRequest",
            LogCategory::ApiResponse => "apiResponse",
            LogCategory::FileOperation => "fileOperation",
            LogCategory::CheckProcess => "checkProcess",
            LogCategory::FormatChange => "formatChange",
        };
        
        return cfg.enabled_categories.contains(&category_str.to_string());
    }
    
    false
}

/// 获取日志文件路径
fn get_log_file_path() -> Option<PathBuf> {
    let config = LOGGER_CONFIG.lock().unwrap();
    
    if let Some(ref cfg) = *config {
        let today = Local::now().format("%Y-%m-%d").to_string();
        return Some(cfg.log_dir.join(format!("debug-{}.log", today)));
    }
    
    None
}

/// 写入日志
fn write_log(category: LogCategory, message: &str) {
    if !should_log(&category) {
        return;
    }
    
    let log_path = match get_log_file_path() {
        Some(p) => p,
        None => return,
    };
    
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let log_line = format!("[{}] [DEBUG] [{}] {}\n", timestamp, category.as_str(), message);
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

/// 记录脱敏过程
pub fn log_desensitize(original: &str, desensitized: &str, rules: &[String]) {
    let message = format!(
        r#"Original: {}
  Desensitized: {}
  Applied rules: {:?}"#,
        original, desensitized, rules
    );
    write_log(LogCategory::Desensitize, &message);
}

/// 记录API请求
pub fn log_api_request(provider: &str, endpoint: &str, headers: &str, body: &str) {
    // 脱敏处理：隐藏API Key
    let safe_headers = mask_api_key(headers);
    let safe_body = mask_api_key(body);
    
    let message = format!(
        r#"Provider: {}
  Endpoint: {}
  Headers: {}
  Body: {}"#,
        provider, endpoint, safe_headers, safe_body
    );
    write_log(LogCategory::ApiRequest, &message);
}

/// 记录API响应
pub fn log_api_response(provider: &str, status: u16, body: &str) {
    // 限制响应体长度
    let truncated_body = if body.len() > 1000 {
        format!("{}... (truncated, total {} bytes)", &body[..1000], body.len())
    } else {
        body.to_string()
    };
    
    let message = format!(
        r#"Provider: {}
  Status: {}
  Body: {}"#,
        provider, status, truncated_body
    );
    write_log(LogCategory::ApiResponse, &message);
}

/// 记录文件操作
pub fn log_file_operation(operation: &str, path: &str, success: bool, message: Option<&str>) {
    let status = if success { "成功" } else { "失败" };
    let log_message = if let Some(msg) = message {
        format!(
            r#"Operation: {}
  Path: {}
  Status: {}
  Message: {}"#,
            operation, path, status, msg
        )
    } else {
        format!(
            r#"Operation: {}
  Path: {}
  Status: {}"#,
            operation, path, status
        )
    };
    write_log(LogCategory::FileOperation, &log_message);
}

/// 记录检查过程
pub fn log_check_process(check_type: &str, input: &str, result: &str) {
    // 脱敏输入内容中的敏感信息
    let safe_input = mask_sensitive_content(input);
    
    let message = format!(
        r#"CheckType: {}
  Input: {}
  Result: {}"#,
        check_type, safe_input, result
    );
    write_log(LogCategory::CheckProcess, &message);
}

/// 记录格式修改
pub fn log_format_change(property: &str, old_value: &str, new_value: &str) {
    let message = format!(
        r#"Property: {}
  Old: {}
  New: {}"#,
        property, old_value, new_value
    );
    write_log(LogCategory::FormatChange, &message);
}

// ============== 辅助函数 ==============

/// 隐藏API Key（只显示前4位和后4位）
fn mask_api_key(text: &str) -> String {
    // 匹配常见的API Key模式
    let patterns = [
        (r#"["']?api[_-]?key["']?\s*[:=]\s*["']([a-zA-Z0-9_-]{20,})["']"#i, "api_key"),
        (r#"["']?authorization["']?\s*[:=]\s*["']Bearer\s+([a-zA-Z0-9_-]{20,})["']"#i, "bearer"),
        (r#"["']?access[_-]?token["']?\s*[:=]\s*["']([a-zA-Z0-9_-]{20,})["']"#i, "token"),
    ];
    
    let mut result = text.to_string();
    
    for (pattern, _) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                if let Some(key) = caps.get(1) {
                    let key_str = key.as_str();
                    if key_str.len() > 12 {
                        let masked = format!(
                            "{}****{}",
                            &key_str[..4],
                            &key_str[key_str.len()-4..]
                        );
                        return caps.get(0).unwrap().as_str().replace(key_str, &masked);
                    }
                }
                caps.get(0).unwrap().as_str().to_string()
            }).to_string();
        }
    }
    
    result
}

/// 隐藏敏感内容
fn mask_sensitive_content(text: &str) -> String {
    let mut result = text.to_string();
    
    // 身份证号
    if let Ok(re) = regex::Regex::new(r"\d{17}[\dXx]") {
        result = re.replace_all(&result, "***************").to_string();
    }
    
    // 手机号
    if let Ok(re) = regex::Regex::new(r"1[3-9]\d{9}") {
        result = re.replace_all(&result, "1**********").to_string();
    }
    
    // 限制长度
    if result.len() > 500 {
        result = format!("{}... (truncated)", &result[..500]);
    }
    
    result
}
