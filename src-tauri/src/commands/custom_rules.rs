use serde::{Deserialize, Serialize};

/// 自定义检查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCheckRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub severity: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建规则请求
#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub severity: String,
}

/// 更新规则请求
#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub prompt_template: Option<String>,
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

/// 获取所有自定义检查规则
#[tauri::command]
pub fn get_custom_check_rules() -> Result<Vec<CustomCheckRule>, String> {
    // TODO: 从数据库加载
    Ok(vec![])
}

/// 创建自定义检查规则
#[tauri::command]
pub fn create_custom_check_rule(request: CreateRuleRequest) -> Result<CustomCheckRule, String> {
    use uuid::Uuid;
    use chrono::Utc;
    
    let now = Utc::now().timestamp();
    
    Ok(CustomCheckRule {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        prompt_template: request.prompt_template,
        severity: request.severity,
        enabled: true,
        created_at: now,
        updated_at: now,
    })
}

/// 更新自定义检查规则
#[tauri::command]
pub fn update_custom_check_rule(_request: UpdateRuleRequest) -> Result<CustomCheckRule, String> {
    // TODO: 更新数据库
    Err("未实现".to_string())
}

/// 删除自定义检查规则
#[tauri::command]
pub fn delete_custom_check_rule(_id: String) -> Result<(), String> {
    // TODO: 从数据库删除
    Ok(())
}

/// 切换规则启用状态
#[tauri::command]
pub fn toggle_custom_check_rule(_id: String) -> Result<bool, String> {
    // TODO: 更新数据库
    Ok(true)
}

/// 导出规则配置
#[tauri::command]
pub fn export_custom_check_rules() -> Result<String, String> {
    let rules = get_custom_check_rules()?;
    serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())
}

/// 导入规则配置
#[tauri::command]
pub fn import_custom_check_rules(json: String) -> Result<usize, String> {
    let rules: Vec<CreateRuleRequest> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let mut count = 0;
    
    for _rule in rules {
        // TODO: 保存到数据库
        count += 1;
    }
    
    Ok(count)
}
