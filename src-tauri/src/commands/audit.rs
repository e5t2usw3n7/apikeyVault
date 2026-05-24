use serde::Serialize;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct AuditEntryDto {
    pub id: i64,
    pub timestamp: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: serde_json::Value,
}

#[tauri::command]
pub async fn get_audit_logs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> CommandResult<Vec<AuditEntryDto>> {
    let mut vault = state.vault.lock().map_err(|e| CommandError::AppError(e.to_string()))?;

    let logs = vault.get_audit_logs(limit.unwrap_or(100))?;
    let dtos = logs.into_iter().map(|entry| AuditEntryDto {
        id: entry.id,
        timestamp: entry.timestamp.to_rfc3339(),
        action: format!("{:?}", entry.action),
        resource_type: entry.resource_type,
        resource_id: entry.resource_id,
        details: entry.details,
    }).collect();

    Ok(dtos)
}
