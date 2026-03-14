use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::services::llm_client::{chat, LlmConfig};
use crate::commands::desensitize::{desensitize, DesensitizeRule};
use crate::commands::tender_parse::TenderExtraction;
use crate::security::audit::actions;
use crate::models::check_result::CheckResult;

/// 检查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub prompt_template: String,
    pub severity: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// 执行完整检查
#[tauri::command]
pub async fn execute_full_check(
    bid_file_path: String,
    tender_extraction: Option<TenderExtraction>,
    general_rules: Vec<CheckRule>,
    custom_rules: Vec<CheckRule>,
    desensitize_rules: Vec<DesensitizeRule>,
    llm_config: LlmConfig,
    state: State<'_, AppState>,
) -> Result<Vec<CheckResult>, String> {
    use crate::commands::file::parse_file;
    
    // 许可证验证（暂时禁用）
    // {
    //     let license_manager = state.license_manager.lock();
    //     license_manager.validate_license()?;
    // }
    
    // 频率限制检查
    {
        let rate_limiter = state.rate_limiter.lock();
        rate_limiter.check_rate_limit("check")?;
    }
    
    // 1. 解析投标文件
    let parse_result = parse_file(bid_file_path.clone()).await?;
    
    // 2. 脱敏
    let desensitized = desensitize(parse_result.content, desensitize_rules)?;
    
    // 3. 记录开始检查
    {
        let audit = state.audit_logger.lock();
        audit.log_success(actions::CHECK_START, &bid_file_path);
    }
    
    // 4. 执行检查
    let mut results = Vec::new();
    let mut check_success = true;
    
    // 通用检查
    for rule in &general_rules {
        let result = execute_single_check(rule, &desensitized.text, &llm_config).await;
        if result.status == "error" {
            check_success = false;
        }
        results.push(result);
    }
    
    // 自定义检查
    for rule in &custom_rules {
        if rule.enabled {
            let result = execute_single_check(rule, &desensitized.text, &llm_config).await;
            if result.status == "error" {
                check_success = false;
            }
            results.push(result);
        }
    }
    
    // 项目特定检查
    if let Some(extraction) = tender_extraction {
        let project_results = execute_project_checks(&extraction, &desensitized.text, &llm_config).await;
        results.extend(project_results);
    }
    
    // 5. 记录频率限制
    {
        let rate_limiter = state.rate_limiter.lock();
        rate_limiter.record_action("check", &bid_file_path, check_success);
    }
    
    // 6. 记录审计日志
    {
        let audit = state.audit_logger.lock();
        if check_success {
            audit.log_success(actions::CHECK_COMPLETE, &bid_file_path);
        } else {
            audit.log_failure(actions::CHECK_COMPLETE, &bid_file_path, "检查过程存在错误");
        }
    }
    
    Ok(results)
}

/// 执行单个检查
async fn execute_single_check(
    rule: &CheckRule,
    content: &str,
    llm_config: &LlmConfig,
) -> CheckResult {
    let prompt = format!(
        "{}\n\n---\n\n以下是标书内容（已脱敏）：\n\n{}",
        rule.prompt_template, content
    );
    
    match chat(llm_config, prompt).await {
        Ok(response) => CheckResult {
            id: rule.id.clone(),
            category: rule.category.clone(),
            name: rule.name.clone(),
            severity: rule.severity.clone(),
            status: "success".to_string(),
            result: Some(response),
            error: None,
        },
        Err(e) => CheckResult {
            id: rule.id.clone(),
            category: rule.category.clone(),
            name: rule.name.clone(),
            severity: rule.severity.clone(),
            status: "error".to_string(),
            result: None,
            error: Some(e),
        },
    }
}

/// 执行项目特定检查
async fn execute_project_checks(
    extraction: &TenderExtraction,
    content: &str,
    llm_config: &LlmConfig,
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    
    // 评分表应答检查
    let scoring_prompt = format!(
        "招标文件要求的评分表项目：\n{}\n\n请检查投标文件是否对所有评分项进行了应答，是否标注了对应页码。",
        extraction.scoring_table.iter()
            .map(|s| format!("- {}（{}分）", s.item, s.score))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    // 必填章节检查
    let sections_prompt = format!(
        "招标文件要求的必填章节：\n{}\n\n请检查投标文件是否包含所有章节，列出缺失的章节。",
        extraction.required_sections.iter()
            .map(|s| format!("- {}", s.name))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    // 工期一致性检查
    let period_prompt = format!(
        "招标文件要求的工期：{}\n\n请检查投标文件中的工期承诺是否与招标文件要求一致。",
        extraction.time_requirements.project_period
    );
    
    // 执行检查
    for (id, name, prompt) in [
        ("scoring_table", "评分表应答检查", scoring_prompt),
        ("required_sections", "必填章节检查", sections_prompt),
        ("period", "工期一致性检查", period_prompt),
    ] {
        let full_prompt = format!("{}\n\n---\n\n投标文件内容：\n{}", prompt, content);
        
        match chat(llm_config, full_prompt).await {
            Ok(response) => results.push(CheckResult {
                id: id.to_string(),
                category: "项目特定".to_string(),
                name: name.to_string(),
                severity: "error".to_string(),
                status: "success".to_string(),
                result: Some(response),
                error: None,
            }),
            Err(e) => results.push(CheckResult {
                id: id.to_string(),
                category: "项目特定".to_string(),
                name: name.to_string(),
                severity: "error".to_string(),
                status: "error".to_string(),
                result: None,
                error: Some(e),
            }),
        }
    }
    
    results
}

/// 获取默认检查规则
#[tauri::command]
pub fn get_default_check_rules() -> Vec<CheckRule> {
    vec![
        CheckRule {
            id: "project_name".to_string(),
            name: "项目名称一致性".to_string(),
            category: "通用检查".to_string(),
            prompt_template: "请检查以下标书中所有出现的项目名称和项目编号，检查是否完全一致。列出所有出现位置，标注不一致之处。".to_string(),
            severity: "error".to_string(),
        },
        CheckRule {
            id: "competitor".to_string(),
            name: "友商标识检查".to_string(),
            category: "通用检查".to_string(),
            prompt_template: "请检查以下标书是否出现以下友商关键词：华为、新华三、深信服、奇安信、天融信、启明星辰、绿盟、山石网科、亚信安全、迪普科技、安恒信息。列出所有发现的位置和上下文。".to_string(),
            severity: "error".to_string(),
        },
        CheckRule {
            id: "amount".to_string(),
            name: "金额一致性检查".to_string(),
            category: "通用检查".to_string(),
            prompt_template: "请提取以下标书中所有金额信息，包括数字和中文大写。检查：1. 大小写是否对应；2. 总价是否等于分项之和；3. 前后文金额是否一致。列出所有金额及位置。".to_string(),
            severity: "error".to_string(),
        },
        CheckRule {
            id: "typo".to_string(),
            name: "错别字检查".to_string(),
            category: "通用检查".to_string(),
            prompt_template: "请检查以下标书中的错别字，包括：1. 常见错别字；2. 专业术语错误；3. 表达不通顺之处。列出所有发现的问题及修改建议。".to_string(),
            severity: "warning".to_string(),
        },
        CheckRule {
            id: "date".to_string(),
            name: "日期检查".to_string(),
            category: "通用检查".to_string(),
            prompt_template: "请检查以下标书中所有日期信息，检查：1. 日期格式是否统一；2. 是否有过期日期；3. 投标有效期是否符合常规要求（通常90-180天）。".to_string(),
            severity: "info".to_string(),
        },
    ]
}
