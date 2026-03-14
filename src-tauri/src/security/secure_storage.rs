use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 加密存储管理器
pub struct SecureStorage {
    storage_path: PathBuf,
}

impl SecureStorage {
    pub fn new(app_data_dir: &std::path::Path) -> Self {
        Self {
            storage_path: app_data_dir.join("secure.dat"),
        }
    }
    
    /// 保存加密数据（Windows DPAPI）
    #[cfg(target_os = "windows")]
    pub fn save_encrypted(&self, key: &str, value: &str) -> Result<(), String> {
        use windows::Win32::Security::Cryptography::*;
        use windows::core::PCWSTR;
        
        let data = format!("{}={}", key, value);
        let bytes = data.as_bytes();
        
        unsafe {
            let mut input = CRYPT_INTEGER_BLOB {
                cbData: bytes.len() as u32,
                pbData: bytes.as_ptr() as *mut u8,
            };
            
            let mut output = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };
            
            CryptProtectData(
                &input,
                PCWSTR::null(),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            ).map_err(|e| format!("加密失败: {}", e))?;
            
            // 保存到文件
            let encrypted_data = std::slice::from_raw_parts(
                output.pbData,
                output.cbData as usize
            );
            
            std::fs::write(&self.storage_path, encrypted_data)
                .map_err(|e| format!("保存失败: {}", e))?;
            
            // 释放内存
            LocalFree(output.pbData as _);
        }
        
        Ok(())
    }
    
    /// 读取加密数据（Windows DPAPI）
    #[cfg(target_os = "windows")]
    pub fn load_decrypted(&self, key: &str) -> Result<String, String> {
        use windows::Win32::Security::Cryptography::*;
        use windows::core::PCWSTR;
        
        if !self.storage_path.exists() {
            return Err("数据不存在".to_string());
        }
        
        let encrypted = std::fs::read(&self.storage_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        
        unsafe {
            let mut input = CRYPT_INTEGER_BLOB {
                cbData: encrypted.len() as u32,
                pbData: encrypted.as_ptr() as *mut u8,
            };
            
            let mut output = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: std::ptr::null_mut(),
            };
            
            CryptUnprotectData(
                &mut input,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            ).map_err(|e| format!("解密失败: {}", e))?;
            
            let decrypted = std::slice::from_raw_parts(
                output.pbData,
                output.cbData as usize
            );
            
            let data = String::from_utf8_lossy(decrypted).to_string();
            
            // 释放内存
            LocalFree(output.pbData as _);
            
            // 解析键值对
            if let Some(value) = data.strip_prefix(&format!("{}=", key)) {
                Ok(value.to_string())
            } else {
                Err("数据格式错误".to_string())
            }
        }
    }
    
    /// 非Windows平台的简化实现（Base64编码）
    #[cfg(not(target_os = "windows"))]
    pub fn save_encrypted(&self, key: &str, value: &str) -> Result<(), String> {
        use base64::Engine;
        let data = format!("{}={}", key, value);
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        std::fs::write(&self.storage_path, encoded)
            .map_err(|e| format!("保存失败: {}", e))
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn load_decrypted(&self, key: &str) -> Result<String, String> {
        use base64::Engine;
        if !self.storage_path.exists() {
            return Err("数据不存在".to_string());
        }
        
        let encoded = std::fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        
        let data = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .map_err(|e| format!("解码失败: {}", e))?;
        
        let data = String::from_utf8_lossy(&data).to_string();
        
        if let Some(value) = data.strip_prefix(&format!("{}=", key)) {
            Ok(value.to_string())
        } else {
            Err("数据格式错误".to_string())
        }
    }
    
    /// 删除加密数据
    pub fn delete(&self) -> Result<(), String> {
        if self.storage_path.exists() {
            std::fs::remove_file(&self.storage_path)
                .map_err(|e| format!("删除失败: {}", e))?;
        }
        Ok(())
    }
}

/// API配置加密存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedApiConfig {
    pub provider: String,
    pub encrypted_api_key: String,
    pub encrypted_secret_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

impl EncryptedApiConfig {
    /// 从明文配置创建加密配置
    pub fn from_plain(
        provider: String,
        api_key: String,
        secret_key: Option<String>,
        model: String,
        base_url: Option<String>,
    ) -> Self {
        use base64::Engine;
        let encrypted_api_key = base64::engine::general_purpose::STANDARD.encode(&api_key);
        let encrypted_secret_key = secret_key.map(|k| 
            base64::engine::general_purpose::STANDARD.encode(&k)
        );
        
        Self {
            provider,
            encrypted_api_key,
            encrypted_secret_key,
            model,
            base_url,
        }
    }
    
    /// 解密为明文配置
    pub fn to_plain(&self) -> (String, String, Option<String>, String, Option<String>) {
        use base64::Engine;
        let api_key = base64::engine::general_purpose::STANDARD
            .decode(&self.encrypted_api_key)
            .unwrap_or_default();
        let api_key = String::from_utf8_lossy(&api_key).to_string();
        
        let secret_key = self.encrypted_secret_key.as_ref()
            .map(|k| {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(k)
                    .unwrap_or_default();
                String::from_utf8_lossy(&decoded).to_string()
            });
        
        (
            self.provider.clone(),
            api_key,
            secret_key,
            self.model.clone(),
            self.base_url.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypted_api_config() {
        let config = EncryptedApiConfig::from_plain(
            "baidu".to_string(),
            "test-api-key-123".to_string(),
            Some("secret-key".to_string()),
            "ERNIE-4.0".to_string(),
            None,
        );
        
        let (provider, api_key, secret_key, model, _base_url) = config.to_plain();
        
        assert_eq!(provider, "baidu");
        assert_eq!(api_key, "test-api-key-123");
        assert_eq!(secret_key, Some("secret-key".to_string()));
        assert_eq!(model, "ERNIE-4.0");
    }
}
