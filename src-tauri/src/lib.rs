mod commands;
pub mod error;
pub mod state;

use apikey_vault_core::config::AppConfig;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![
            commands::auth::vault_status,
            commands::auth::vault_init,
            commands::auth::vault_unlock,
            commands::auth::vault_lock,
            commands::auth::vault_try_restore_session,
            commands::keys::list_keys,
            commands::keys::search_keys,
            commands::keys::get_key_value,
            commands::keys::add_key,
            commands::keys::update_key,
            commands::keys::delete_key,
            commands::keys::rename_key,
            commands::keys::rotate_key,
            commands::keys::test_connectivity,
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::update_group,
            commands::groups::delete_group,
            commands::audit::get_audit_logs,
            commands::import_export::import_keys,
            commands::import_export::export_keys,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::backup_vault,
            commands::config::restore_vault,
            commands::config::reset_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
