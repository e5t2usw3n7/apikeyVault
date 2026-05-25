use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub vault_path: PathBuf,
    pub auto_lock_minutes: u32,
    pub clipboard_clear_seconds: u32,
    pub theme: String,
    pub default_environment: String,
    pub audit_log_enabled: bool,
    pub max_history: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        let vault_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("apikey-vault");

        Self {
            vault_path,
            auto_lock_minutes: 15,
            clipboard_clear_seconds: 30,
            theme: "dark".to_string(),
            default_environment: "development".to_string(),
            audit_log_enabled: true,
            max_history: 100,
        }
    }
}

impl AppConfig {
    /// 配置文件路径
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("apikey-vault")
            .join("config.toml")
    }

    /// 加载配置
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save().ok();
            config
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}