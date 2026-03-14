use serde::{Deserialize, Serialize};

/// 历史记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub project_name: String,
    pub bid_file_name: String,
    pub check_time: String,
    pub status: String,
    pub error_count: u32,
}

/// 获取检查历史
#[tauri::command]
pub fn get_check_history() -> Result<Vec<HistoryItem>, String> {
    // TODO: 从数据库加载
    Ok(vec![])
}

/// 删除检查历史
#[tauri::command]
pub fn delete_check_history(_id: String) -> Result<(), String> {
    // TODO: 从数据库删除
    Ok(())
}

/// 清除所有历史
#[tauri::command]
pub fn clear_all_history() -> Result<(), String> {
    // TODO: 清除数据库中所有历史
    Ok(())
}
