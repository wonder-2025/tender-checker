use serde::{Deserialize, Serialize};

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub category: String,
    pub name: String,
    pub severity: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub suggestion: Option<String>,
}
