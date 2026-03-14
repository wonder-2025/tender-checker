use rusqlite::Connection;
use tauri::{AppHandle, Manager};

/// 初始化数据库
pub fn init_database(_app: &AppHandle) -> Result<(), String> {
    // 创建数据库连接
    let db_path = get_db_path(_app)?;
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("数据库连接失败: {}", e))?;
    
    // 创建表
    create_tables(&conn)?;
    
    Ok(())
}

/// 获取数据库路径
fn get_db_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取应用目录失败: {}", e))?;
    
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("创建应用目录失败: {}", e))?;
    
    Ok(app_dir.join("tender_checker.db"))
}

/// 创建数据表
fn create_tables(conn: &Connection) -> Result<(), String> {
    // 项目表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            tender_file_path TEXT,
            bid_file_path TEXT,
            status TEXT DEFAULT 'pending',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // 招标文件提取结果
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tender_extractions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            project_name TEXT,
            project_no TEXT,
            tendering_unit TEXT,
            scoring_table TEXT,
            required_sections TEXT,
            qualification_requirements TEXT,
            time_requirements TEXT,
            format_requirements TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // 检查结果
    conn.execute(
        "CREATE TABLE IF NOT EXISTS check_results (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            category TEXT NOT NULL,
            item TEXT NOT NULL,
            severity TEXT NOT NULL,
            status TEXT NOT NULL,
            result TEXT,
            error TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // API配置
    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_configs (
            id TEXT PRIMARY KEY,
            provider TEXT NOT NULL,
            api_key TEXT NOT NULL,
            secret_key TEXT,
            model TEXT NOT NULL,
            base_url TEXT,
            is_default INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // 脱敏规则
    conn.execute(
        "CREATE TABLE IF NOT EXISTS desensitize_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            pattern TEXT NOT NULL,
            replacement TEXT NOT NULL,
            enabled INTEGER DEFAULT 1,
            priority INTEGER DEFAULT 0,
            is_custom INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // 自定义检查规则
    conn.execute(
        "CREATE TABLE IF NOT EXISTS custom_check_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            prompt_template TEXT NOT NULL,
            severity TEXT DEFAULT 'warning',
            enabled INTEGER DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
