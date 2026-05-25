use serde::Serialize;
use tauri::State;

use apikey_vault_core::core::key::Environment;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: usize,
}

#[tauri::command]
pub async fn import_keys(
    state: State<'_, AppState>,
    format: String,
    content: String,
    environment: String,
) -> CommandResult<ImportResult> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let env = Environment::from_str(&environment);

    // Parse content based on format into Vec<(name, provider, key_type, value)>
    let records = parse_import_content(&format, &content)?;

    let imported = vault.import_keys(records, env)?;
    Ok(ImportResult { imported })
}

fn parse_import_content(format: &str, content: &str) -> CommandResult<Vec<(String, String, String, String)>> {
    match format {
        "csv" => {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let mut records = Vec::new();
            for result in rdr.records() {
                let record = result.map_err(|e| CommandError::AppError(e.to_string()))?;
                if record.len() >= 4 {
                    records.push((
                        record[0].to_string(),
                        record[1].to_string(),
                        record[2].to_string(),
                        record[3].to_string(),
                    ));
                }
            }
            Ok(records)
        }
        "json" => {
            let parsed: Vec<serde_json::Value> = serde_json::from_str(content)
                .map_err(|e| CommandError::AppError(e.to_string()))?;
            let mut records = Vec::new();
            for item in parsed {
                let name = item["name"].as_str().unwrap_or("").to_string();
                let provider = item["provider"].as_str().unwrap_or("").to_string();
                let key_type = item["key_type"].as_str().unwrap_or("api_key").to_string();
                let value = item["value"].as_str().unwrap_or("").to_string();
                if !name.is_empty() && !value.is_empty() {
                    records.push((name, provider, key_type, value));
                }
            }
            Ok(records)
        }
        "dotenv" | "env" => {
            let mut records = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    if !key.is_empty() && !value.is_empty() {
                        records.push((key.clone(), "unknown".to_string(), "api_key".to_string(), value));
                    }
                }
            }
            Ok(records)
        }
        _ => Err(CommandError::AppError(format!("不支持的格式: {}", format))),
    }
}

#[tauri::command]
pub async fn export_keys(
    state: State<'_, AppState>,
    format: String,
    environment: Option<String>,
) -> CommandResult<String> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let keys = vault.list_keys()?;

    let filtered: Vec<_> = if let Some(env_str) = environment {
        let env = Environment::from_str(&env_str);
        keys.into_iter().filter(|k| k.environment == env).collect()
    } else {
        keys
    };

    match format.as_str() {
        "csv" => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            wtr.write_record(&["name", "provider", "key_type", "environment", "description", "tags"])
                .map_err(|e| CommandError::AppError(e.to_string()))?;
            for key in &filtered {
                wtr.write_record(&[
                    &key.name,
                    &key.provider,
                    &key.key_type.to_string(),
                    &key.environment.to_string(),
                    key.description.as_deref().unwrap_or(""),
                    &key.tags.join(","),
                ]).map_err(|e| CommandError::AppError(e.to_string()))?;
            }
            let data = wtr.into_inner().map_err(|e| CommandError::AppError(e.to_string()))?;
            String::from_utf8(data).map_err(|e| CommandError::AppError(e.to_string()))
        }
        "json" => {
            serde_json::to_string_pretty(&filtered).map_err(|e| CommandError::AppError(e.to_string()))
        }
        "dotenv" | "env" => {
            let lines: Vec<String> = filtered.iter().map(|k| {
                format!("{}={}", k.name.to_uppercase(), k.description.as_deref().unwrap_or(""))
            }).collect();
            Ok(lines.join("\n"))
        }
        _ => Err(CommandError::AppError(format!("不支持的格式: {}", format))),
    }
}
