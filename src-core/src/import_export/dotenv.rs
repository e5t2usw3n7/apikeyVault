use std::path::Path;
use std::io::Write;
use crate::error::AppError;

/// 从 .env 文件导入密钥
pub fn import_dotenv(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Import(format!("Failed to read .env file: {}", e)))?;
    parse_dotenv_content(&content)
}

/// 从 .env 格式字符串内容解析密钥（供 GUI 等非文件场景使用）
pub fn parse_dotenv_content(content: &str) -> Result<Vec<(String, String, String, String)>, AppError> {
    let mut keys = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() && !value.is_empty() {
                keys.push((key, String::new(), "ApiKey".to_string(), value));
            }
        }
    }

    Ok(keys)
}

/// 将密钥导出为 .env 格式字符串
pub fn export_dotenv_to_string(keys: &[(String, String, String, String)]) -> Result<String, AppError> {
    let mut output = String::new();
    for (name, _provider, _key_type, value) in keys {
        output.push_str(&format!("{}={}\n", name, value));
    }
    Ok(output)
}

/// 将密钥导出到 .env 文件
pub fn export_dotenv(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    let content = export_dotenv_to_string(keys)?;
    let mut file = std::fs::File::create(path)
        .map_err(|e| AppError::Export(format!("Failed to create .env file: {}", e)))?;
    file.write_all(content.as_bytes())
        .map_err(|e| AppError::Export(format!("Failed to write .env file: {}", e)))?;
    Ok(())
}
