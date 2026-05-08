use std::path::Path;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Serialize, Deserialize)]
struct JsonKeyRecord {
    name: String,
    provider: String,
    key_type: String,
    value: Option<String>,
    environment: Option<String>,
    tags: Option<Vec<String>>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct JsonExport {
    version: String,
    keys: Vec<JsonKeyRecord>,
}

pub fn import_json(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Import(format!("Failed to read JSON file: {}", e)))?;

    let records: Vec<JsonKeyRecord> = serde_json::from_str(&content)
        .map_err(|e| AppError::Import(format!("Invalid JSON format: {}", e)))?;

    let keys = records.into_iter().map(|r| {
        (r.name, r.provider, r.key_type, r.value.unwrap_or_default())
    }).collect();

    Ok(keys)
}

pub fn export_json(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    let records: Vec<JsonKeyRecord> = keys.iter().map(|(name, provider, key_type, value)| {
        JsonKeyRecord {
            name: name.clone(),
            provider: provider.clone(),
            key_type: key_type.clone(),
            value: Some(value.clone()),
            environment: None,
            tags: None,
            description: None,
        }
    }).collect();

    let export = JsonExport {
        version: "1.0".to_string(),
        keys: records,
    };

    let content = serde_json::to_string_pretty(&export)
        .map_err(|e| AppError::Export(format!("Failed to serialize JSON: {}", e)))?;

    let mut file = std::fs::File::create(path)
        .map_err(|e| AppError::Export(format!("Failed to create JSON file: {}", e)))?;

    file.write_all(content.as_bytes())
        .map_err(|e| AppError::Export(format!("Failed to write JSON file: {}", e)))?;

    Ok(())
}