// Debug 调试命令
// 设计者: wonder-宏 & JARVIS AI Assistant

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// Debug配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub log_level: String,
    pub log_items: Vec<String>,
    pub log_file_path: String,
    pub log_file_size: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_level: "debug".to_string(),
            log_items: vec![
                "desensitize".to_string(),
                "apiRequest".to_string(),
                "apiResponse".to_string(),
                "fileOperation".to_string(),
                "checkProcess".to_string(),
            ],
            log_file_path: String::new(),
            log_file_size: "0 KB".to_string(),
        }
    }
}

/// 获取Debug配置
#[tauri::command]
pub fn get_debug_config(data_dir: String) -> Result<DebugConfig, String> {
    let config_path = PathBuf::from(&data_dir).join("debug_config.json");
    
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置失败: {}", e))?;
        let mut config: DebugConfig = serde_json::from_str(&content)
            .unwrap_or_default();
        
        // 更新日志文件路径和大小
        let log_path = get_log_path(&data_dir);
        config.log_file_path = log_path.to_string_lossy().to_string();
        config.log_file_size = get_log_file_size(&log_path);
        
        Ok(config)
    } else {
        let mut config = DebugConfig::default();
        let log_path = get_log_path(&data_dir);
        config.log_file_path = log_path.to_string_lossy().to_string();
        Ok(config)
    }
}

/// 保存Debug配置
#[tauri::command]
pub fn save_debug_config(data_dir: String, config: DebugConfig) -> Result<(), String> {
    let config_path = PathBuf::from(&data_dir).join("debug_config.json");
    
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("保存配置失败: {}", e))?;
    
    Ok(())
}

/// 读取日志文件
#[tauri::command]
pub fn read_log_file(data_dir: String, lines: Option<usize>) -> Result<String, String> {
    let log_path = get_log_path(&data_dir);
    
    if !log_path.exists() {
        return Ok("日志文件不存在".to_string());
    }
    
    let content = fs::read_to_string(&log_path)
        .map_err(|e| format!("读取日志失败: {}", e))?;
    
    if let Some(n) = lines {
        let all_lines: Vec<&str> = content.lines().collect();
        let start = if all_lines.len() > n { all_lines.len() - n } else { 0 };
        Ok(all_lines[start..].join("\n"))
    } else {
        Ok(content)
    }
}

/// 清空日志文件
#[tauri::command]
pub fn clear_log_file(data_dir: String) -> Result<(), String> {
    let log_path = get_log_path(&data_dir);
    
    if log_path.exists() {
        fs::write(&log_path, "")
            .map_err(|e| format!("清空日志失败: {}", e))?;
    }
    
    Ok(())
}

/// 导出日志文件
#[tauri::command]
pub fn export_log_file(data_dir: String, export_path: String) -> Result<String, String> {
    let log_path = get_log_path(&data_dir);
    
    if !log_path.exists() {
        return Err("日志文件不存在".to_string());
    }
    
    let export_file = PathBuf::from(&export_path);
    fs::copy(&log_path, &export_file)
        .map_err(|e| format!("导出日志失败: {}", e))?;
    
    Ok(export_file.to_string_lossy().to_string())
}

/// 获取日志文件路径
#[tauri::command]
pub fn get_log_path_cmd(data_dir: String) -> String {
    get_log_path(&data_dir).to_string_lossy().to_string()
}

/// 打开日志目录
#[tauri::command]
pub fn open_log_folder(data_dir: String) -> Result<(), String> {
    let log_dir = PathBuf::from(&data_dir).join("logs");
    
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)
            .map_err(|e| format!("创建日志目录失败: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    Ok(())
}

// ============== 内部函数 ==============

fn get_log_path(data_dir: &str) -> PathBuf {
    let log_dir = PathBuf::from(data_dir).join("logs");
    let today = Local::now().format("%Y-%m-%d").to_string();
    log_dir.join(format!("debug-{}.log", today))
}

fn get_log_file_size(path: &PathBuf) -> String {
    if !path.exists() {
        return "0 KB".to_string();
    }
    
    match fs::metadata(path) {
        Ok(meta) => {
            let size = meta.len();
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
            }
        }
        Err(_) => "未知".to_string()
    }
}
