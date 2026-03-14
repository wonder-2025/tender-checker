use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::security::device::get_device_fingerprint;

/// 许可证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// 许可证ID
    pub license_id: String,
    /// 许可证密钥（加密后）
    pub license_key: String,
    /// 设备指纹
    pub device_fingerprint: String,
    /// 用户名
    pub user_name: String,
    /// 公司名称
    pub company: String,
    /// 联系邮箱
    pub email: String,
    /// 有效期
    pub expires_at: DateTime<Utc>,
    /// 每日检查上限
    pub max_checks_per_day: u32,
    /// 已启用的功能
    pub features: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 签名（RSA）
    pub signature: String,
}

/// 许可证状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub valid: bool,
    pub license: Option<License>,
    pub message: String,
    pub days_remaining: i64,
}

/// 许可证管理器
pub struct LicenseManager {
    license_path: std::path::PathBuf,
}

impl LicenseManager {
    pub fn new(app_data_dir: &std::path::Path) -> Self {
        Self {
            license_path: app_data_dir.join("license.dat"),
        }
    }
    
    /// 导入许可证
    pub fn import_license(&self, license_key: &str) -> Result<License, String> {
        // 解析许可证
        let license = parse_license(license_key)?;
        
        // 验证签名
        verify_license_signature(&license)?;
        
        // 验证有效期
        if license.expires_at < Utc::now() {
            return Err("许可证已过期".to_string());
        }
        
        // 保存许可证
        self.save_license(&license)?;
        
        Ok(license)
    }
    
    /// 获取当前许可证状态
    pub fn get_license_status(&self) -> LicenseStatus {
        match self.load_license() {
            Ok(license) => {
                // 验证设备绑定
                let current_fp = get_device_fingerprint();
                if license.device_fingerprint != current_fp {
                    return LicenseStatus {
                        valid: false,
                        license: Some(license.clone()),
                        message: "许可证与当前设备不匹配".to_string(),
                        days_remaining: 0,
                    };
                }
                
                // 计算剩余天数
                let now = Utc::now();
                let days_remaining = (license.expires_at - now).num_days();
                
                if license.expires_at < now {
                    LicenseStatus {
                        valid: false,
                        license: Some(license),
                        message: "许可证已过期".to_string(),
                        days_remaining: 0,
                    }
                } else {
                    LicenseStatus {
                        valid: true,
                        license: Some(license),
                        message: "许可证有效".to_string(),
                        days_remaining,
                    }
                }
            }
            Err(_) => LicenseStatus {
                valid: false,
                license: None,
                message: "未导入许可证".to_string(),
                days_remaining: 0,
            },
        }
    }
    
    /// 验证许可证
    pub fn validate_license(&self) -> Result<License, String> {
        let status = self.get_license_status();
        
        if !status.valid {
            return Err(status.message);
        }
        
        status.license.ok_or("许可证不存在".to_string())
    }
    
    /// 检查功能是否可用
    pub fn check_feature(&self, feature: &str) -> Result<bool, String> {
        let license = self.validate_license()?;
        Ok(license.features.contains(&feature.to_string()))
    }
    
    /// 保存许可证到本地
    fn save_license(&self, license: &License) -> Result<(), String> {
        let content = serde_json::to_string_pretty(license)
            .map_err(|e| format!("序列化失败: {}", e))?;
        
        // 简单编码（生产环境应使用加密）
        let encoded = base64::encode(content);
        
        std::fs::write(&self.license_path, encoded)
            .map_err(|e| format!("保存失败: {}", e))?;
        
        Ok(())
    }
    
    /// 从本地加载许可证
    fn load_license(&self) -> Result<License, String> {
        if !self.license_path.exists() {
            return Err("许可证文件不存在".to_string());
        }
        
        let encoded = std::fs::read_to_string(&self.license_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        
        let content = base64::decode(&encoded)
            .map_err(|e| format!("解码失败: {}", e))?;
        
        let license: License = serde_json::from_slice(&content)
            .map_err(|e| format!("解析失败: {}", e))?;
        
        Ok(license)
    }
    
    /// 删除许可证
    pub fn remove_license(&self) -> Result<(), String> {
        if self.license_path.exists() {
            std::fs::remove_file(&self.license_path)
                .map_err(|e| format!("删除失败: {}", e))?;
        }
        Ok(())
    }
}

/// 解析许可证字符串
fn parse_license(license_key: &str) -> Result<License, String> {
    // 解码Base64
    let decoded = base64::decode(license_key)
        .map_err(|e| format!("许可证格式错误: {}", e))?;
    
    // 解析JSON
    let license: License = serde_json::from_slice(&decoded)
        .map_err(|e| format!("许可证解析失败: {}", e))?;
    
    Ok(license)
}

/// 验证许可证签名
fn verify_license_signature(license: &License) -> Result<(), String> {
    // TODO: 实现RSA签名验证
    // 当前简化实现：直接通过
    Ok(())
}

/// 生成许可证（仅管理员使用）
pub fn generate_license(
    device_fingerprint: String,
    user_name: String,
    company: String,
    email: String,
    days_valid: u32,
    max_checks_per_day: u32,
    features: Vec<String>,
) -> Result<String, String> {
    use uuid::Uuid;
    
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(days_valid as i64);
    
    let license = License {
        license_id: format!("TC-{}-{}", 
            now.format("%Y"),
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        license_key: String::new(),
        device_fingerprint,
        user_name,
        company,
        email,
        expires_at,
        max_checks_per_day,
        features,
        created_at: now,
        signature: String::new(), // TODO: 实现签名
    };
    
    // 序列化并编码
    let json = serde_json::to_string(&license)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    let encoded = base64::encode(&json);
    
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_license() {
        let license_key = generate_license(
            "test-device-fp".to_string(),
            "张三".to_string(),
            "测试公司".to_string(),
            "test@example.com".to_string(),
            365,
            50,
            vec!["basic_check".to_string()],
        ).unwrap();
        
        println!("Generated license key:\n{}", license_key);
        assert!(!license_key.is_empty());
    }
}
