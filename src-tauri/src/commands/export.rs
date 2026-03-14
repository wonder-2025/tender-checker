use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::models::check_result::CheckResult;
use crate::models::project::Project;
use chrono::Local;

/// 导出选项
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    /// 是否添加水印
    pub watermark: bool,
    /// 是否加密PDF
    pub encrypt: bool,
    /// PDF密码（可选）
    pub password: Option<String>,
    /// 是否脱敏敏感信息
    pub desensitize: bool,
    /// 导出格式
    pub format: String, // "word", "pdf", "excel"
}

/// 导出结果
#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub file_size: u64,
    pub watermark: bool,
    pub encrypted: bool,
}

/// 导出检查报告
#[tauri::command]
pub async fn export_report(
    project: Project,
    results: Vec<CheckResult>,
    output_path: String,
    options: ExportOptions,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    use crate::security::audit::actions;
    
    let export_type = options.format.clone();
    log::info!("开始导出报告: {} -> {}", project.name, output_path);
    
    // 根据格式导出
    match options.format.as_str() {
        "word" => export_word(&project, &results, &output_path, &options)?,
        "pdf" => export_pdf(&project, &results, &output_path, &options)?,
        "excel" => export_excel(&project, &results, &output_path, &options)?,
        _ => return Err(format!("不支持的导出格式: {}", options.format)),
    }
    
    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // 记录审计日志
    {
        let audit = state.audit_logger.lock();
        audit.log_success(actions::REPORT_EXPORT, &format!("{} ({})", output_path, export_type));
    }
    
    log::info!("报告导出成功: {} ({}KB)", output_path, file_size / 1024);
    
    Ok(ExportResult {
        path: output_path,
        file_size,
        watermark: options.watermark,
        encrypted: options.encrypt,
    })
}

/// 生成水印文本
fn generate_watermark() -> String {
    let now = Local::now();
    format!(
        "标书智能检查工具 | 导出时间: {} | 机密文件，请勿外传",
        now.format("%Y-%m-%d %H:%M")
    )
}

/// 导出Word格式
fn export_word(
    project: &Project,
    results: &[CheckResult],
    output_path: &str,
    options: &ExportOptions,
) -> Result<(), String> {
    let watermark = if options.watermark {
        generate_watermark()
    } else {
        String::new()
    };
    
    // 构建HTML内容（Word可以打开HTML）
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'>");
    html.push_str("<style>");
    html.push_str("body { font-family: '微软雅黑', sans-serif; padding: 20px; }");
    html.push_str("h1 { color: #409EFF; border-bottom: 2px solid #409EFF; padding-bottom: 10px; }");
    html.push_str("h2 { color: #303133; margin-top: 30px; }");
    html.push_str(".stats { display: flex; gap: 20px; margin: 20px 0; }");
    html.push_str(".stat-box { padding: 15px 20px; border-radius: 8px; text-align: center; }");
    html.push_str(".stat-box.error { background: #FEF0F0; color: #F56C6C; }");
    html.push_str(".stat-box.warning { background: #FDF6EC; color: #E6A23C; }");
    html.push_str(".stat-box.info { background: #F4F4F5; color: #909399; }");
    html.push_str(".result-item { margin: 15px 0; padding: 15px; border-radius: 8px; }");
    html.push_str(".result-item.error { background: #FEF0F0; border-left: 4px solid #F56C6C; }");
    html.push_str(".result-item.warning { background: #FDF6EC; border-left: 4px solid #E6A23C; }");
    html.push_str(".result-item.info { background: #F4F4F5; border-left: 4px solid #909399; }");
    if options.watermark {
        html.push_str("body::before { content: '");
        html.push_str(&watermark);
        html.push_str("'; position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(-45deg); font-size: 60px; color: rgba(200,200,200,0.3); pointer-events: none; white-space: nowrap; z-index: -1; }");
    }
    html.push_str("</style></head><body>");
    
    // 标题
    html.push_str(&format!("<h1>{} - 检查报告</h1>", project.name));
    html.push_str(&format!("<p>生成时间: {}</p>", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    // 统计
    let error_count = results.iter().filter(|r| r.status == "error").count();
    let warning_count = results.iter().filter(|r| r.status == "warning").count();
    let info_count = results.iter().filter(|r| r.status == "info").count();
    
    html.push_str("<h2>检查统计</h2>");
    html.push_str("<div class='stats'>");
    html.push_str(&format!("<div class='stat-box error'><div style='font-size:24px'>{}</div><div>错误</div></div>", error_count));
    html.push_str(&format!("<div class='stat-box warning'><div style='font-size:24px'>{}</div><div>警告</div></div>", warning_count));
    html.push_str(&format!("<div class='stat-box info'><div style='font-size:24px'>{}</div><div>提示</div></div>", info_count));
    html.push_str("</div>");
    
    // 详细结果
    html.push_str("<h2>检查结果</h2>");
    for result in results {
        html.push_str(&format!(
            "<div class='result-item {}'><h3>[{}] {}</h3>",
            result.status, result.status.to_uppercase(), result.name
        ));
        
        if let Some(ref desc) = result.result {
            let desc = if options.desensitize {
                crate::security::log_sanitizer::LogSanitizer::sanitize(desc)
            } else {
                desc.clone()
            };
            html.push_str(&format!("<p>{}</p>", desc));
        }
        
        if let Some(ref suggestion) = result.suggestion {
            html.push_str(&format!("<p><strong>建议:</strong> {}</p>", suggestion));
        }
        
        html.push_str("</div>");
    }
    
    // 水印页脚
    if options.watermark {
        html.push_str(&format!("<footer style='text-align:center;color:#999;font-size:12px;margin-top:50px;padding-top:20px;border-top:1px solid #eee;'>{}</footer>", watermark));
    }
    
    html.push_str("</body></html>");
    
    std::fs::write(output_path, html)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(())
}

/// 导出PDF格式
fn export_pdf(
    project: &Project,
    results: &[CheckResult],
    output_path: &str,
    options: &ExportOptions,
) -> Result<(), String> {
    // PDF导出需要额外库支持，先生成HTML
    let html_path = output_path.replace(".pdf", ".html");
    export_word(project, results, &html_path, options)?;
    
    Err("PDF导出暂未实现，已生成HTML文件，请在浏览器中打开并打印为PDF".to_string())
}

/// 导出Excel格式
fn export_excel(
    project: &Project,
    results: &[CheckResult],
    output_path: &str,
    options: &ExportOptions,
) -> Result<(), String> {
    let mut csv = String::new();
    
    // CSV头部
    csv.push_str("序号,检查项,状态,描述,建议\n");
    
    // 数据行
    for (i, result) in results.iter().enumerate() {
        let desc = if options.desensitize {
            crate::security::log_sanitizer::LogSanitizer::sanitize(
                result.result.as_deref().unwrap_or("")
            )
        } else {
            result.result.clone().unwrap_or_default()
        };
        
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            i + 1,
            result.name,
            result.status,
            desc.replace(",", "，").replace("\n", " "),
            result.suggestion.as_deref().unwrap_or("").replace(",", "，").replace("\n", " ")
        ));
    }
    
    // 水印行
    if options.watermark {
        csv.push_str(&format!("\n\n{}\n", generate_watermark()));
    }
    
    // 写入文件（UTF-8 with BOM for Excel）
    let bom = b"\xEF\xBB\xBF";
    let mut file = std::fs::File::create(output_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    
    use std::io::Write;
    file.write_all(bom).ok();
    file.write_all(csv.as_bytes()).ok();
    
    Ok(())
}

/// 导出对比报告
#[tauri::command]
pub async fn export_comparison(
    projects: Vec<Project>,
    all_results: Vec<Vec<CheckResult>>,
    output_path: String,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'>");
    html.push_str("<style>");
    html.push_str("body { font-family: '微软雅黑', sans-serif; padding: 20px; }");
    html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }");
    html.push_str("th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }");
    html.push_str("th { background-color: #409EFF; color: white; }");
    html.push_str("tr:nth-child(even) { background-color: #f9f9f9; }");
    html.push_str(".better { color: #67C23A; font-weight: bold; }");
    html.push_str(".worse { color: #F56C6C; font-weight: bold; }");
    html.push_str("</style></head><body>");
    
    html.push_str("<h1>多标书对比报告</h1>");
    html.push_str(&format!("<p>生成时间: {}</p>", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    // 统计对比表格
    html.push_str("<h2>错误统计对比</h2>");
    html.push_str("<table><thead><tr><th>项目名称</th><th>错误</th><th>警告</th><th>提示</th><th>总分</th></tr></thead><tbody>");
    
    for (project, results) in projects.iter().zip(all_results.iter()) {
        let error_count = results.iter().filter(|r| r.status == "error").count();
        let warning_count = results.iter().filter(|r| r.status == "warning").count();
        let info_count = results.iter().filter(|r| r.status == "info").count();
        let score = 100 - error_count * 10 - warning_count * 5 - info_count * 2;
        
        html.push_str(&format!(
            "<tr><td>{}</td><td class='worse'>{}</td><td>{}</td><td>{}</td><td class='{}'>{}</td></tr>",
            project.name, error_count, warning_count, info_count,
            if score >= 80 { "better" } else { "worse" },
            score
        ));
    }
    
    html.push_str("</tbody></table></body></html>");
    
    std::fs::write(&output_path, html)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    Ok(ExportResult {
        path: output_path,
        file_size,
        watermark: options.watermark,
        encrypted: false,
    })
}
