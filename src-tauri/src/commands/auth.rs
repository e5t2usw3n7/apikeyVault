use serde::Serialize;
use tauri::State;

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

    let vault_state = vault.state_string();
    let is_initialized = vault.is_initialized();

    Ok(VaultStatus {
        state: vault_state,
        is_initialized,
    })
}

#[tauri::command]
pub async fn vault_init(
    state: State<'_, AppState>,
    password: String,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    vault.init(&password)?;
    Ok(())
}

#[tauri::command]
pub async fn vault_unlock(
    state: State<'_, AppState>,
    password: String,
) -> CommandResult<()> {
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

#[tauri::command]
pub async fn vault_change_password(
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    vault.change_password(&old_password, &new_password)?;
    Ok(())
}
