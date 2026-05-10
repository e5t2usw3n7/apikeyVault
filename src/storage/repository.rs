use rusqlite::{params, OptionalExtension};
use uuid::Uuid;
use crate::core::key::{KeyEntry, KeyType, Environment};
use crate::core::group::Group;
use crate::core::audit::{AuditEntry, AuditAction};
use crate::storage::database::Database;
use crate::error::AppError;

/// 数据仓库，封装所有数据库操作
pub struct Repository<'a> {
    db: &'a Database,
}

impl<'a> Repository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    // ==================== 密钥操作 ====================

    /// 插入新密钥
    pub fn insert_key(&self, key: &KeyEntry) -> Result<(), AppError> {
        self.db.conn().execute(
            "INSERT INTO keys (id, name, provider, key_type, encrypted_value, description, tags, environment, group_id, created_at, updated_at, expires_at, last_used_at, usage_count, metadata, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                key.id.to_string(),
                key.name,
                key.provider,
                serde_json::to_string(&key.key_type).unwrap(),
                key.encrypted_value,
                key.description,
                serde_json::to_string(&key.tags).unwrap(),
                key.environment.to_string(),
                key.group_id.map(|id| id.to_string()),
                key.created_at.to_rfc3339(),
                key.updated_at.to_rfc3339(),
                key.expires_at.map(|dt| dt.to_rfc3339()),
                key.last_used_at.map(|dt| dt.to_rfc3339()),
                key.usage_count as i64,
                key.metadata.to_string(),
                key.version as i64,
            ],
        )?;
        Ok(())
    }

    /// 更新密钥
    pub fn update_key(&self, key: &KeyEntry) -> Result<(), AppError> {
        let affected = self.db.conn().execute(
            "UPDATE keys SET name=?1, provider=?2, key_type=?3, encrypted_value=?4, description=?5, tags=?6, environment=?7, group_id=?8, updated_at=?9, expires_at=?10, usage_count=?11, metadata=?12, version=?13 WHERE id=?14",
            params![
                key.name,
                key.provider,
                serde_json::to_string(&key.key_type).unwrap(),
                key.encrypted_value,
                key.description,
                serde_json::to_string(&key.tags).unwrap(),
                key.environment.to_string(),
                key.group_id.map(|id| id.to_string()),
                key.updated_at.to_rfc3339(),
                key.expires_at.map(|dt| dt.to_rfc3339()),
                key.usage_count as i64,
                key.metadata.to_string(),
                key.version as i64,
                key.id.to_string(),
            ],
        )?;
        if affected == 0 {
            return Err(AppError::KeyNotFound(key.id.to_string()));
        }
        Ok(())
    }

    /// 删除密钥
    pub fn delete_key(&self, id: &Uuid) -> Result<(), AppError> {
        let affected = self.db.conn().execute(
            "DELETE FROM keys WHERE id=?1",
            params![id.to_string()],
        )?;
        if affected == 0 {
            return Err(AppError::KeyNotFound(id.to_string()));
        }
        Ok(())
    }

    /// 根据 ID 查询密钥
    pub fn get_key_by_id(&self, id: &Uuid) -> Result<Option<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE id=?1")?;

        let result = stmt
            .query_row(params![id.to_string()], |row| self.row_to_key(row))
            .optional()?;

        Ok(result)
    }

    /// 根据名称查询密钥
    pub fn get_key_by_name(&self, name: &str, environment: &str) -> Result<Option<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE name=?1 AND environment=?2")?;

        let result = stmt
            .query_row(params![name, environment], |row| self.row_to_key(row))
            .optional()?;

        Ok(result)
    }

    /// 根据名称查询密钥（不限制环境，返回第一个匹配）
    pub fn get_key_by_name_any_env(&self, name: &str) -> Result<Option<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE name=?1 ORDER BY created_at DESC LIMIT 1")?;

        let result = stmt
            .query_row(params![name], |row| self.row_to_key(row))
            .optional()?;

        Ok(result)
    }

    /// 列出所有密钥
    pub fn list_keys(&self) -> Result<Vec<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys ORDER BY created_at DESC")?;

        let rows = stmt
            .query_map([], |row| self.row_to_key(row))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// 按环境筛选密钥
    #[allow(dead_code)]
    pub fn list_keys_by_environment(&self, environment: &str) -> Result<Vec<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE environment=?1 ORDER BY created_at DESC")?;

        let rows = stmt
            .query_map(params![environment], |row| self.row_to_key(row))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// 按分组查询密钥
    #[allow(dead_code)]
    pub fn list_keys_by_group(&self, group_id: &Uuid) -> Result<Vec<KeyEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE group_id=?1 ORDER BY created_at DESC")?;

        let rows = stmt
            .query_map(params![group_id.to_string()], |row| self.row_to_key(row))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// 搜索密钥
    pub fn search_keys(&self, query: &str) -> Result<Vec<KeyEntry>, AppError> {
        let pattern = format!("%{}%", query);
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE name LIKE ?1 OR provider LIKE ?1 OR description LIKE ?1 ORDER BY name")?;

        let rows = stmt
            .query_map(params![pattern], |row| self.row_to_key(row))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// 获取即将过期的密钥
    #[allow(dead_code)]
    pub fn get_expiring_keys(&self, within_hours: i64) -> Result<Vec<KeyEntry>, AppError> {
        let threshold = chrono::Utc::now() + chrono::Duration::hours(within_hours);
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM keys WHERE expires_at IS NOT NULL AND expires_at <= ?1")?;

        let rows = stmt
            .query_map(params![threshold.to_rfc3339()], |row| self.row_to_key(row))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// 将数据库行转换为 KeyEntry
    fn row_to_key(&self, row: &rusqlite::Row) -> rusqlite::Result<KeyEntry> {
        let id_str: String = row.get(0)?;
        let key_type_str: String = row.get(3)?;
        let tags_str: String = row.get(6)?;
        let environment_str: String = row.get(7)?;
        let group_id_str: Option<String> = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let updated_at_str: String = row.get(10)?;
        let expires_at_str: Option<String> = row.get(11)?;
        let last_used_at_str: Option<String> = row.get(12)?;
        let metadata_str: String = row.get(14)?;

        Ok(KeyEntry {
            id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            name: row.get(1)?,
            provider: row.get(2)?,
            key_type: serde_json::from_str(&key_type_str).unwrap_or(KeyType::Other(key_type_str)),
            encrypted_value: row.get(4)?,
            description: row.get(5)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            environment: Environment::from_str(&environment_str),
            group_id: group_id_str.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            expires_at: expires_at_str.as_deref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            last_used_at: last_used_at_str.as_deref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            usage_count: row.get::<_, i64>(13)? as u64,
            metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null),
            version: row.get::<_, i64>(15)? as u32,
        })
    }

    // ==================== 分组操作 ====================

    /// 插入分组
    pub fn insert_group(&self, group: &Group) -> Result<(), AppError> {
        self.db.conn().execute(
            "INSERT INTO groups (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                group.id.to_string(),
                group.name,
                group.description,
                group.created_at.to_rfc3339(),
                group.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 查询所有分组
    pub fn list_groups(&self) -> Result<Vec<Group>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM groups ORDER BY name")?;

        let rows = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let created_at_str: String = row.get(3)?;
                let updated_at_str: String = row.get(4)?;

                Ok(Group {
                    id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                })
            })?;

        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }

    /// 重命名分组
    pub fn rename_group(&self, id: &Uuid, new_name: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.db.conn().execute(
            "UPDATE groups SET name=?1, updated_at=?2 WHERE id=?3",
            params![new_name, now, id.to_string()],
        )?;
        if affected == 0 {
            return Err(AppError::GroupNotFound(id.to_string()));
        }
        Ok(())
    }

    /// 删除分组
    pub fn delete_group(&self, id: &Uuid) -> Result<(), AppError> {
        let affected = self.db.conn().execute(
            "DELETE FROM groups WHERE id=?1",
            params![id.to_string()],
        )?;
        if affected == 0 {
            return Err(AppError::GroupNotFound(id.to_string()));
        }
        Ok(())
    }

    // ==================== 审计日志操作 ====================

    /// 插入审计日志
    pub fn insert_audit_log(&self, entry: &AuditEntry) -> Result<(), AppError> {
        self.db.conn().execute(
            "INSERT INTO audit_log (timestamp, action, resource_type, resource_id, details) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.action.to_string(),
                entry.resource_type,
                entry.resource_id,
                entry.details.as_ref().map(|d| d.to_string()),
            ],
        )?;
        Ok(())
    }

    /// 查询审计日志
    pub fn list_audit_logs(&self, limit: i64) -> Result<Vec<AuditEntry>, AppError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare("SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ?1")?;

        let rows = stmt
            .query_map(params![limit], |row| {
                let timestamp_str: String = row.get(1)?;
                let action_str: String = row.get(2)?;
                let details_str: Option<String> = row.get(5)?;

                Ok(AuditEntry {
                    id: row.get(0)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    action: match action_str.as_str() {
                        "LOGIN" => AuditAction::Login,
                        "LOGOUT" => AuditAction::Logout,
                        "KEY_CREATED" => AuditAction::KeyCreated,
                        "KEY_UPDATED" => AuditAction::KeyUpdated,
                        "KEY_DELETED" => AuditAction::KeyDeleted,
                        "KEY_VIEWED" => AuditAction::KeyViewed,
                        "KEY_COPIED" => AuditAction::KeyCopied,
                        "KEY_ROTATED" => AuditAction::KeyRotated,
                        "KEY_IMPORTED" => AuditAction::KeyImported,
                        "KEY_EXPORTED" => AuditAction::KeyExported,
                        "GROUP_CREATED" => AuditAction::GroupCreated,
                        "GROUP_UPDATED" => AuditAction::GroupUpdated,
                        "GROUP_DELETED" => AuditAction::GroupDeleted,
                        "PASSWORD_CHANGED" => AuditAction::PasswordChanged,
                        "BACKUP_CREATED" => AuditAction::BackupCreated,
                        "BACKUP_RESTORED" => AuditAction::BackupRestored,
                        "VAULT_LOCKED" => AuditAction::VaultLocked,
                        "VAULT_UNLOCKED" => AuditAction::VaultUnlocked,
                        _ => AuditAction::Login,
                    },
                    resource_type: row.get(3)?,
                    resource_id: row.get(4)?,
                    details: details_str.and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row?);
        }
        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_key() {
        let db = Database::open_in_memory().unwrap();
        let repo = Repository::new(&db);

        let key = KeyEntry::new(
            "test-key".to_string(),
            "OpenAI".to_string(),
            KeyType::ApiKey,
            vec![1, 2, 3],
        );
        let id = key.id;

        repo.insert_key(&key).unwrap();
        let retrieved = repo.get_key_by_id(&id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "test-key");
        assert_eq!(retrieved.provider, "OpenAI");
    }

    #[test]
    fn test_insert_and_list_groups() {
        let db = Database::open_in_memory().unwrap();
        let repo = Repository::new(&db);

        let group = Group::new("test-group".to_string());
        repo.insert_group(&group).unwrap();

        let groups = repo.list_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "test-group");
    }

    #[test]
    fn test_search_keys() {
        let db = Database::open_in_memory().unwrap();
        let repo = Repository::new(&db);

        let key1 = KeyEntry::new("openai-key".to_string(), "OpenAI".to_string(), KeyType::ApiKey, vec![1]);
        let key2 = KeyEntry::new("aws-key".to_string(), "AWS".to_string(), KeyType::ApiKey, vec![2]);

        repo.insert_key(&key1).unwrap();
        repo.insert_key(&key2).unwrap();

        let results = repo.search_keys("openai").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "openai-key");
    }

    #[test]
    fn test_audit_log() {
        let db = Database::open_in_memory().unwrap();
        let repo = Repository::new(&db);

        let entry = AuditEntry::new(
            AuditAction::KeyCreated,
            "key".to_string(),
            Some("test-id".to_string()),
            None,
        );
        repo.insert_audit_log(&entry).unwrap();

        let logs = repo.list_audit_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
    }
}