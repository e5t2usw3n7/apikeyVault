use serde::Serialize;
use tauri::State;

use apikey_vault_core::core::vault::VaultState;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct VaultStatus {
    pub state: String,
    pub is_initialized: bool,
}

#[tauri::command]
pub async fn vault_status(state: State<'_, AppState>) -> CommandResult<VaultStatus> {
    let vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let state_str = match vault.state() {
        VaultState::Uninitialized => "uninitialized",
        VaultState::Locked => "locked",
        VaultState::Unlocked => "unlocked",
    };

    Ok(VaultStatus {
        state: state_str.to_string(),
        is_initialized: vault.is_initialized(),
    })
}

#[tauri::command]
pub async fn vault_init(state: State<'_, AppState>, password: String) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.init(&password)?;
    Ok(())
}

#[tauri::command]
pub async fn vault_unlock(state: State<'_, AppState>, password: String) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.unlock(&password)?;
    Ok(())
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.lock();
    Ok(())
}

#[tauri::command]
pub async fn vault_try_restore_session(state: State<'_, AppState>) -> CommandResult<bool> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let restored = vault.try_restore_session()?;
    Ok(restored)
}
