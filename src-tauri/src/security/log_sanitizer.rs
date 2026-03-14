use regex::Regex;
use std::sync::LazyLock;

/// 敏感信息模式（静态初始化）
static SENSITIVE_PATTERNS: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
    vec![
        // 手机号
        (Regex::new(r"1[3-9]\d{9}").unwrap(), "[手机号]"),
        
        // 身份证号
        (Regex::new(r"\d{17}[\dXx]").unwrap(), "[身份证号]"),
        
        // 银行卡号
        (Regex::new(r"\d{16,19}").unwrap(), "[银行卡号]"),
        
        // 邮箱
        (Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(), "[邮箱]"),
        
        // 金额
        (Regex::new(r"(\d{1,3}(,\d{3})*(\.\d{2})?)\s*(元|万元|亿元)").unwrap(), "[金额]"),
        
        // 公司名称
        (Regex::new(r"[^\s]{2,30}(有限公司|股份有限公司|集团|有限责任公司)").unwrap(), "[公司名]"),
        
        // IP地址
        (Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap(), "[IP地址]"),
        
        // API密钥模式
        (Regex::new(r"[A-Za-z0-9]{32,}").unwrap(), "[密钥]"),
    ]
});

/// 日志脱敏器
pub struct LogSanitizer;

impl LogSanitizer {
    /// 脱敏文本
    pub fn sanitize(text: &str) -> String {
        let mut result = text.to_string();
        
        for (pattern, replacement) in SENSITIVE_PATTERNS.iter() {
            result = pattern.replace_all(&result, *replacement).to_string();
        }
        
        result
    }
    
    /// 脱敏并截断
    pub fn sanitize_truncate(text: &str, max_len: usize) -> String {
        let sanitized = Self::sanitize(text);
        
        if sanitized.len() > max_len {
            format!("{}...[已截断]", &sanitized[..max_len])
        } else {
            sanitized
        }
    }
}

/// 日志宏（自动脱敏）
#[macro_export]
macro_rules! safe_log {
    (info, $($arg:tt)*) => {
        log::info!("{}", $crate::security::log_sanitizer::LogSanitizer::sanitize(&format!($($arg)*)))
    };
    (warn, $($arg:tt)*) => {
        log::warn!("{}", $crate::security::log_sanitizer::LogSanitizer::sanitize(&format!($($arg)*)))
    };
    (error, $($arg:tt)*) => {
        log::error!("{}", $crate::security::log_sanitizer::LogSanitizer::sanitize(&format!($($arg)*)))
    };
    (debug, $($arg:tt)*) => {
        log::debug!("{}", $crate::security::log_sanitizer::LogSanitizer::sanitize(&format!($($arg)*)))
    };
}

/// 文件路径脱敏
pub fn sanitize_file_path(path: &str) -> String {
    // 只保留文件名，隐藏完整路径
    if let Some(filename) = path.rsplit(|c| c == '/' || c == '\\').next() {
        format!("[文件]/{}", filename)
    } else {
        "[文件]".to_string()
    }
}

/// API响应脱敏
pub fn sanitize_api_response(response: &str) -> String {
    // 截断响应，避免泄露大量内容
    LogSanitizer::sanitize_truncate(response, 200)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_phone() {
        let text = "联系电话：13812345678";
        let sanitized = LogSanitizer::sanitize(text);
        assert_eq!(sanitized, "联系电话：[手机号]");
    }
    
    #[test]
    fn test_sanitize_id_card() {
        let text = "身份证：123456789012345678";
        let sanitized = LogSanitizer::sanitize(text);
        assert_eq!(sanitized, "身份证：[身份证号]");
    }
    
    #[test]
    fn test_sanitize_amount() {
        let text = "金额为12,345.67元";
        let sanitized = LogSanitizer::sanitize(text);
        assert_eq!(sanitized, "金额为[金额]");
    }
    
    #[test]
    fn test_sanitize_company() {
        let text = "XX科技有限公司中标";
        let sanitized = LogSanitizer::sanitize(text);
        assert!(sanitized.contains("[公司名]"));
    }
    
    #[test]
    fn test_sanitize_multiple() {
        let text = "张三 13812345678 身份证123456789012345678";
        let sanitized = LogSanitizer::sanitize(text);
        assert_eq!(sanitized, "张三 [手机号] 身份证[身份证号]");
    }
}
