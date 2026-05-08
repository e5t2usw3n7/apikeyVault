use std::path::Path;
use std::io::{BufRead, BufReader, Write};
use crate::error::AppError;

pub fn import_csv(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    let file = std::fs::File::open(path)
        .map_err(|e| AppError::Import(format!("Failed to open CSV file: {}", e)))?;
    let reader = BufReader::new(file);
    let mut keys = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| AppError::Import(format!("Failed to read line: {}", e)))?;
        if i == 0 && line.to_lowercase().starts_with("name,") {
            continue; // Skip header
        }
        let parts: Vec<&str> = line.splitn(4, ',').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            let provider = if parts.len() > 1 { parts[1].trim().to_string() } else { "Unknown".to_string() };
            let key_type = if parts.len() > 2 { parts[2].trim().to_string() } else { "ApiKey".to_string() };
            let value = if parts.len() > 3 { parts[3].trim().to_string() } else { String::new() };
            keys.push((name, provider, key_type, value));
        }
    }
    Ok(keys)
}

pub fn export_csv(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| AppError::Export(format!("Failed to create CSV file: {}", e)))?;

    writeln!(file, "name,provider,key_type,value")
        .map_err(|e| AppError::Export(format!("Failed to write header: {}", e)))?;

    for (name, provider, key_type, value) in keys {
        writeln!(file, "{},{},{},{}",
            escape_csv(name),
            escape_csv(provider),
            escape_csv(key_type),
            escape_csv(value),
        ).map_err(|e| AppError::Export(format!("Failed to write record: {}", e)))?;
    }
    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}