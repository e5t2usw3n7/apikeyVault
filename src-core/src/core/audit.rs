use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 审计日志操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    Login,
    Logout,
    KeyCreated,
    KeyUpdated,
    KeyDeleted,
    KeyViewed,
    KeyCopied,
    KeyRotated,
    KeyImported,
    KeyExported,
    GroupCreated,
    GroupUpdated,
    GroupDeleted,
    PasswordChanged,
    BackupCreated,
    BackupRestored,
    VaultLocked,
    VaultUnlocked,
    KeyTested,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Login => write!(f, "LOGIN"),
            AuditAction::Logout => write!(f, "LOGOUT"),
            AuditAction::KeyCreated => write!(f, "KEY_CREATED"),
            AuditAction::KeyUpdated => write!(f, "KEY_UPDATED"),
            AuditAction::KeyDeleted => write!(f, "KEY_DELETED"),
            AuditAction::KeyViewed => write!(f, "KEY_VIEWED"),
            AuditAction::KeyCopied => write!(f, "KEY_COPIED"),
            AuditAction::KeyRotated => write!(f, "KEY_ROTATED"),
            AuditAction::KeyImported => write!(f, "KEY_IMPORTED"),
            AuditAction::KeyExported => write!(f, "KEY_EXPORTED"),
            AuditAction::GroupCreated => write!(f, "GROUP_CREATED"),
            AuditAction::GroupUpdated => write!(f, "GROUP_UPDATED"),
            AuditAction::GroupDeleted => write!(f, "GROUP_DELETED"),
            AuditAction::PasswordChanged => write!(f, "PASSWORD_CHANGED"),
            AuditAction::BackupCreated => write!(f, "BACKUP_CREATED"),
            AuditAction::BackupRestored => write!(f, "BACKUP_RESTORED"),
            AuditAction::VaultLocked => write!(f, "VAULT_LOCKED"),
            AuditAction::VaultUnlocked => write!(f, "VAULT_UNLOCKED"),
            AuditAction::KeyTested => write!(f, "KEY_TESTED"),
        }
    }
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl AuditEntry {
    pub fn new(
        action: AuditAction,
        resource_type: String,
        resource_id: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: 0, // 由数据库自动生成
            timestamp: Utc::now(),
            action,
            resource_type,
            resource_id,
            details,
        }
    }
}