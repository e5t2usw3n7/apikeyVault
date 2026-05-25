use std::sync::Mutex;

use apikey_vault_core::config::AppConfig;
use apikey_vault_core::core::vault::Vault;

pub struct AppState {
    pub vault: Mutex<Vault>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            vault: Mutex::new(Vault::new(config)),
        }
    }
}
