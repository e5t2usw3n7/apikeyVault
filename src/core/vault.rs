use std::path::Path;
use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::core::key::{KeyEntry, KeyType, Environment};
use crate::core::group::Group;
use crate::core::audit::{AuditEntry, AuditAction};
use crate::crypto::encryption::EncryptionEngine;
use crate::crypto::kdf::KeyDeriver;
use crate::error::AppError;
use crate::storage::database::Database;
use crate::storage::repository::Repository;
use crate::validation;

/// Vault 状态
#[derive(Debug, Clone, PartialEq)]
pub enum VaultState {
    /// 未初始化
    Uninitialized,
    /// 已锁定
    Locked,
    /// 已解锁
    Unlocked,
}

/// 密钥过滤条件
#[derive(Debug, Default, Clone)]
pub struct KeyFilter {
    pub environment: Option<String>,
    pub group_id: Option<String>,
    pub tag: Option<String>,
}
/// 用于密码验证的魔术字符串
const VERIFIER_MAGIC: &[u8] = b"API_KEY_VAULT_VERIFIER_v1";

/// Vault 核心结构
pub struct Vault {
    config: AppConfig,
    db: Option<Database>,
    session_enabled: bool,       // 是否启用会话持久化（default: true）
    master_key: Option<Vec<u8>>,
    state: VaultState,
    last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl Vault {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            db: None,
            session_enabled: true,
            master_key: None,
            state: VaultState::Uninitialized,
            last_activity: None,
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> &VaultState {
        &self.state
    }

    // ==================== 会话持久化管理 ====================

    // /// 启用/禁用会话持久化
    // pub fn set_session_enabled(&mut self, enabled: bool) {
    //     self.session_enabled = enabled;
    // }

    /// 会话文件路径
    fn session_path(&self) -> std::path::PathBuf {
        self.config.vault_path.join(".session")
    }

    /// 保存会话：将 master_key 写入会话文件
    fn save_session(&self) -> Result<(), AppError> {
        if !self.session_enabled {
            return Ok(());
        }
        if let Some(ref key) = self.master_key {
            std::fs::write(self.session_path(), key)
                .map_err(|e| AppError::IoError(format!("无法保存会话: {}", e)))?;
        }
        Ok(())
    }

    /// 清除会话文件
    fn clear_session(&self) {
        if !self.session_enabled {
            return;
        }
        let path = self.session_path();
        let _ = std::fs::remove_file(&path);
    }

    /// 尝试从会话文件恢复解锁状态
    /// 返回 true 表示恢复成功，false 表示无可恢复会话
    pub fn try_restore_session(&mut self) -> Result<bool, AppError> {
        if self.state != VaultState::Uninitialized && self.state != VaultState::Locked {
            return Ok(false);
        }

        let session_file = self.session_path();
        if !session_file.exists() {
            return Ok(false);
        }

        match std::fs::read(&session_file) {
            Ok(master_key) => {
                match self.unlock_with_master_key(master_key) {
                    Ok(()) => Ok(true),
                    Err(_) => {
                        // 会话文件无效，清除
                        let _ = std::fs::remove_file(&session_file);
                        Ok(false)
                    }
                }
            }
            Err(_) => {
                // 无法读取会话文件，清除
                let _ = std::fs::remove_file(&session_file);
                Ok(false)
            }
        }
    }

    // ==================== 初始化/锁定/解锁 ====================

    /// 重置 Vault（删除所有数据文件）
    pub fn reset(&mut self) -> Result<(), AppError> {
        let db_path = self.config.vault_path.join("vault.db");
        let salt_path = self.config.vault_path.join(".salt");
        let verify_path = self.config.vault_path.join(".verify");
        let backup_dir = self.config.vault_path.join("backups");

        // 清除会话
        self.clear_session();

        if db_path.exists() {
            std::fs::remove_file(&db_path)
                .map_err(|e| AppError::IoError(format!("Failed to remove vault database: {}", e)))?;
        }
        if salt_path.exists() {
            std::fs::remove_file(&salt_path)
                .map_err(|e| AppError::IoError(format!("Failed to remove salt file: {}", e)))?;
        }
        if verify_path.exists() {
            std::fs::remove_file(&verify_path)
                .map_err(|e| AppError::IoError(format!("Failed to remove verifier file: {}", e)))?;
        }
        if backup_dir.exists() {
            std::fs::remove_dir_all(&backup_dir)
                .map_err(|e| AppError::IoError(format!("Failed to remove backup directory: {}", e)))?;
        }

        self.db = None;
        self.master_key = None;
        self.state = VaultState::Uninitialized;
        self.last_activity = None;

        Ok(())
    }

    /// 验证文件路径
    fn verify_path(&self) -> std::path::PathBuf {
        self.config.vault_path.join(".verify")
    }

    /// 存储密码验证令牌：用主密钥加密魔术字符串
    fn save_verifier(&self) -> Result<(), AppError> {
        let key = self.master_key.as_ref().ok_or(AppError::VaultLocked)?;
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key[..32]);
        let encryptor = EncryptionEngine::new(&key_bytes);
        let encrypted = encryptor.encrypt(VERIFIER_MAGIC)?;
        std::fs::write(self.verify_path(), &encrypted)
            .map_err(|e| AppError::IoError(format!("Failed to write verifier: {}", e)))?;
        Ok(())
    }

    /// 验证主密钥是否正确：尝试解密验证文件并比对魔术字符串
    fn verify_master_key(&self) -> Result<bool, AppError> {
        let verify_path = self.verify_path();
        if !verify_path.exists() {
            return Err(AppError::IoError("验证文件不存在，Vault 可能已损坏".to_string()));
        }
        let encrypted = std::fs::read(&verify_path)
            .map_err(|e| AppError::IoError(format!("Failed to read verifier: {}", e)))?;

        let key = self.master_key.as_ref().ok_or(AppError::VaultLocked)?;
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key[..32]);
        let encryptor = EncryptionEngine::new(&key_bytes);

        match encryptor.decrypt(&encrypted) {
            Ok(decrypted) => Ok(decrypted == VERIFIER_MAGIC),
            Err(_) => Ok(false),
        }
    }

    /// 初始化 Vault
    pub fn init(&mut self, password: &str) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.config.vault_path)
            .map_err(|e| AppError::IoError(format!("Failed to create vault directory: {}", e)))?;

        let db_path = self.config.vault_path.join("vault.db");

        if db_path.exists() {
            return Err(AppError::VaultAlreadyInitialized);
        }

        let db = Database::open(&db_path)?;

        // 派生主密钥
        let kdf = KeyDeriver::new();
        let salt = crate::crypto::kdf::generate_salt();
        let master_key = kdf.derive_key(password, &salt)?;

        // 存储 salt
        let salt_path = self.config.vault_path.join(".salt");
        std::fs::write(&salt_path, &salt)
            .map_err(|e| AppError::IoError(format!("Failed to write salt: {}", e)))?;

        self.db = Some(db);
        self.master_key = Some(master_key.clone());
        self.state = VaultState::Unlocked;
        self.last_activity = Some(Utc::now());

        // 存储密码验证令牌
        self.save_verifier()?;

        // 保存会话，后续命令无需重复输入密码
        self.save_session()?;

        self.log_action(AuditAction::VaultUnlocked, "vault", None, None)?;

        Ok(())
    }

    /// 检查 Vault 是否已初始化
    pub fn is_initialized(&self) -> bool {
        let db_path = self.config.vault_path.join("vault.db");
        db_path.exists()
    }

    /// 解锁 Vault
    pub fn unlock(&mut self, password: &str) -> Result<(), AppError> {
        let db_path = self.config.vault_path.join("vault.db");
        if !db_path.exists() {
            return Err(AppError::VaultNotInitialized);
        }

        let salt_path = self.config.vault_path.join(".salt");
        let salt = std::fs::read(&salt_path)
            .map_err(|e| AppError::IoError(format!("Failed to read salt: {}", e)))?;

        let kdf = KeyDeriver::new();
        let master_key = kdf.derive_key(password, &salt)?;

        // 暂存 master_key 用于验证
        self.master_key = Some(master_key);

        // 验证密码是否正确
        match self.verify_master_key() {
            Ok(true) => {
                // 密码正确，继续解锁
            }
            Ok(false) => {
                // 密码错误，清除 master_key
                self.master_key = None;
                return Err(AppError::InvalidInput("密码错误".to_string()));
            }
            Err(e) => {
                // 验证文件问题
                self.master_key = None;
                return Err(e);
            }
        }

        let db = Database::open(&db_path)?;

        self.db = Some(db);
        self.state = VaultState::Unlocked;
        self.last_activity = Some(Utc::now());

        // 保存会话，后续命令无需重复输入密码
        self.save_session()?;

        self.log_action(AuditAction::VaultUnlocked, "vault", None, None)?;

        Ok(())
    }

    /// 使用已有的 master key 解锁（用于从会话恢复）
    pub fn unlock_with_master_key(&mut self, master_key: Vec<u8>) -> Result<(), AppError> {
        let db_path = self.config.vault_path.join("vault.db");
        if !db_path.exists() {
            return Err(AppError::VaultNotInitialized);
        }

        // 暂存 master_key 用于验证
        self.master_key = Some(master_key);

        // 验证 master key 是否正确
        match self.verify_master_key() {
            Ok(true) => {
                // 验证通过
            }
            Ok(false) => {
                self.master_key = None;
                return Err(AppError::InvalidInput("会话密钥无效".to_string()));
            }
            Err(e) => {
                self.master_key = None;
                return Err(e);
            }
        }

        let db = Database::open(&db_path)?;
        self.db = Some(db);
        self.state = VaultState::Unlocked;
        self.last_activity = Some(Utc::now());

        Ok(())
    }

    /// 锁定 Vault
    pub fn lock(&mut self) {
        // 清除会话文件（锁定后不可恢复）
        self.clear_session();

        if let Some(ref mut key) = self.master_key {
            zeroize::Zeroize::zeroize(key);
        }
        self.master_key = None;
        self.db = None;
        self.state = VaultState::Locked;
    }

    /// 检查是否需要自动锁定
    pub fn check_auto_lock(&mut self) {
        if self.state != VaultState::Unlocked {
            return;
        }
        if let Some(last) = self.last_activity {
            let elapsed = Utc::now() - last;
            if elapsed.num_minutes() >= self.config.auto_lock_minutes as i64 {
                self.lock();
            }
        }
    }

    /// 获取仓库引用
    fn repo(&self) -> Result<Repository<'_>, AppError> {
        let db = self.db.as_ref().ok_or(AppError::VaultLocked)?;
        Ok(Repository::new(db))
    }

    /// 更新活动时间
    fn touch(&mut self) {
        self.last_activity = Some(Utc::now());
    }

    /// 获取加密引擎
    pub fn encryptor(&self) -> Result<EncryptionEngine, AppError> {
        let key = self.master_key.as_ref().ok_or(AppError::VaultLocked)?;
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key[..32]);
        Ok(EncryptionEngine::new(&key_bytes))
    }

    /// 记录审计日志
    fn log_action(&self, action: AuditAction, resource_type: &str, resource_id: Option<String>, details: Option<serde_json::Value>) -> Result<(), AppError> {
        if !self.config.audit_log_enabled {
            return Ok(());
        }
        if let Some(ref db) = self.db {
            let repo = Repository::new(db);
            let entry = AuditEntry::new(action, resource_type.to_string(), resource_id, details);
            repo.insert_audit_log(&entry)?;
        }
        Ok(())
    }

    // ==================== 密钥操作 ====================

    /// 添加密钥
    pub fn add_key(
        &mut self,
        name: String,
        provider: String,
        key_type: KeyType,
        value: &str,
        environment: Environment,
        description: Option<String>,
        group_id: Option<Uuid>,
        tags: Vec<String>,
    ) -> Result<KeyEntry, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        validation::validate_key_name(&name)?;

        let result = validation::validate_key_format(value, &key_type);
        for warning in &result.warnings {
            eprintln!("Warning: {}", warning);
        }

        let repo = self.repo()?;
        if repo.get_key_by_name(&name, &environment.to_string())?.is_some() {
            return Err(AppError::DuplicateKey(name));
        }

        let encryptor = self.encryptor()?;
        let encrypted_value = encryptor.encrypt(value.as_bytes())?;

        let mut entry = KeyEntry::new(name, provider, key_type, encrypted_value);
        entry.environment = environment;
        entry.description = description;
        entry.group_id = group_id;
        entry.tags = tags;

        let repo = self.repo()?;
        repo.insert_key(&entry)?;

        self.log_action(
            AuditAction::KeyCreated,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(entry)
    }

    /// 获取密钥（解密）
    pub fn get_key(&mut self, name: &str, environment: &str) -> Result<(KeyEntry, String), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let entry = repo.get_key_by_name(name, environment)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        let encryptor = self.encryptor()?;
        let decrypted = encryptor.decrypt(&entry.encrypted_value)?;
        let value = String::from_utf8(decrypted)
            .map_err(|_| AppError::InvalidInput("Invalid UTF-8 in key value".to_string()))?;

        self.log_action(
            AuditAction::KeyViewed,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok((entry, value))
    }

    /// 获取密钥（解密，不限制环境，自动搜索所有环境）
    pub fn get_key_any_env(&mut self, name: &str) -> Result<(KeyEntry, String), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let entry = repo.get_key_by_name_any_env(name)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        let encryptor = self.encryptor()?;
        let decrypted = encryptor.decrypt(&entry.encrypted_value)?;
        let value = String::from_utf8(decrypted)
            .map_err(|_| AppError::InvalidInput("Invalid UTF-8 in key value".to_string()))?;

        self.log_action(
            AuditAction::KeyViewed,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok((entry, value))
    }

    // ==================== 密钥过滤 ====================

    /// 列出所有密钥（带过滤）
    pub fn list_keys_filtered(&mut self, filter: &KeyFilter) -> Result<Vec<KeyEntry>, AppError> {
        let keys = self.list_keys()?;
        Ok(Self::apply_filter(keys, filter))
    }

    /// 搜索密钥（带过滤）
    pub fn search_keys_filtered(&mut self, query: &str, filter: &KeyFilter) -> Result<Vec<KeyEntry>, AppError> {
        let keys = self.search_keys(query)?;
        Ok(Self::apply_filter(keys, filter))
    }

    /// 对密钥列表应用过滤条件（内部辅助）
    fn apply_filter(keys: Vec<KeyEntry>, filter: &KeyFilter) -> Vec<KeyEntry> {
        keys.into_iter().filter(|key| {
            if let Some(ref env) = filter.environment {
                if key.environment.to_string() != *env {
                    return false;
                }
            }
            if let Some(ref gid) = filter.group_id {
                if let Some(key_gid) = key.group_id {
                    if key_gid.to_string() != *gid {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if let Some(ref tag) = filter.tag {
                if !key.tags.contains(tag) {
                    return false;
                }
            }
            true
        }).collect()
    }
    /// 列出所有密钥
    pub fn list_keys(&mut self) -> Result<Vec<KeyEntry>, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let keys = repo.list_keys()?;
        self.touch();
        Ok(keys)
    }

    /// 搜索密钥
    pub fn search_keys(&mut self, query: &str) -> Result<Vec<KeyEntry>, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let keys = repo.search_keys(query)?;
        self.touch();
        Ok(keys)
    }

    /// 更新密钥
    pub fn update_key(&mut self, name: &str, environment: &str, new_value: Option<&str>, new_description: Option<&str>, new_tags: Option<Vec<String>>) -> Result<KeyEntry, AppError> {
        self.update_key_full(name, environment, new_value, None, new_description, new_tags, None)
    }

    /// 更新密钥（不限制环境）
    pub fn update_key_any_env(&mut self, name: &str, new_value: Option<&str>, new_description: Option<&str>, new_tags: Option<Vec<String>>) -> Result<KeyEntry, AppError> {
        self.update_key_full_any_env(name, new_value, None, new_description, new_tags, None)
    }

    /// 更新密钥（完整版本，支持所有字段）
    pub fn update_key_full(
        &mut self,
        name: &str,
        environment: &str,
        new_value: Option<&str>,
        new_key_type: Option<KeyType>,
        new_description: Option<&str>,
        new_tags: Option<Vec<String>>,
        new_group_id: Option<Option<Uuid>>,
    ) -> Result<KeyEntry, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let mut entry = repo.get_key_by_name(name, environment)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        Self::apply_key_updates(&mut entry, &self.encryptor()?, new_value, new_key_type, new_description, new_tags, new_group_id)?;
        entry.updated_at = Utc::now();

        let repo = self.repo()?;
        repo.update_key(&entry)?;

        self.log_action(
            AuditAction::KeyUpdated,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(entry)
    }

    /// 更新密钥（完整版本，不限制环境）
    pub fn update_key_full_any_env(
        &mut self,
        name: &str,
        new_value: Option<&str>,
        new_key_type: Option<KeyType>,
        new_description: Option<&str>,
        new_tags: Option<Vec<String>>,
        new_group_id: Option<Option<Uuid>>,
    ) -> Result<KeyEntry, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let mut entry = repo.get_key_by_name_any_env(name)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        Self::apply_key_updates(&mut entry, &self.encryptor()?, new_value, new_key_type, new_description, new_tags, new_group_id)?;
        entry.updated_at = Utc::now();

        let repo = self.repo()?;
        repo.update_key(&entry)?;

        self.log_action(
            AuditAction::KeyUpdated,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(entry)
    }

    /// 应用密钥更新（内部辅助方法）
    fn apply_key_updates(
        entry: &mut KeyEntry,
        encryptor: &EncryptionEngine,
        new_value: Option<&str>,
        new_key_type: Option<KeyType>,
        new_description: Option<&str>,
        new_tags: Option<Vec<String>>,
        new_group_id: Option<Option<Uuid>>,
    ) -> Result<(), AppError> {
        if let Some(value) = new_value {
            entry.encrypted_value = encryptor.encrypt(value.as_bytes())?;
            entry.version += 1;
        }

        if let Some(kt) = new_key_type {
            entry.key_type = kt;
        }

        if let Some(desc) = new_description {
            entry.description = Some(desc.to_string());
        }

        if let Some(tags) = new_tags {
            entry.tags = tags;
        }

        if let Some(gid) = new_group_id {
            entry.group_id = gid;
        }

        Ok(())
    }

    /// 删除密钥
    pub fn delete_key(&mut self, name: &str, environment: &str) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let entry = repo.get_key_by_name(name, environment)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        repo.delete_key(&entry.id)?;

        self.log_action(
            AuditAction::KeyDeleted,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    /// 删除密钥（不限制环境）
    pub fn delete_key_any_env(&mut self, name: &str) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let entry = repo.get_key_by_name_any_env(name)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        repo.delete_key(&entry.id)?;

        self.log_action(
            AuditAction::KeyDeleted,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    /// 旋转密钥
    pub fn rotate_key(&mut self, name: &str, environment: &str, new_value: &str) -> Result<KeyEntry, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let mut entry = repo.get_key_by_name(name, environment)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        let encryptor = self.encryptor()?;
        entry.encrypted_value = encryptor.encrypt(new_value.as_bytes())?;
        entry.version += 1;
        entry.updated_at = Utc::now();

        let repo = self.repo()?;
        repo.update_key(&entry)?;

        self.log_action(
            AuditAction::KeyRotated,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(entry)
    }

    /// 旋转密钥（不限制环境）
    pub fn rotate_key_any_env(&mut self, name: &str, new_value: &str) -> Result<KeyEntry, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let mut entry = repo.get_key_by_name_any_env(name)?
            .ok_or_else(|| AppError::KeyNotFound(name.to_string()))?;

        let encryptor = self.encryptor()?;
        entry.encrypted_value = encryptor.encrypt(new_value.as_bytes())?;
        entry.version += 1;
        entry.updated_at = Utc::now();

        let repo = self.repo()?;
        repo.update_key(&entry)?;

        self.log_action(
            AuditAction::KeyRotated,
            "key",
            Some(entry.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(entry)
    }


    /// 获取即将过期的密钥
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_expiring_keys(&mut self, hours: i64) -> Result<Vec<KeyEntry>, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let keys = repo.get_expiring_keys(hours)?;
        self.touch();
        Ok(keys)
    }

    // ==================== 分组操作 ====================

    /// 创建分组
    pub fn create_group(&mut self, name: String) -> Result<Group, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let group = Group::new(name);
        let repo = self.repo()?;
        repo.insert_group(&group)?;

        self.log_action(
            AuditAction::GroupCreated,
            "group",
            Some(group.id.to_string()),
            None,
        )?;

        self.touch();
        Ok(group)
    }

    /// 列出所有分组
    pub fn list_groups(&mut self) -> Result<Vec<Group>, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let groups = repo.list_groups()?;
        self.touch();
        Ok(groups)
    }

    /// 重命名分组
    pub fn rename_group(&mut self, id: &Uuid, new_name: String) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        repo.rename_group(id, &new_name)?;

        self.log_action(
            AuditAction::GroupUpdated,
            "group",
            Some(id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    /// 更新分组（名称和描述）
    pub fn update_group(&mut self, id: &Uuid, new_name: &str, new_description: Option<String>) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        repo.update_group(id, new_name, new_description.as_deref())?;

        self.log_action(
            AuditAction::GroupUpdated,
            "group",
            Some(id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    /// 更新分组描述
    pub fn update_group_description(&mut self, id: &Uuid, new_description: Option<String>) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        repo.update_group_description(id, new_description.as_deref())?;

        self.log_action(
            AuditAction::GroupUpdated,
            "group",
            Some(id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    /// 删除分组
    pub fn delete_group(&mut self, id: &Uuid) -> Result<(), AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        repo.delete_group(id)?;

        self.log_action(
            AuditAction::GroupDeleted,
            "group",
            Some(id.to_string()),
            None,
        )?;

        self.touch();
        Ok(())
    }

    // ==================== 审计日志 ====================

    /// 获取审计日志
    pub fn get_audit_logs(&mut self, limit: i64) -> Result<Vec<AuditEntry>, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let repo = self.repo()?;
        let logs = repo.list_audit_logs(limit)?;
        self.touch();
        Ok(logs)
    }

    // ==================== 导入导出 ====================

    /// 导入密钥
    pub fn import_keys(&mut self, records: Vec<(String, String, String, String)>, environment: Environment) -> Result<usize, AppError> {
        self.check_auto_lock();
        if self.state != VaultState::Unlocked {
            return Err(AppError::VaultLocked);
        }

        let encryptor = self.encryptor()?;
        let mut imported = 0;

        for (name, provider, key_type_str, value) in records {
            if value.is_empty() {
                continue;
            }

            let key_type = KeyType::from_str(&key_type_str);

            let encrypted_value = encryptor.encrypt(value.as_bytes())?;
            let entry = KeyEntry::new(name, provider, key_type, encrypted_value);
            let entry = KeyEntry {
                environment: environment.clone(),
                ..entry
            };

            let repo = self.repo()?;
            if repo.get_key_by_name(&entry.name, &entry.environment.to_string())?.is_some() {
                continue;
            }
            repo.insert_key(&entry)?;
            imported += 1;
        }

        self.log_action(
            AuditAction::KeyImported,
            "key",
            None,
            Some(serde_json::json!({ "count": imported })),
        )?;

        self.touch();
        Ok(imported)
    }

    // ==================== 备份恢复 ====================

    /// 创建备份
    pub fn backup(&self, backup_path: &Path) -> Result<(), AppError> {
        let db_path = self.config.vault_path.join("vault.db");
        if !db_path.exists() {
            return Err(AppError::VaultNotInitialized);
        }

        std::fs::copy(&db_path, backup_path)
            .map_err(|e| AppError::IoError(format!("Failed to create backup: {}", e)))?;

        self.log_action(
            AuditAction::BackupCreated,
            "backup",
            Some(backup_path.to_string_lossy().to_string()),
            None,
        )?;

        Ok(())
    }

    /// 从备份恢复
    pub fn restore(&mut self, backup_path: &Path) -> Result<(), AppError> {
        let db_path = self.config.vault_path.join("vault.db");

        self.lock();

        std::fs::copy(backup_path, &db_path)
            .map_err(|e| AppError::IoError(format!("Failed to restore backup: {}", e)))?;

        self.state = VaultState::Locked;

        self.log_action(
            AuditAction::BackupRestored,
            "backup",
            Some(backup_path.to_string_lossy().to_string()),
            None,
        )?;

        Ok(())
    }

    // ==================== 配置 ====================

    /// 获取配置引用
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取配置可变引用
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    // /// 获取 master key 的引用（用于保存会话）
    // pub fn get_master_key(&self) -> Option<&Vec<u8>> {
    //     self.master_key.as_ref()
    // }
}
