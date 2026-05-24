use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_keys(
    state: State<'_, AppState>,
    format: String,
    content: String,
    environment: String,
) -> CommandResult<ImportResult> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let result = vault.import_keys(&format, &content, env)?;

    Ok(ImportResult {
        imported: result.imported,
        skipped: result.skipped,
        errors: result.errors,
    })
}

#[tauri::command]
pub async fn export_keys(
    state: State<'_, AppState>,
    format: String,
    environment: Option<String>,
) -> CommandResult<String> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = environment.and_then(|e| {
        match e.as_str() {
            "Production" => Some(apikey_vault_core::core::key::Environment::Production),
            "Staging" => Some(apikey_vault_core::core::key::Environment::Staging),
            "Development" => Some(apikey_vault_core::core::key::Environment::Development),
            "Testing" => Some(apikey_vault_core::core::key::Environment::Testing),
            _ => Some(apikey_vault_core::core::key::Environment::Other(e)),
        }
    });

    let content = vault.export_keys(&format, env)?;
    Ok(content)
}
