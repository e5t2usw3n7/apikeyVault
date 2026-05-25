use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct AppConfigDto {
    pub vault_path: String,
    pub auto_lock_minutes: u32,
    pub clipboard_clear_seconds: u32,
    pub theme: String,
    pub default_environment: String,
    pub audit_log_enabled: bool,
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> CommandResult<AppConfigDto> {
    let vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let config = vault.config();

    Ok(AppConfigDto {
        vault_path: config.vault_path.to_string_lossy().to_string(),
        auto_lock_minutes: config.auto_lock_minutes,
        clipboard_clear_seconds: config.clipboard_clear_seconds,
        theme: config.theme.clone(),
        default_environment: config.default_environment.clone(),
        audit_log_enabled: config.audit_log_enabled,
    })
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    auto_lock_minutes: Option<u32>,
    clipboard_clear_seconds: Option<u32>,
    theme: Option<String>,
    default_environment: Option<String>,
    audit_log_enabled: Option<bool>,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let config = vault.config_mut();

    if let Some(minutes) = auto_lock_minutes {
        config.auto_lock_minutes = minutes;
    }
    if let Some(seconds) = clipboard_clear_seconds {
        config.clipboard_clear_seconds = seconds;
    }
    if let Some(t) = theme {
        config.theme = t;
    }
    if let Some(env) = default_environment {
        config.default_environment = env;
    }
    if let Some(enabled) = audit_log_enabled {
        config.audit_log_enabled = enabled;
    }

    Ok(())
}

#[tauri::command]
pub async fn backup_vault(state: State<'_, AppState>, file_path: String) -> CommandResult<()> {
    let vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.backup(Path::new(&file_path))?;
    Ok(())
}

#[tauri::command]
pub async fn restore_vault(state: State<'_, AppState>, file_path: String) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.restore(Path::new(&file_path))?;
    Ok(())
}

#[tauri::command]
pub async fn reset_vault(state: State<'_, AppState>) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.reset()?;
    Ok(())
}
