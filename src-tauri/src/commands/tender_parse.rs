use serde::{Deserialize, Serialize};
use crate::services::llm_client::{chat, LlmConfig};
use crate::commands::file::parse_file;

/// 招标文件提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderExtraction {
    pub project_info: ProjectInfo,
    pub scoring_table: Vec<ScoringItem>,
    pub required_sections: Vec<RequiredSection>,
    pub qualification_requirements: Vec<QualificationReq>,
    pub time_requirements: TimeRequirements,
    pub format_requirements: FormatRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub project_name: String,
    pub project_no: String,
    pub tendering_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringItem {
    pub id: u32,
    pub category: String,
    pub item: String,
    pub score: u32,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredSection {
    pub name: String,
    pub required: bool,
    pub position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationReq {
    pub name: String,
    pub q_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRequirements {
    pub project_period: String,
    pub bid_validity: String,
    pub bid_bond: BidBond,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidBond {
    pub amount: String,
    pub deadline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequirements {
    pub body_font: FontSpec,
    pub title_font: FontSpec,
    pub page_margin: MarginSpec,
    pub line_spacing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSpec {
    pub name: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginSpec {
    pub top: String,
    pub bottom: String,
    pub left: String,
    pub right: String,
}

/// 解析招标文件
#[tauri::command]
pub async fn parse_tender_document(
    file_path: String,
    llm_config: LlmConfig,
) -> Result<TenderExtraction, String> {
    // 1. 解析文件内容
    let parse_result = parse_file(file_path).await?;
    
    // 2. 调用AI提取结构化信息
    let prompt = build_extraction_prompt(&parse_result.content);
    
    let response = chat(&llm_config, prompt).await
        .map_err(|e| format!("AI提取失败: {}", e))?;
    
    // 3. 解析AI返回的JSON
    let extraction: TenderExtraction = serde_json::from_str(&response)
        .map_err(|e| format!("解析AI响应失败: {}", e))?;
    
    Ok(extraction)
}

/// 构建提取提示词
fn build_extraction_prompt(content: &str) -> String {
    format!(
        r#"请从以下招标文件中提取结构化信息，以JSON格式返回。

需要提取的内容：
1. 项目信息（项目名称、项目编号、招标单位）
2. 评分表（评审项、分值、要求）
3. 必填章节（章节名称）
4. 资质要求（证书名称、类型）
5. 时间要求（工期、投标有效期、投标保证金）
6. 格式要求（字体、字号、页边距、行间距）

返回JSON格式如下：
{{
  "project_info": {{
    "project_name": "项目名称",
    "project_no": "项目编号",
    "tendering_unit": "招标单位"
  }},
  "scoring_table": [
    {{
      "id": 1,
      "category": "商务部分",
      "item": "评审项名称",
      "score": 5,
      "requirement": "具体要求"
    }}
  ],
  "required_sections": [
    {{
      "name": "章节名称",
      "required": true,
      "position": "位置说明"
    }}
  ],
  "qualification_requirements": [
    {{
      "name": "资质名称",
      "q_type": "business",
      "required": true
    }}
  ],
  "time_requirements": {{
    "project_period": "工期",
    "bid_validity": "投标有效期",
    "bid_bond": {{
      "amount": "保证金金额",
      "deadline": "截止时间"
    }}
  }},
  "format_requirements": {{
    "body_font": {{ "name": "字体名", "size": "字号" }},
    "title_font": {{ "name": "字体名", "size": "字号" }},
    "page_margin": {{ "top": "上", "bottom": "下", "left": "左", "right": "右" }},
    "line_spacing": "行间距"
  }}
}}

---

招标文件内容：

{}"#,
        content
    )
}
