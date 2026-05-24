use serde::{Deserialize, Serialize};
use tauri::State;

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

#[derive(Serialize, Deserialize)]
pub struct KeyFilterDto {
    pub environment: Option<String>,
    pub provider: Option<String>,
    pub key_type: Option<String>,
    pub group_id: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Serialize)]
pub struct ConnectivityResultDto {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

#[tauri::command]
pub async fn list_keys(state: State<'_, AppState>) -> CommandResult<Vec<KeyEntryDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let keys = vault.list_keys()?;
    let dtos = keys.into_iter().map(|k| KeyEntryDto {
        id: k.id.to_string(),
        name: k.name,
        provider: k.provider,
        key_type: format!("{:?}", k.key_type),
        description: k.description,
        tags: k.tags,
        environment: format!("{:?}", k.environment),
        group_id: k.group_id.map(|id| id.to_string()),
        created_at: k.created_at.to_rfc3339(),
        updated_at: k.updated_at.to_rfc3339(),
        expires_at: k.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: k.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: k.usage_count,
        version: k.version,
    }).collect();

    Ok(dtos)
}

#[tauri::command]
pub async fn search_keys(
    state: State<'_, AppState>,
    query: String,
) -> CommandResult<Vec<KeyEntryDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let keys = vault.search_keys(&query)?;
    let dtos = keys.into_iter().map(|k| KeyEntryDto {
        id: k.id.to_string(),
        name: k.name,
        provider: k.provider,
        key_type: format!("{:?}", k.key_type),
        description: k.description,
        tags: k.tags,
        environment: format!("{:?}", k.environment),
        group_id: k.group_id.map(|id| id.to_string()),
        created_at: k.created_at.to_rfc3339(),
        updated_at: k.updated_at.to_rfc3339(),
        expires_at: k.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: k.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: k.usage_count,
        version: k.version,
    }).collect();

    Ok(dtos)
}

#[tauri::command]
pub async fn get_key_value(
    state: State<'_, AppState>,
    name: String,
    environment: String,
) -> CommandResult<String> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let value = vault.get_key_value(&name, env)?;
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

    let kt = match key_type.as_str() {
        "ApiKey" => apikey_vault_core::core::key::KeyType::ApiKey,
        "BearerToken" => apikey_vault_core::core::key::KeyType::BearerToken,
        "BasicAuth" => apikey_vault_core::core::key::KeyType::BasicAuth,
        "OAuth2" => apikey_vault_core::core::key::KeyType::OAuth2,
        "Jwt" => apikey_vault_core::core::key::KeyType::Jwt,
        "SshKey" => apikey_vault_core::core::key::KeyType::SshKey,
        _ => apikey_vault_core::core::key::KeyType::Other(key_type.clone()),
    };

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let gid = group_id.and_then(|id| {
        uuid::Uuid::parse_str(&id).ok()
    });

    let entry = vault.add_key(
        &name,
        &provider,
        kt,
        &value,
        env,
        description.as_deref(),
        gid,
        tags,
    )?;

    Ok(KeyEntryDto {
        id: entry.id.to_string(),
        name: entry.name,
        provider: entry.provider,
        key_type: format!("{:?}", entry.key_type),
        description: entry.description,
        tags: entry.tags,
        environment: format!("{:?}", entry.environment),
        group_id: entry.group_id.map(|id| id.to_string()),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        expires_at: entry.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: entry.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        version: entry.version,
    })
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

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let entry = vault.update_key(&name, env, value.as_deref(), description.as_deref(), tags)?;

    Ok(KeyEntryDto {
        id: entry.id.to_string(),
        name: entry.name,
        provider: entry.provider,
        key_type: format!("{:?}", entry.key_type),
        description: entry.description,
        tags: entry.tags,
        environment: format!("{:?}", entry.environment),
        group_id: entry.group_id.map(|id| id.to_string()),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        expires_at: entry.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: entry.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        version: entry.version,
    })
}

#[tauri::command]
pub async fn delete_key(
    state: State<'_, AppState>,
    name: String,
    environment: String,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    vault.delete_key(&name, env)?;
    Ok(())
}

#[tauri::command]
pub async fn rename_key(
    state: State<'_, AppState>,
    old_name: String,
    environment: String,
    new_name: String,
) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let entry = vault.rename_key(&old_name, env, &new_name)?;

    Ok(KeyEntryDto {
        id: entry.id.to_string(),
        name: entry.name,
        provider: entry.provider,
        key_type: format!("{:?}", entry.key_type),
        description: entry.description,
        tags: entry.tags,
        environment: format!("{:?}", entry.environment),
        group_id: entry.group_id.map(|id| id.to_string()),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        expires_at: entry.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: entry.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        version: entry.version,
    })
}

#[tauri::command]
pub async fn rotate_key(
    state: State<'_, AppState>,
    name: String,
    environment: String,
    new_value: String,
) -> CommandResult<KeyEntryDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let entry = vault.rotate_key(&name, env, &new_value)?;

    Ok(KeyEntryDto {
        id: entry.id.to_string(),
        name: entry.name,
        provider: entry.provider,
        key_type: format!("{:?}", entry.key_type),
        description: entry.description,
        tags: entry.tags,
        environment: format!("{:?}", entry.environment),
        group_id: entry.group_id.map(|id| id.to_string()),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        expires_at: entry.expires_at.map(|t| t.to_rfc3339()),
        last_used_at: entry.last_used_at.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        version: entry.version,
    })
}

#[tauri::command]
pub async fn test_connectivity(
    state: State<'_, AppState>,
    name: String,
    environment: String,
) -> CommandResult<ConnectivityResultDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let env = match environment.as_str() {
        "Production" => apikey_vault_core::core::key::Environment::Production,
        "Staging" => apikey_vault_core::core::key::Environment::Staging,
        "Development" => apikey_vault_core::core::key::Environment::Development,
        "Testing" => apikey_vault_core::core::key::Environment::Testing,
        _ => apikey_vault_core::core::key::Environment::Other(environment.clone()),
    };

    let result = vault.test_key_connectivity(&name, env)?;

    Ok(ConnectivityResultDto {
        success: result.success,
        message: result.message,
        latency_ms: result.latency_ms,
    })
}
