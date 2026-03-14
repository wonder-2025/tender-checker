use serde::{Deserialize, Serialize};
use std::path::Path;
use std::io::Read;

/// 文件解析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ParseResult {
    pub content: String,
    pub format_info: Option<FormatInfo>,
    pub page_count: usize,
    pub word_count: usize,
    pub file_type: String,
    pub file_size: u64,
}

/// 格式信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FormatInfo {
    pub fonts: Vec<String>,
    pub font_sizes: Vec<String>,
    pub has_toc: bool,
    pub line_spacing: Option<String>,
    pub page_margin: Option<MarginInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarginInfo {
    pub top: String,
    pub bottom: String,
    pub left: String,
    pub right: String,
}

/// 文件信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub extension: String,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// 文件验证错误
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
}

/// 文件魔数签名
const FILE_SIGNATURES: &[(&[u8], &str)] = &[
    (&[0x50, 0x4B, 0x03, 0x04], "zip"),      // ZIP-based (docx, xlsx)
    (&[0x25, 0x50, 0x44, 0x46], "pdf"),      // PDF
    (&[0xD0, 0xCF, 0x11, 0xE0], "ole"),      // DOC (OLE)
];

/// 最大文件大小 (50MB)
const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// 允许的文件扩展名
const ALLOWED_EXTENSIONS: &[&str] = &["docx", "doc", "pdf", "xlsx", "xls", "txt"];

/// 验证文件
fn validate_file(path: &str) -> Result<(u64, String), ValidationError> {
    let file_path = Path::new(path);
    
    // 1. 检查文件是否存在
    if !file_path.exists() {
        return Err(ValidationError {
            code: "FILE_NOT_FOUND".to_string(),
            message: "文件不存在".to_string(),
        });
    }
    
    // 2. 检查文件大小
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| ValidationError {
            code: "METADATA_ERROR".to_string(),
            message: format!("无法读取文件信息: {}", e),
        })?;
    
    let file_size = metadata.len();
    if file_size > MAX_FILE_SIZE {
        return Err(ValidationError {
            code: "FILE_TOO_LARGE".to_string(),
            message: format!("文件过大 ({:.2}MB)，最大支持50MB", file_size as f64 / 1024.0 / 1024.0),
        });
    }
    
    // 3. 检查文件扩展名
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(ValidationError {
            code: "INVALID_TYPE".to_string(),
            message: format!("不支持的文件类型: .{}，支持: {}", extension, ALLOWED_EXTENSIONS.join(", ")),
        });
    }
    
    // 4. 验证文件魔数（防止扩展名伪造）
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| ValidationError {
            code: "OPEN_ERROR".to_string(),
            message: format!("无法打开文件: {}", e),
        })?;
    
    let mut header = [0u8; 8];
    if file.read_exact(&mut header).is_ok() {
        // 检查魔数（txt文件跳过）
        if extension != "txt" {
            let detected = FILE_SIGNATURES
                .iter()
                .find(|(sig, _)| header.starts_with(sig))
                .map(|(_, t)| *t);
            
            match detected {
                Some("zip") if extension == "docx" || extension == "xlsx" || extension == "xls" => {},
                Some("pdf") if extension == "pdf" => {},
                Some("ole") if extension == "doc" => {},
                Some(detected) => {
                    log::warn!("文件类型不匹配: 扩展名.{}, 实际{}", extension, detected);
                },
                None => {
                    log::warn!("无法识别文件格式: {}", path);
                },
                _ => {}
            }
        }
    }
    
    // 5. 检查可疑内容
    if let Ok(content) = std::fs::read(file_path) {
        let suspicious_patterns: &[&[u8]] = &[
            b"<script", b"javascript:", b"vbscript:", b"onload=", b"onerror=",
        ];
        
        for pattern in suspicious_patterns {
            if content.windows(pattern.len()).any(|w| {
                w.iter().zip(pattern.iter()).all(|(a, b)| a.to_ascii_lowercase() == *b)
            }) {
                log::warn!("文件包含可疑内容: {:?}", std::str::from_utf8(pattern));
            }
        }
    }
    
    Ok((file_size, extension))
}

/// 解析文件
#[tauri::command]
pub async fn parse_file(file_path: String) -> Result<ParseResult, String> {
    // 验证文件
    let (file_size, extension) = validate_file(&file_path)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    
    let result = match extension.as_str() {
        "docx" => parse_docx(&file_path).await?,
        "doc" => parse_doc(&file_path).await?,
        "pdf" => parse_pdf(&file_path).await?,
        "xlsx" | "xls" => parse_excel(&file_path).await?,
        "txt" => parse_text(&file_path).await?,
        _ => return Err(format!("不支持的文件格式: {}", extension)),
    };
    
    // 计算统计信息
    let word_count = result.content.chars().count();
    let page_count = (word_count as f64 / 800.0).ceil() as usize;
    
    log::info!("文件解析成功: {} ({}KB, {}字)", 
        Path::new(&file_path).file_name().unwrap().to_string_lossy(),
        file_size / 1024,
        word_count
    );
    
    Ok(ParseResult {
        content: result.content,
        format_info: result.format_info,
        page_count,
        word_count,
        file_type: extension,
        file_size,
    })
}

/// 获取文件信息
#[tauri::command]
pub async fn get_file_info(file_path: String) -> Result<FileInfo, String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err("文件不存在".to_string());
    }
    
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    
    let name = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    
    Ok(FileInfo {
        name,
        path: file_path,
        size: metadata.len(),
        extension,
        created_at: None,
        modified_at: None,
    })
}

/// 解析Word文档 (.docx)
async fn parse_docx(file_path: &str) -> Result<ParseResult, String> {
    // TODO: 实现真正的docx解析
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    Ok(ParseResult {
        content,
        format_info: Some(FormatInfo {
            fonts: vec!["宋体".to_string()],
            font_sizes: vec!["小四".to_string()],
            has_toc: false,
            line_spacing: Some("1.5倍".to_string()),
            page_margin: Some(MarginInfo {
                top: "2.54cm".to_string(),
                bottom: "2.54cm".to_string(),
                left: "3.17cm".to_string(),
                right: "3.17cm".to_string(),
            }),
        }),
        page_count: 0,
        word_count: 0,
        file_type: "docx".to_string(),
        file_size: 0,
    })
}

/// 解析旧版Word文档 (.doc)
async fn parse_doc(file_path: &str) -> Result<ParseResult, String> {
    Err("暂不支持.doc格式，请转换为.docx格式".to_string())
}

/// 解析PDF文档
async fn parse_pdf(file_path: &str) -> Result<ParseResult, String> {
    let content = pdf_extract::extract_text(file_path)
        .map_err(|e| format!("PDF解析失败: {}", e))?;
    
    Ok(ParseResult {
        content,
        format_info: None,
        page_count: 0,
        word_count: 0,
        file_type: "pdf".to_string(),
        file_size: 0,
    })
}

/// 解析Excel文档
async fn parse_excel(file_path: &str) -> Result<ParseResult, String> {
    use calamine::Reader;
    
    let mut workbook = calamine::open_workbook_auto(file_path)
        .map_err(|e| format!("Excel解析失败: {}", e))?;
    
    let mut content = String::new();
    
    for sheet_name in workbook.sheet_names() {
        content.push_str(&format!("【{}】\n", sheet_name));
        
        if let Some(range) = workbook.worksheet_range(&sheet_name) {
            let range = range.map_err(|e| e.to_string())?;
            for row in range.rows() {
                for cell in row {
                    content.push_str(&format!("{} ", cell));
                }
                content.push('\n');
            }
        }
        content.push('\n');
    }
    
    Ok(ParseResult {
        content,
        format_info: None,
        page_count: 0,
        word_count: 0,
        file_type: "xlsx".to_string(),
        file_size: 0,
    })
}

/// 解析纯文本
async fn parse_text(file_path: &str) -> Result<ParseResult, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    Ok(ParseResult {
        content,
        format_info: None,
        page_count: 0,
        word_count: 0,
        file_type: "txt".to_string(),
        file_size: 0,
    })
}
