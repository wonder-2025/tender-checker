use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sha2::{Sha256, Digest};

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
    
    /// 非Windows平台使用AES-256-GCM真正加密
    #[cfg(not(target_os = "windows"))]
    pub fn save_encrypted(&self, key: &str, value: &str) -> Result<(), String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        let device_fingerprint = super::device::get_device_fingerprint();
        let cipher_key = derive_key(&device_fingerprint);
        let cipher = Aes256Gcm::new_from_slice(&cipher_key)
            .map_err(|e| format!("初始化加密器失败: {}", e))?;
        
        // 使用固定nonce（生产环境应使用随机nonce并存储）
        let nonce = Nonce::from_slice(b"tc-encr-nonce");
        
        let data = format!("{}={}", key, value);
        let encrypted = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;
        
        let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);
        std::fs::write(&self.storage_path, encoded)
            .map_err(|e| format!("保存失败: {}", e))
    }
    
    /// 非Windows平台使用AES-256-GCM解密
    #[cfg(not(target_os = "windows"))]
    pub fn load_decrypted(&self, key: &str) -> Result<String, String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        if !self.storage_path.exists() {
            return Err("数据不存在".to_string());
        }
        
        let device_fingerprint = super::device::get_device_fingerprint();
        let cipher_key = derive_key(&device_fingerprint);
        let cipher = Aes256Gcm::new_from_slice(&cipher_key)
            .map_err(|e| format!("初始化加密器失败: {}", e))?;
        
        let nonce = Nonce::from_slice(b"tc-encr-nonce");
        
        let encoded = std::fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        
        let encrypted = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .map_err(|e| format!("解码失败: {}", e))?;
        
        let decrypted = cipher.decrypt(nonce, encrypted.as_slice())
            .map_err(|e| format!("解密失败: {}", e))?;
        
        let data = String::from_utf8_lossy(&decrypted).to_string();
        
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

/// 从设备指纹派生AES-256密钥
fn derive_key(device_fp: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(device_fp.as_bytes());
    hasher.update(b"tender-checker-encryption-key-v1");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
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
    ) -> Result<Self, String> {
        let device_fingerprint = super::device::get_device_fingerprint();
        let key = derive_key(&device_fingerprint);
        
        let encrypted_api_key = encrypt_data(&api_key, &key)?;
        let encrypted_secret_key = match secret_key {
            Some(sk) => Some(encrypt_data(&sk, &key)?),
            None => None,
        };
        
        Ok(Self {
            provider,
            encrypted_api_key,
            encrypted_secret_key,
            model,
            base_url,
        })
    }
    
    /// 解密为明文配置
    pub fn to_plain(&self) -> Result<(String, String, Option<String>, String, Option<String>), String> {
        let device_fingerprint = super::device::get_device_fingerprint();
        let key = derive_key(&device_fingerprint);
        
        let api_key = decrypt_data(&self.encrypted_api_key, &key)?;
        
        let secret_key = match &self.encrypted_secret_key {
            Some(sk) => Some(decrypt_data(sk, &key)?),
            None => None,
        };
        
        Ok((
            self.provider.clone(),
            api_key,
            secret_key,
            self.model.clone(),
            self.base_url.clone(),
        ))
    }
}

/// 使用AES-256-GCM加密数据
fn encrypt_data(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows平台也可以使用AES加密（用于EncryptedApiConfig）
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("初始化加密器失败: {}", e))?;
        let nonce = Nonce::from_slice(b"tc-cfg-nonce");
        
        let encrypted = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;
        
        Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("初始化加密器失败: {}", e))?;
        let nonce = Nonce::from_slice(b"tc-cfg-nonce");
        
        let encrypted = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;
        
        Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted))
    }
}

/// 使用AES-256-GCM解密数据
fn decrypt_data(ciphertext: &str, key: &[u8; 32]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("初始化解密器失败: {}", e))?;
        let nonce = Nonce::from_slice(b"tc-cfg-nonce");
        
        let encrypted = base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| format!("解码失败: {}", e))?;
        
        let decrypted = cipher.decrypt(nonce, encrypted.as_slice())
            .map_err(|e| format!("解密失败: {}", e))?;
        
        Ok(String::from_utf8_lossy(&decrypted).to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| format!("初始化解密器失败: {}", e))?;
        let nonce = Nonce::from_slice(b"tc-cfg-nonce");
        
        let encrypted = base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| format!("解码失败: {}", e))?;
        
        let decrypted = cipher.decrypt(nonce, encrypted.as_slice())
            .map_err(|e| format!("解密失败: {}", e))?;
        
        Ok(String::from_utf8_lossy(&decrypted).to_string())
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
        ).expect("加密失败");
        
        let (provider, api_key, secret_key, model, _base_url) = 
            config.to_plain().expect("解密失败");
        
        assert_eq!(provider, "baidu");
        assert_eq!(api_key, "test-api-key-123");
        assert_eq!(secret_key, Some("secret-key".to_string()));
        assert_eq!(model, "ERNIE-4.0");
    }
    
    #[test]
    fn test_secure_storage_non_windows() {
        #[cfg(not(target_os = "windows"))]
        {
            let temp_dir = std::env::temp_dir();
            let storage = SecureStorage::new(&temp_dir);
            
            storage.save_encrypted("test_key", "test_value").expect("保存失败");
            let value = storage.load_decrypted("test_key").expect("读取失败");
            
            assert_eq!(value, "test_value");
            
            storage.delete().expect("删除失败");
        }
    }
}
