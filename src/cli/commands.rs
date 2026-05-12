use std::path::Path;
use uuid::Uuid;
use dialoguer::Password;

use crate::cli::{Cli, Commands, GroupAction, KeyAction, ImportFormat};
use crate::config::AppConfig;
use crate::core::key::{KeyType, Environment};
use crate::core::vault::Vault;
use crate::error::AppError;
use crate::import_export;
use crate::core::vault::KeyFilter;

/// 创建已恢复会话的 Vault 实例
fn create_vault_with_session(config: AppConfig) -> Result<Vault, AppError> {
    let mut vault = Vault::new(config);
    let _ = vault.try_restore_session()?;
    Ok(vault)
}

/// 创建新的 Vault 实例（不恢复会话）
fn create_vault(config: AppConfig) -> Result<Vault, AppError> {
    let vault = Vault::new(config);
    Ok(vault)
}

/// 执行 CLI 命令
pub fn execute(cli: Cli) -> Result<(), AppError> {
    let config = AppConfig::load();

    match cli.command {
        Commands::Init { force } => {
            let mut vault = create_vault(config)?;
            if vault.is_initialized() && !force {
                println!("Vault 已经初始化。使用 --force 强制重新初始化，或使用 'unlock' 命令解锁。");
                return Ok(());
            }

            if vault.is_initialized() && force {
                vault.reset()?;
            }

            let password = Password::new().with_prompt("设置主密码").interact()
                .map_err(|e| AppError::IoError(e.to_string()))?;
            let confirm = Password::new().with_prompt("确认主密码").interact()
                .map_err(|e| AppError::IoError(e.to_string()))?;

            if password != confirm {
                println!("❌ 密码不匹配");
                return Ok(());
            }

            vault.init(&password)?;
            println!("✅ Vault 已初始化");
        }

        Commands::Unlock => {
            let mut vault = create_vault(config)?;
            if !vault.is_initialized() {
                println!("❌ Vault 未初始化。请先运行 'init' 命令。");
                return Ok(());
            }

            let password = Password::new().with_prompt("输入主密码").interact()
                .map_err(|e| AppError::IoError(e.to_string()))?;
            vault.unlock(&password)?;
            println!("✅ Vault 已解锁（会话已保存，后续命令无需重复输入密码）");
        }

        Commands::Lock => {
            let mut vault = create_vault(config)?;
            vault.lock();
            println!("🔒 Vault 已锁定");
        }

        Commands::Status => {
            let mut vault = create_vault_with_session(config.clone())?;

            println!("Vault 状态:");
            println!("  路径: {}", config.vault_path.display());
            println!("  状态: {}", match vault.state() {
                crate::core::vault::VaultState::Uninitialized => "未初始化",
                crate::core::vault::VaultState::Locked => "已锁定",
                crate::core::vault::VaultState::Unlocked => "已解锁",
            });

            if *vault.state() == crate::core::vault::VaultState::Unlocked {
                let keys = vault.list_keys()?;
                let groups = vault.list_groups()?;
                println!("  密钥数量: {}", keys.len());
                println!("  分组数量: {}", groups.len());
                println!("  审计日志: {}", config.audit_log_enabled);
                println!("  自动锁定: {} 分钟", config.auto_lock_minutes);
            }
        }

        // ==================== 密钥管理 ====================
        Commands::Key { action } => {
            execute_key_action(action, config)?;
        }

        // ==================== 分组管理 ====================
        Commands::Group { action } => {
            execute_group_action(action, config)?;
        }

        // ==================== 搜索 ====================
        Commands::Search { query, group, tag } => {
            let mut vault = create_vault_with_session(config)?;
            let filter = KeyFilter {
                environment: None,
                group_id: group,
                tag,
            };
            let filtered = vault.search_keys_filtered(&query, &filter)?;

            if filtered.is_empty() {
                println!("未找到匹配的密钥");
                return Ok(());
            }

            println!("搜索结果:");
            println!("{:<30} {:<15} {:<12}", "名称", "提供商", "环境");
            println!("{}", "-".repeat(60));
            for key in &filtered {
                println!("{:<30} {:<15} {:<12}", key.name, key.provider, key.environment);
            }
            println!("\n共 {} 个结果", filtered.len());
        }

        // ==================== 导入 ====================
        Commands::Import { format, file, environment, skip_existing: _ } => {
            let mut vault = create_vault_with_session(config)?;
            let file_str = file.to_string_lossy().to_string();
            let file_path = Path::new(&file_str);
            let records = match format {
                ImportFormat::Csv => import_export::import_from_csv(file_path)?,
                ImportFormat::Json => import_export::import_from_json(file_path)?,
                ImportFormat::Env => import_export::import_from_dotenv(file_path)?,
            };
            let env = Environment::from_str(&environment);
            let count = vault.import_keys(records, env)?;
            println!("✅ 已导入 {} 个密钥", count);
        }

        // ==================== 导出 ====================
        Commands::Export { format, file, environment, include_values } => {
            let mut vault = create_vault_with_session(config)?;
            let filter = KeyFilter {
                environment,
                ..Default::default()
            };
            let keys = vault.list_keys_filtered(&filter)?;

            let export_data: Vec<(String, String, String, String)> = if include_values {
                keys.iter().map(|k| {
                    let value = vault.get_key(&k.name, &k.environment.to_string())
                        .map(|(_, v)| v)
                        .unwrap_or_default();
                    (k.name.clone(), k.provider.clone(), k.key_type.to_string(), value)
                }).collect()
            } else {
                keys.iter().map(|k| {
                    (k.name.clone(), k.provider.clone(), k.key_type.to_string(), String::new())
                }).collect()
            };

            let file_str = file.to_string_lossy().to_string();
            let file_path = Path::new(&file_str);
            match format {
                ImportFormat::Csv => import_export::export_to_csv(file_path, &export_data)?,
                ImportFormat::Json => import_export::export_to_json(file_path, &export_data)?,
                ImportFormat::Env => import_export::export_to_dotenv(file_path, &export_data)?,
            };
            println!("✅ 已导出 {} 个密钥到 {}", keys.len(), file.display());
        }

        // ==================== 旋转 ====================
        Commands::Rotate { name, value, environment } => {
            let mut vault = create_vault_with_session(config)?;
            let new_value = if let Some(v) = value {
                v
            } else {
                Password::new().with_prompt("输入新密钥值").interact()
                    .map_err(|e| AppError::IoError(e.to_string()))?
            };
            let entry = if let Some(ref env) = environment {
                vault.rotate_key(&name, env, &new_value)?
            } else {
                vault.rotate_key_any_env(&name, &new_value)?
            };
            println!("✅ 密钥已旋转: {} (版本: {})", entry.name, entry.version);
        }

        // ==================== 审计日志 ====================
        Commands::Audit { limit, action: _ } => {
            let mut vault = create_vault_with_session(config)?;
            let logs = vault.get_audit_logs(limit)?;

            if logs.is_empty() {
                println!("没有审计日志");
                return Ok(());
            }

            println!("{:<22} {:<20} {:<12} {}", "时间", "操作", "资源类型", "资源ID");
            println!("{}", "-".repeat(75));
            for log in &logs {
                println!("{:<22} {:<20} {:<12} {}",
                    log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    log.action,
                    log.resource_type,
                    log.resource_id.as_deref().unwrap_or("-"));
            }
        }

        // ==================== 备份 ====================
        Commands::Backup { file, encrypt: _ } => {
            let vault = create_vault_with_session(config)?;
            vault.backup(&file)?;
            println!("✅ 备份已创建: {}", file.display());
        }

        // ==================== 恢复 ====================
        Commands::Restore { file } => {
            let mut vault = create_vault(config)?;
            vault.restore(&file)?;
            println!("✅ 已从备份恢复。Vault 已锁定，请重新登录。");
        }

        // ==================== 修改密码 ====================
        Commands::ChangePassword => {
            println!("修改密码功能暂未实现");
        }

        // ==================== 标签管理 ====================
        Commands::Tag { action: _ } => {
            println!("标签管理功能暂未实现");
        }

        // ==================== Shell 集成 ====================
        Commands::Shell { action: _ } => {
            println!("Shell 集成功能暂未实现");
        }

        // ==================== 模板管理 ====================
        Commands::Template { action: _ } => {
            println!("模板管理功能暂未实现");
        }

        // ==================== 配置管理 ====================
        Commands::Config { action } => {
            match action {
                crate::cli::ConfigAction::Show => {
                    println!("当前配置:");
                    println!("  vault_path: {}", config.vault_path.display());
                    println!("  auto_lock_minutes: {}", config.auto_lock_minutes);
                    println!("  clipboard_clear_seconds: {}", config.clipboard_clear_seconds);
                    println!("  audit_log_enabled: {}", config.audit_log_enabled);
                    println!("  theme: {}", config.theme);
                    println!("  default_environment: {}", config.default_environment);
                }
                crate::cli::ConfigAction::Set { key, value } => {
                    println!("设置配置: {} = {}", key, value);
                    println!("注意：配置修改需要编辑 config.toml 文件");
                    println!("当前配置文件路径: {}", AppConfig::config_path().display());
                }
                crate::cli::ConfigAction::Reset { .. } => {
                    let default_config = AppConfig::default();
                    default_config.save().ok();
                    println!("✅ 配置已重置为默认值");
                }
            }
        }

        // ==================== 安全检查 ====================
        Commands::SecurityCheck => {
            println!("安全检查功能暂未实现");
        }

        // ==================== 环境变量 ====================
        Commands::Env { name, var, shell: _ } => {
            println!("设置环境变量: {} (var: {:?})", name, var);
            println!("暂未实现");
        }

        // ==================== GUI ====================
        Commands::Gui => {
            // GUI 在 main.rs 中单独处理，这里不会执行到
            println!("请直接运行不带子命令来启动 GUI");
        }
    }

    Ok(())
}

/// 执行密钥管理子命令
fn execute_key_action(action: KeyAction, config: AppConfig) -> Result<(), AppError> {
    match action {
        KeyAction::Add { name, provider, key_type, value, environment, description, group, tags } => {
            let mut vault = create_vault_with_session(config)?;
            let kt = KeyType::from(key_type);
            let env = Environment::from_str(&environment);
            let gid = group.and_then(|s| Uuid::parse_str(&s).ok());

            let val = if let Some(v) = value {
                v
            } else {
                Password::new().with_prompt("输入密钥值").interact()
                    .map_err(|e| AppError::IoError(e.to_string()))?
            };

            let entry = vault.add_key(name, provider, kt, &val, env, description, gid, tags)?;
            println!("✅ 密钥已添加: {} (ID: {})", entry.name, entry.id);
        }

        KeyAction::Get { name, environment, copy, full } => {
            let mut vault = create_vault_with_session(config)?;
            let (entry, decrypted) = if let Some(ref env) = environment {
                vault.get_key(&name, env)?
            } else {
                vault.get_key_any_env(&name)?
            };

            if copy {
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(&decrypted) {
                            eprintln!("⚠️ 剪贴板写入失败: {}", e);
                        } else {
                            println!("✅ 已复制到剪贴板");
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ 无法访问剪贴板: {}", e);
                    }
                }
                return Ok(());
            }

            println!("密钥详情:");
            println!("  名称: {}", entry.name);
            println!("  提供商: {}", entry.provider);
            println!("  类型: {}", entry.key_type);
            println!("  环境: {}", entry.environment);
            if full {
                println!("  值: {}", decrypted);
            } else {
                let masked = if decrypted.len() > 8 {
                    format!("{}...{}", &decrypted[..4], &decrypted[decrypted.len()-4..])
                } else {
                    "****".to_string()
                };
                println!("  值: {} (使用 --full 显示完整)", masked);
            }
            if let Some(desc) = &entry.description {
                println!("  描述: {}", desc);
            }
            println!("  创建时间: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S"));
        }

        KeyAction::List { environment, group, tag, show_hidden: _ } => {
            let mut vault = create_vault_with_session(config)?;
            let filter = KeyFilter {
                environment,
                group_id: group,
                tag,
            };
            let filtered = vault.list_keys_filtered(&filter)?;

            if filtered.is_empty() {
                println!("没有存储的密钥");
                return Ok(());
            }

            println!("{:<30} {:<15} {:<12} {:<12}", "名称", "提供商", "类型", "环境");
            println!("{}", "-".repeat(75));
            for key in &filtered {
                println!("{:<30} {:<15} {:<12} {:<12}",
                    key.name, key.provider, key.key_type, key.environment);
            }
            println!("\n共 {} 个密钥", filtered.len());
        }

        KeyAction::Update { name, environment, value, description, tags } => {
            let mut vault = create_vault_with_session(config)?;
            let entry = if let Some(ref env) = environment {
                vault.update_key(&name, env, value.as_deref(), description.as_deref(), tags)?
            } else {
                vault.update_key_any_env(&name, value.as_deref(), description.as_deref(), tags)?
            };
            println!("✅ 密钥已更新: {}", entry.name);
        }

        KeyAction::Delete { name, environment, force } => {
            if !force {
                println!("确定要删除密钥 '{}' 吗？使用 --force 跳过确认", name);
                return Ok(());
            }
            let mut vault = create_vault_with_session(config)?;
            if let Some(ref env) = environment {
                vault.delete_key(&name, env)?;
            } else {
                vault.delete_key_any_env(&name)?;
            }
            println!("✅ 密钥已删除: {}", name);
        }
    }

    Ok(())
}

/// 执行分组管理子命令
fn execute_group_action(action: GroupAction, config: AppConfig) -> Result<(), AppError> {
    match action {
        GroupAction::Create { name } => {
            let mut vault = create_vault_with_session(config)?;
            let group = vault.create_group(name)?;
            println!("✅ 分组已创建: {} (ID: {})", group.name, group.id);
        }

        GroupAction::List => {
            let mut vault = create_vault_with_session(config)?;
            let groups = vault.list_groups()?;

            if groups.is_empty() {
                println!("没有分组");
                return Ok(());
            }

            println!("{:<36} {:<20} {}", "ID", "名称", "描述");
            println!("{}", "-".repeat(70));
            for group in &groups {
                let desc_str = group.description.as_deref().unwrap_or("-");
                println!("{:<36} {:<20} {}", group.id, group.name, desc_str);
            }
            println!("\n共 {} 个分组", groups.len());
        }

        GroupAction::Rename { id, name } => {
            let mut vault = create_vault_with_session(config)?;
            let group_id = Uuid::parse_str(&id)
                .map_err(|_| AppError::InvalidInput("无效的分组ID格式".to_string()))?;
            vault.rename_group(&group_id, name.clone())?;
            println!("✅ 分组已重命名为: {}", name);
        }

        GroupAction::Delete { id, force } => {
            if !force {
                println!("确定要删除分组 '{}' 吗？使用 --force 跳过确认", id);
                return Ok(());
            }
            let mut vault = create_vault_with_session(config)?;
            let group_id = Uuid::parse_str(&id)
                .map_err(|_| AppError::InvalidInput("无效的分组ID格式".to_string()))?;
            vault.delete_group(&group_id)?;
            println!("✅ 分组已删除: {}", id);
        }
    }

    Ok(())
}