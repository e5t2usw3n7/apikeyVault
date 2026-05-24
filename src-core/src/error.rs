use thiserror::Error;

/// Type alias for backwards compatibility
pub type ApiKeyError = AppError;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    #[error("Duplicate key name: {0}")]
    DuplicateKeyName(String),

    #[error("Vault not initialized")]
    VaultNotInitialized,

    #[error("Vault already initialized")]
    VaultAlreadyInitialized,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Operation cancelled")]
    Cancelled,
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    #[error("Key derivation failed")]
    KdfError,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("HMAC verification failed")]
    HmacVerificationFailed,
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not initialized")]
    NotInitialized,
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}