use serde::{Deserialize, Serialize};

/// 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub tender_file_path: Option<String>,
    pub bid_file_path: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}
