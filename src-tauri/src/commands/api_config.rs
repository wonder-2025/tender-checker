use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::security::secure_storage::SecureStorage;

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub secret_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

/// API测试结果
#[derive(Debug, Serialize)]
pub struct ApiTestResult {
    pub success: bool,
    pub message: String,
}

/// 测试API连接
#[tauri::command]
pub async fn test_api_connection(config: LlmConfig) -> Result<ApiTestResult, String> {
    use crate::services::llm_client::chat;
    use crate::security::log_sanitizer::LogSanitizer;
    
    // 记录测试（脱敏）
    log::info!("测试API连接: provider={}, model={}", 
        config.provider, 
        config.model
    );
    
    match chat(&config, "你好，这是一个测试消息。".to_string()).await {
        Ok(_) => Ok(ApiTestResult {
            success: true,
            message: "API连接成功".to_string(),
        }),
        Err(e) => {
            // 脱敏错误信息
            let sanitized_error = LogSanitizer::sanitize(&e);
            Ok(ApiTestResult {
                success: false,
                message: format!("API连接失败: {}", sanitized_error),
            })
        }
    }
}

/// 保存API配置（加密存储）
#[tauri::command]
pub fn save_api_config(
    config: LlmConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use crate::security::secure_storage::EncryptedApiConfig;
    use crate::security::audit::actions;
    
    // 加密配置
    let encrypted = EncryptedApiConfig::from_plain(
        config.provider.clone(),
        config.api_key,
        config.secret_key,
        config.model,
        config.base_url,
    );
    
    // 序列化并保存
    let json = serde_json::to_string(&encrypted)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    let storage = SecureStorage::new(&get_app_data_dir(&state));
    storage.save_encrypted("api_config", &json)?;
    
    // 记录审计日志（不记录敏感信息）
    let audit = state.audit_logger.lock();
    audit.log_success(actions::CONFIG_CHANGE, "API配置已更新");
    
    log::info!("API配置已加密保存: provider={}", config.provider);
    
    Ok(())
}

/// 加载API配置（解密）
#[tauri::command]
pub fn load_api_config(
    state: State<'_, AppState>,
) -> Result<Option<LlmConfig>, String> {
    use crate::security::secure_storage::EncryptedApiConfig;
    
    let storage = SecureStorage::new(&get_app_data_dir(&state));
    
    match storage.load_decrypted("api_config") {
        Ok(json) => {
            let encrypted: EncryptedApiConfig = serde_json::from_str(&json)
                .map_err(|e| format!("解析失败: {}", e))?;
            
            let (provider, api_key, secret_key, model, base_url) = encrypted.to_plain();
            
            Ok(Some(LlmConfig {
                provider,
                api_key,
                secret_key,
                model,
                base_url,
            }))
        }
        Err(_) => Ok(None),
    }
}

/// 删除API配置
#[tauri::command]
pub fn delete_api_config(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let storage = SecureStorage::new(&get_app_data_dir(&state));
    storage.delete()?;
    
    log::info!("API配置已删除");
    Ok(())
}

/// 获取应用数据目录
fn get_app_data_dir(state: &State<'_, AppState>) -> std::path::PathBuf {
    // 从状态中获取，或使用默认路径
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("data")
}
