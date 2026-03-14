use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub hostname: String,
    pub cpu_id: String,
    pub disk_id: String,
    pub motherboard_id: String,
}

/// 获取设备指纹
pub fn get_device_fingerprint() -> String {
    let mut hasher = Sha256::new();
    
    // 收集设备信息
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_default();
    
    hasher.update(hostname.as_bytes());
    
    // Windows平台获取硬件信息
    #[cfg(target_os = "windows")]
    {
        if let Ok(cpu_id) = get_cpu_id_windows() {
            hasher.update(cpu_id.as_bytes());
        }
        if let Ok(disk_id) = get_disk_id_windows() {
            hasher.update(disk_id.as_bytes());
        }
        if let Ok(mb_id) = get_motherboard_id_windows() {
            hasher.update(mb_id.as_bytes());
        }
    }
    
    // 非Windows平台使用hostname + 随机salt
    #[cfg(not(target_os = "windows"))]
    {
        hasher.update(b"tender-checker-salt-v1");
    }
    
    format!("{:x}", hasher.finalize())
}

/// 获取完整设备信息
pub fn get_device_info() -> DeviceInfo {
    DeviceInfo {
        hostname: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_default(),
        cpu_id: get_cpu_id().unwrap_or_default(),
        disk_id: get_disk_id().unwrap_or_default(),
        motherboard_id: get_motherboard_id().unwrap_or_default(),
    }
}

/// 获取CPU ID
#[cfg(target_os = "windows")]
fn get_cpu_id() -> Result<String, String> {
    get_cpu_id_windows()
}

#[cfg(not(target_os = "windows"))]
fn get_cpu_id() -> Result<String, String> {
    Ok(String::new())
}

/// 获取磁盘序列号
#[cfg(target_os = "windows")]
fn get_disk_id() -> Result<String, String> {
    get_disk_id_windows()
}

#[cfg(not(target_os = "windows"))]
fn get_disk_id() -> Result<String, String> {
    Ok(String::new())
}

/// 获取主板序列号
#[cfg(target_os = "windows")]
fn get_motherboard_id() -> Result<String, String> {
    get_motherboard_id_windows()
}

#[cfg(not(target_os = "windows"))]
fn get_motherboard_id() -> Result<String, String> {
    Ok(String::new())
}

/// Windows平台获取CPU ID
#[cfg(target_os = "windows")]
fn get_cpu_id_windows() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("wmic")
        .args(&["cpu", "get", "ProcessorId"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    Ok(id)
}

/// Windows平台获取磁盘序列号
#[cfg(target_os = "windows")]
fn get_disk_id_windows() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("wmic")
        .args(&["diskdrive", "get", "SerialNumber"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    Ok(id)
}

/// Windows平台获取主板序列号
#[cfg(target_os = "windows")]
fn get_motherboard_id_windows() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("wmic")
        .args(&["baseboard", "get", "SerialNumber"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_fingerprint() {
        let fp = get_device_fingerprint();
        assert!(!fp.is_empty());
        println!("Device fingerprint: {}", fp);
    }
}
