use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("{0}")]
    AppError(String),

    #[error("Vault 未初始化")]
    VaultNotInitialized,

    #[error("Vault 已锁定")]
    VaultLocked,

    #[error("密码错误")]
    WrongPassword,

    #[error("密钥未找到: {0}")]
    KeyNotFound(String),

    #[error("分组未找到: {0}")]
    GroupNotFound(String),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("加密错误: {0}")]
    CryptoError(String),

    #[error("IO 错误: {0}")]
    IoError(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let message = self.to_string();
        serializer.serialize_str(&message)
    }
}

impl From<apikey_vault_core::error::AppError> for CommandError {
    fn from(err: apikey_vault_core::error::AppError) -> Self {
        CommandError::AppError(err.to_string())
    }
}

impl From<apikey_vault_core::error::StorageError> for CommandError {
    fn from(err: apikey_vault_core::error::StorageError) -> Self {
        CommandError::StorageError(err.to_string())
    }
}

impl From<apikey_vault_core::error::CryptoError> for CommandError {
    fn from(err: apikey_vault_core::error::CryptoError) -> Self {
        CommandError::CryptoError(err.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError::SerializationError(err.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
