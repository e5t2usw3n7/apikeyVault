use serde::{Deserialize, Serialize};
use tauri::State;

use apikey_vault_core::core::key::{Environment, KeyType};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct KeyEntryDto {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub key_type: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub environment: String,
    pub group_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub usage_count: u64,
    pub version: u32,
}

#[derive(Serialize)]
pub struct ConnectivityResultDto {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

fn to_dto(entry: apikey_vault_core::core::key::KeyEntry) -> KeyEntryDto {
    KeyEntryDto {
        id: entry.id.to_string(),
        name: entry.name,
        provider: entry.provider,
        key_type: entry.key_type.to_string(),
        description: entry.description,
        tags: entry.tags,
        environment: entry.environment.to_string(),
        group_id: entry.group_id.map(|id| id.to_string()),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        expires_at: entry.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: entry.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        version: entry.version,
    }
}

fn parse_env(environment: &str) -> Environment {
    Environment::from_str(environment)
}

#[tauri::command]
pub async fn list_keys(state: State<'_, AppState>) -> CommandResult<Vec<KeyEntryDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let keys = vault.list_keys()?;
    Ok(keys.into_iter().map(to_dto).collect())
}

#[tauri::command]
pub async fn search_keys(state: State<'_, AppState>, query: String) -> CommandResult<Vec<KeyEntryDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let keys = vault.search_keys(&query)?;
    Ok(keys.into_iter().map(to_dto).collect())
}

#[tauri::command]
pub async fn get_key_value(state: State<'_, AppState>, name: String, environment: String) -> CommandResult<String> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let (_entry, value) = vault.get_key(&name, &environment)?;
    Ok(value)
}

#[tauri::command]
pub async fn add_key(
    state: State<'_, AppState>,
    name: String,
    provider: String,
    key_type: String,
    value: String,
    environment: String,
    description: Option<String>,
    group_id: Option<String>,
    tags: Vec<String>,
) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let kt = KeyType::from_str(&key_type);
    let env = Environment::from_str(&environment);
    let gid = group_id.and_then(|id| uuid::Uuid::parse_str(&id).ok());

    let entry = vault.add_key(name, provider, kt, &value, env, description, gid, tags)?;
    Ok(to_dto(entry))
}

#[tauri::command]
pub async fn update_key(
    state: State<'_, AppState>,
    name: String,
    environment: String,
    value: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let entry = vault.update_key(&name, &environment, value.as_deref(), description.as_deref(), tags)?;
    Ok(to_dto(entry))
}

#[tauri::command]
pub async fn delete_key(state: State<'_, AppState>, name: String, environment: String) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.delete_key(&name, &environment)?;
    Ok(())
}

#[tauri::command]
pub async fn rename_key(state: State<'_, AppState>, old_name: String, environment: String, new_name: String) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let entry = vault.rename_key(&old_name, &environment, &new_name)?;
    Ok(to_dto(entry))
}

#[tauri::command]
pub async fn rotate_key(state: State<'_, AppState>, name: String, environment: String, new_value: String) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let entry = vault.rotate_key(&name, &environment, &new_value)?;
    Ok(to_dto(entry))
}

#[tauri::command]
pub async fn test_connectivity(state: State<'_, AppState>, name: String, environment: String) -> CommandResult<ConnectivityResultDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;
    let result = vault.test_key_connectivity(&name, &environment)?;
    Ok(ConnectivityResultDto {
        success: result.success,
        message: result.message,
        latency_ms: result.latency_ms,
    })
}
