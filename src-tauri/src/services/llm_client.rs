use reqwest::Client;
use serde::{Deserialize, Serialize};

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub secret_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

/// Chat请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

/// 消息
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Chat响应
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

/// 选择项
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// 调用LLM API
pub async fn chat(config: &LlmConfig, prompt: String) -> Result<String, String> {
    let client = Client::new();
    
    let (base_url, auth_header) = match config.provider.as_str() {
        "baidu" => {
            // 百度千帆API
            let url = format!(
                "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{}?access_token={}",
                get_baidu_model_path(&config.model),
                config.api_key
            );
            (url, None)
        },
        "aliyun" => {
            // 阿里通义API
            let url = config.base_url.clone()
                .unwrap_or_else(|| "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string());
            (url, Some(format!("Bearer {}", config.api_key)))
        },
        "openai" => {
            // OpenAI API
            let url = config.base_url.clone()
                .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
            (url, Some(format!("Bearer {}", config.api_key)))
        },
        _ => {
            // 自定义API
            let url = config.base_url.clone()
                .ok_or_else(|| "自定义API需要提供base_url".to_string())?;
            (url, Some(format!("Bearer {}", config.api_key)))
        }
    };
    
    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "你是一个专业的标书审核专家，具有丰富的招投标经验。".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        temperature: 0.7,
        max_tokens: 4000,
    };
    
    let mut req = client.post(&base_url)
        .header("Content-Type", "application/json");
    
    if let Some(auth) = auth_header {
        req = req.header("Authorization", auth);
    }
    
    let response = req.json(&request)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API请求失败: {} - {}", status, body));
    }
    
    let chat_response: ChatResponse = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "无响应内容".to_string())
}

/// 获取百度模型路径
fn get_baidu_model_path(model: &str) -> String {
    match model {
        "ERNIE-4.0" => "completions_pro".to_string(),
        "ERNIE-3.5" => "completions".to_string(),
        _ => "completions".to_string(),
    }
}
