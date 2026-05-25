use std::path::Path;
use std::io::Write;
use crate::error::AppError;

/// 从 CSV 文件导入密钥
pub fn import_csv(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Import(format!("Failed to read CSV file: {}", e)))?;
    parse_csv_content(&content)
}

/// 从 CSV 字符串内容解析密钥（供 GUI 等非文件场景使用）
pub fn parse_csv_content(content: &str) -> Result<Vec<(String, String, String, String)>, AppError> {
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let mut keys = Vec::new();

    for result in rdr.records() {
        let record = result
            .map_err(|e| AppError::Import(format!("Failed to parse CSV record: {}", e)))?;
        if record.len() >= 4 {
            let name = record[0].to_string();
            let provider = record[1].to_string();
            let key_type = record[2].to_string();
            let value = record[3].to_string();
            if !name.is_empty() && !value.is_empty() {
                keys.push((name, provider, key_type, value));
            }
        }
    }

    Ok(keys)
}

/// 将密钥导出为 CSV 字符串
pub fn export_csv_to_string(keys: &[(String, String, String, String)]) -> Result<String, AppError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["name", "provider", "key_type", "value"])
        .map_err(|e| AppError::Export(format!("Failed to write CSV header: {}", e)))?;

    for (name, provider, key_type, value) in keys {
        wtr.write_record([name, provider, key_type, value])
            .map_err(|e| AppError::Export(format!("Failed to write CSV record: {}", e)))?;
    }

    wtr.flush()
        .map_err(|e| AppError::Export(format!("Failed to flush CSV writer: {}", e)))?;

    String::from_utf8(wtr.into_inner()
        .map_err(|e| AppError::Export(format!("Failed to get CSV output: {}", e)))?)
        .map_err(|e| AppError::Export(format!("CSV output is not valid UTF-8: {}", e)))
}

/// 将密钥导出到 CSV 文件
pub fn export_csv(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    let content = export_csv_to_string(keys)?;
    let mut file = std::fs::File::create(path)
        .map_err(|e| AppError::Export(format!("Failed to create CSV file: {}", e)))?;
    file.write_all(content.as_bytes())
        .map_err(|e| AppError::Export(format!("Failed to write CSV file: {}", e)))?;
    Ok(())
}
