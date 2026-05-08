use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 密钥类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    ApiKey,
    OAuthToken,
    SshKey,
    Certificate,
    JwtToken,
    Password,
    Other(String),
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::ApiKey => write!(f, "API Key"),
            KeyType::OAuthToken => write!(f, "OAuth Token"),
            KeyType::SshKey => write!(f, "SSH Key"),
            KeyType::Certificate => write!(f, "Certificate"),
            KeyType::JwtToken => write!(f, "JWT Token"),
            KeyType::Password => write!(f, "Password"),
            KeyType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl KeyType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "api_key" | "api-key" | "apikey" => KeyType::ApiKey,
            "oauth_token" | "oauth-token" | "oauthtoken" | "oauth" => KeyType::OAuthToken,
            "ssh_key" | "ssh-key" | "sshkey" | "ssh" => KeyType::SshKey,
            "certificate" | "cert" => KeyType::Certificate,
            "jwt_token" | "jwt-token" | "jwttoken" | "jwt" => KeyType::JwtToken,
            "password" | "pwd" => KeyType::Password,
            other => KeyType::Other(other.to_string()),
        }
    }
}

/// 环境标识
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Environment {
    Production,
    Staging,
    Development,
    Testing,
    Custom(String),
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Production => write!(f, "production"),
            Environment::Staging => write!(f, "staging"),
            Environment::Development => write!(f, "development"),
            Environment::Testing => write!(f, "testing"),
            Environment::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stg" => Environment::Staging,
            "development" | "dev" => Environment::Development,
            "testing" | "test" => Environment::Testing,
            other => Environment::Custom(other.to_string()),
        }
    }
}

/// 密钥条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub key_type: KeyType,
    pub encrypted_value: Vec<u8>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub environment: Environment,
    pub group_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub metadata: serde_json::Value,
    pub version: u32,
}

impl KeyEntry {
    pub fn new(
        name: String,
        provider: String,
        key_type: KeyType,
        encrypted_value: Vec<u8>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name,
            provider,
            key_type,
            encrypted_value,
            description: None,
            tags: Vec::new(),
            environment: Environment::Development,
            group_id: None,
            created_at: now,
            updated_at: now,
            expires_at: None,
            last_used_at: None,
            usage_count: 0,
            metadata: serde_json::Value::Null,
            version: 1,
        }
    }

    #[cfg(test)]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expires| Utc::now() > expires)
            .unwrap_or(false)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_entry_new() {
        let entry = KeyEntry::new(
            "test-key".to_string(),
            "OpenAI".to_string(),
            KeyType::ApiKey,
            vec![1, 2, 3],
        );
        assert_eq!(entry.name, "test-key");
        assert_eq!(entry.provider, "OpenAI");
        assert_eq!(entry.key_type, KeyType::ApiKey);
        assert!(!entry.is_expired());
        assert_eq!(entry.usage_count, 0);
    }

    #[test]
    fn test_key_entry_expired() {
        let mut entry = KeyEntry::new(
            "test-key".to_string(),
            "OpenAI".to_string(),
            KeyType::ApiKey,
            vec![1, 2, 3],
        );
        assert!(!entry.is_expired());

        entry.expires_at = Some(Utc::now() - chrono::Duration::days(1));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(format!("{}", KeyType::ApiKey), "API Key");
        assert_eq!(format!("{}", KeyType::OAuthToken), "OAuth Token");
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(format!("{}", Environment::Production), "production");
        assert_eq!(format!("{}", Environment::Development), "development");
    }
}