use tauri::State;
use crate::AppState;
use crate::security::{
    device::DeviceInfo,
    license::LicenseStatus,
    rate_limiter::UsageStats,
    audit::{AuditLog, AuditStats},
};

/// 获取设备指纹
#[tauri::command]
pub fn get_device_info() -> DeviceInfo {
    crate::security::device::get_device_info()
}

/// 获取许可证状态
#[tauri::command]
pub fn get_license_status(state: State<'_, AppState>) -> LicenseStatus {
    state.license_manager.lock().get_license_status()
}

/// 导入许可证
#[tauri::command]
pub fn import_license(
    license_key: String,
    state: State<'_, AppState>,
) -> Result<crate::security::license::License, String> {
    let manager = state.license_manager.lock();
    let license = manager.import_license(&license_key)?;
    
    // 更新审计日志的用户信息
    let mut audit = state.audit_logger.lock();
    audit.set_user(license.user_name.clone(), license.license_id.clone());
    
    // 记录日志
    audit.log_success("license_import", &license.license_id);
    
    Ok(license)
}

/// 删除许可证
#[tauri::command]
pub fn remove_license(state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.license_manager.lock();
    manager.remove_license()
}

/// 获取使用统计
#[tauri::command]
pub fn get_usage_stats(state: State<'_, AppState>) -> UsageStats {
    state.rate_limiter.lock().get_today_stats()
}

/// 获取审计日志
#[tauri::command]
pub fn get_audit_logs(
    state: State<'_, AppState>,
    limit: usize,
) -> Vec<AuditLog> {
    state.audit_logger.lock().read_logs(limit)
}

/// 获取审计统计
#[tauri::command]
pub fn get_audit_stats(
    state: State<'_, AppState>,
    days: u32,
) -> AuditStats {
    state.audit_logger.lock().get_stats(days)
}

/// 检查操作频率限制
#[tauri::command]
pub fn check_rate_limit(
    action: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 先验证许可证
    {
        let manager = state.license_manager.lock();
        manager.validate_license()?;
    }
    
    // 检查频率限制
    let limiter = state.rate_limiter.lock();
    limiter.check_rate_limit(&action)
}

/// 更新频率限制配置
#[tauri::command]
pub fn update_rate_limit_config(
    _config: crate::security::rate_limiter::RateLimitConfig,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // 此操作需要管理员权限，暂时禁用
    Err("此功能需要管理员权限".to_string())
}

/// 检查功能是否可用
#[tauri::command]
pub fn check_feature(
    feature: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let manager = state.license_manager.lock();
    manager.check_feature(&feature)
}
