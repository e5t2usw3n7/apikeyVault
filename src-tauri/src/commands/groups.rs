use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct GroupDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn list_groups(state: State<'_, AppState>) -> CommandResult<Vec<GroupDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let groups = vault.list_groups()?;
    let dtos = groups.into_iter().map(|g| GroupDto {
        id: g.id.to_string(),
        name: g.name,
        description: g.description,
        created_at: g.created_at.to_rfc3339(),
        updated_at: g.updated_at.to_rfc3339(),
    }).collect();

    Ok(dtos)
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<GroupDto> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let group = vault.create_group(&name)?;

    Ok(GroupDto {
        id: group.id.to_string(),
        name: group.name,
        description: group.description,
        created_at: group.created_at.to_rfc3339(),
        updated_at: group.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let group_id = uuid::Uuid::parse_str(&id).map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.update_group(group_id, name.as_deref(), description.as_deref())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_group(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let group_id = uuid::Uuid::parse_str(&id).map_err(|e| CommandError::AppError(e.to_string()))?;
    vault.delete_group(group_id)?;

    Ok(())
}
