use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 脱敏规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesensitizeRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub replacement: String,
    pub enabled: bool,
    pub priority: u32,
}

/// 脱敏结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DesensitizeResult {
    pub text: String,
    pub sensitive_map: HashMap<String, String>,
    pub stats: DesensitizeStats,
}

/// 脱敏统计
#[derive(Debug, Serialize, Deserialize)]
pub struct DesensitizeStats {
    pub total_replacements: usize,
    pub by_rule: HashMap<String, usize>,
}

/// 执行脱敏
#[tauri::command]
pub fn desensitize(
    text: String,
    rules: Vec<DesensitizeRule>,
) -> Result<DesensitizeResult, String> {
    let mut result = text;
    let mut sensitive_map = HashMap::new();
    let mut stats = DesensitizeStats {
        total_replacements: 0,
        by_rule: HashMap::new(),
    };
    
    // 按优先级排序
    let mut sorted_rules = rules;
    sorted_rules.sort_by(|a, b| a.priority.cmp(&b.priority));
    
    // 执行脱敏
    for rule in sorted_rules.iter().filter(|r| r.enabled) {
        if let Ok(re) = Regex::new(&rule.pattern) {
            let captures: Vec<_> = re.find_iter(&result).collect();
            
            for cap in captures {
                let original = cap.as_str().to_string();
                let id = format!("{}_{}", rule.id, sensitive_map.len());
                
                result = re.replace(&result.clone(), &rule.replacement).to_string();
                sensitive_map.insert(id.clone(), original);
                
                *stats.by_rule.entry(rule.name.clone()).or_insert(0) += 1;
                stats.total_replacements += 1;
            }
        }
    }
    
    Ok(DesensitizeResult {
        text: result,
        sensitive_map,
        stats,
    })
}

/// 获取默认脱敏规则
#[tauri::command]
pub fn get_default_rules() -> Vec<DesensitizeRule> {
    vec![
        DesensitizeRule {
            id: "company_name".into(),
            name: "\u{516c}\u{53f8}\u{540d}\u{79f0}".into(),
            pattern: r"([^\s]{2,20})(\u{6709}\u{9650}\u{516c}\u{53f8}|\u{80a1}\u{4efd}\u{6709}\u{9650}\u{516c}\u{53f8}|\u{96c6}\u{56e2}|\u{6709}\u{9650}\u{8d23}\u{4efb}\u{516c}\u{53f8})".into(),
            replacement: "[\u{516c}\u{53f8}\u{540d}\u{79f0}]".into(),
            enabled: true,
            priority: 1,
        },
        DesensitizeRule {
            id: "amount".into(),
            name: "\u{91d1}\u{989d}".into(),
            pattern: r"(\d{1,3}(,\d{3})*(\.\d{2})?)\s*(\u{5143}|\u{4e07}\u{5143}|\u{4ebf}\u{5143})".into(),
            replacement: "[\u{91d1}\u{989d}]".into(),
            enabled: true,
            priority: 2,
        },
        DesensitizeRule {
            id: "phone".into(),
            name: "\u{624b}\u{673a}\u{53f7}".into(),
            pattern: r"1[3-9]\d{9}".into(),
            replacement: "[\u{624b}\u{673a}\u{53f7}]".into(),
            enabled: true,
            priority: 3,
        },
        DesensitizeRule {
            id: "id_card".into(),
            name: "\u{8eab}\u{4efd}\u{8bc1}\u{53f7}".into(),
            pattern: r"\d{17}[\dXx]".into(),
            replacement: "[\u{8eab}\u{4efd}\u{8bc1}\u{53f7}]".into(),
            enabled: true,
            priority: 4,
        },
        DesensitizeRule {
            id: "bank_account".into(),
            name: "\u{94f6}\u{884c}\u{8d26}\u{53f7}".into(),
            pattern: r"\d{16,19}".into(),
            replacement: "[\u{94f6}\u{884c}\u{8d26}\u{53f7}]".into(),
            enabled: true,
            priority: 5,
        },
        DesensitizeRule {
            id: "email".into(),
            name: "\u{90ae}\u{7bb1}".into(),
            pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".into(),
            replacement: "[\u{90ae}\u{7bb1}]".into(),
            enabled: true,
            priority: 6,
        },
    ]
}

/// 创建自定义脱敏规则
#[tauri::command]
pub fn create_desensitize_rule(rule: DesensitizeRule) -> Result<DesensitizeRule, String> {
    Ok(rule)
}

/// 删除自定义脱敏规则
#[tauri::command]
pub fn delete_desensitize_rule(_id: String) -> Result<(), String> {
    Ok(())
}
