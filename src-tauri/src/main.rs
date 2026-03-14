// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod services;
mod models;
mod db;
mod security;

use tauri::Manager;
use std::sync::Arc;
use parking_lot::Mutex;
use security::{license::LicenseManager, rate_limiter::RateLimiter, audit::AuditLogger, device::get_device_fingerprint};

/// 应用状态
pub struct AppState {
    pub license_manager: Arc<Mutex<LicenseManager>>,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    pub audit_logger: Arc<Mutex<AuditLogger>>,
}

fn main() {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("标书智能检查工具启动中...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            db::init_database(&app_handle)?;
            
            // 获取应用数据目录
            let app_data_dir = app.path().app_data_dir()
                .expect("无法获取应用数据目录");
            std::fs::create_dir_all(&app_data_dir).ok();
            
            // 获取设备指纹
            let device_fp = get_device_fingerprint();
            log::info!("设备指纹: {}...{}",
                &device_fp[..8],
                &device_fp[device_fp.len()-8..]
            );
            
            // 初始化安全模块
            let license_manager = LicenseManager::new(&app_data_dir);
            let rate_limiter = RateLimiter::new(Default::default());
            let audit_logger = AuditLogger::new(&app_data_dir, device_fp.clone());
            
            // 存储到状态
            let state = AppState {
                license_manager: Arc::new(Mutex::new(license_manager)),
                rate_limiter: Arc::new(Mutex::new(rate_limiter)),
                audit_logger: Arc::new(Mutex::new(audit_logger)),
            };
            
            app.manage(state);
            
            // 记录启动日志
            log::info!("应用初始化完成");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 安全相关
            commands::security::get_device_info,
            commands::security::get_license_status,
            commands::security::import_license,
            commands::security::remove_license,
            commands::security::get_usage_stats,
            commands::security::get_audit_logs,
            
            // 文件操作
            commands::file::parse_file,
            commands::file::get_file_info,
            
            // 招标文件解析
            commands::tender_parse::parse_tender_document,
            
            // 脱敏引擎
            commands::desensitize::desensitize,
            commands::desensitize::get_default_rules,
            commands::desensitize::create_desensitize_rule,
            commands::desensitize::delete_desensitize_rule,
            
            // 检查执行
            commands::check::execute_full_check,
            commands::check::get_default_check_rules,
            
            // 自定义检查规则
            commands::custom_rules::get_custom_check_rules,
            commands::custom_rules::create_custom_check_rule,
            commands::custom_rules::update_custom_check_rule,
            commands::custom_rules::delete_custom_check_rule,
            commands::custom_rules::toggle_custom_check_rule,
            commands::custom_rules::export_custom_check_rules,
            commands::custom_rules::import_custom_check_rules,
            
            // API配置
            commands::api_config::test_api_connection,
            commands::api_config::save_api_config,
            commands::api_config::load_api_config,
            
            // 历史记录
            commands::history::get_check_history,
            commands::history::delete_check_history,
            commands::history::clear_all_history,
            
            // 报告导出
            commands::export::export_report,
            commands::export::export_comparison,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
