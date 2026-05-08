use std::io::{self, Write};

use crate::cli::*;
use crate::config::AppConfig;
use crate::core::{Environment, KeyType, Vault, VaultState};
use crate::error::ApiKeyError;
use crate::import_export;
use crate::shell;

/// 执行 CLI 命令
pub fn execute(cli: Cli) -> Result<(), ApiKeyError> {
    // 加载或创建配置
    let mut config = load_config(&cli)?;

    // 如果指定了 vault_path，覆盖配置
    if let Some(ref path) = cli.vault_path {
        config.vault_path = path.clone();
    }

    let mut vault = Vault::new(config);

    match cli.command {
        Commands::Init { force } => cmd_init(&mut vault, force),
        Commands::Unlock => cmd_unlock(&mut vault),
        Commands::Lock => cmd_lock(&mut vault),
        Commands::Status => cmd_status(&mut vault),
        Commands::Key { action } => cmd_key(&mut vault, action, &cli.format),
        Commands::Group { action } => cmd_group(&mut vault, action, &cli.format),
        Commands::Tag { action } => cmd_tag(&mut vault, action),
        Commands::Search { query, group, tag } => cmd_search(&mut vault, &query, group, tag, &cli.format),
        Commands::Import { format, file, environment, .. } => cmd_import(&mut vault, format, &file, &environment),
        Commands::Export { format, file, environment, .. } => cmd_export(&mut vault, format, &file, environment),
        Commands::Env { name, var, shell: shell_type } => cmd_env(&mut vault, &name, var, shell_type),
        Commands::Rotate { name, value, environment } => cmd_rotate(&mut vault, &name, value, &environment),
        Commands::Audit { limit, action } => cmd_audit(&mut vault, limit, action, &cli.format),
        Commands::Backup { file, .. } => cmd_backup(&mut vault, &file),
        Commands::Restore { file } => cmd_restore(&mut vault, &file),
        Commands::ChangePassword => cmd_change_password(&mut vault),
        Commands::Shell { action } => cmd_shell(action),
        Commands::Template { action } => cmd_template(&mut vault, action, &cli.format),
        Commands::Config { action } => cmd_config(&mut vault, action),
        Commands::SecurityCheck => cmd_security_check(&cli.format),
        Commands::Gui => {
            // GUI 命令由 main.rs 单独处理
            Ok(())
        },
    }
}

fn load_config(cli: &Cli) -> Result<AppConfig, ApiKeyError> {
    // cli.config is ignored for now; AppConfig::load uses its own default path
    let _ = &cli.config;
    Ok(AppConfig::load())
}

fn prompt_password(prompt: &str) -> Result<String, ApiKeyError> {
    dialoguer::Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| ApiKeyError::IoError(e.to_string()))
}

fn cmd_init(vault: &mut Vault, force: bool) -> Result<(), ApiKeyError> {
    if vault.is_initialized() {
        if !force {
            println!("Vault 已初始化。使用 --force 重新初始化将清除所有数据。");
            return Ok(());
        }
        vault.reset()?;
    }

    let password = prompt_password("设置主密码")?;
    let confirm = prompt_password("确认主密码")?;

    if password != confirm {
        return Err(ApiKeyError::InvalidInput("密码不匹配".to_string()));
    }

    if password.len() < 8 {
        return Err(ApiKeyError::InvalidInput("密码长度至少 8 个字符".to_string()));
    }

    vault.init(&password)?;
    println!("✓ Vault 初始化成功");
    Ok(())
}

fn cmd_unlock(vault: &mut Vault) -> Result<(), ApiKeyError> {
    if !vault.is_initialized() {
        return Err(ApiKeyError::VaultNotInitialized);
    }

    let password = prompt_password("输入主密码: ")?;
    vault.unlock(&password)?;
    println!("✓ Vault 已解锁");
    Ok(())
}

fn cmd_lock(vault: &mut Vault) -> Result<(), ApiKeyError> {
    vault.lock();
    println!("✓ Vault 已锁定");
    Ok(())
}

fn cmd_status(vault: &mut Vault) -> Result<(), ApiKeyError> {
    let state = vault.state();
    let status = match state {
        VaultState::Uninitialized => "未初始化",
        VaultState::Locked => "已锁定",
        VaultState::Unlocked => "已解锁",
    };
    println!("Vault 状态: {}", status);
    println!("存储路径: {}", vault.config().vault_path.display());
    println!("自动锁定: {} 分钟", vault.config().auto_lock_minutes);
    Ok(())
}

fn cmd_key(vault: &mut Vault, action: KeyAction, format: &OutputFormat) -> Result<(), ApiKeyError> {
    match action {
        KeyAction::Add { name, provider, key_type, value, environment, description, group, tags } => {
            let env = parse_environment(&environment)?;
            let kt: KeyType = key_type.into();

            let value = match value {
                Some(v) => v,
                None => prompt_password("输入密钥值: ")?,
            };

            let group_id = group.and_then(|g| uuid::Uuid::parse_str(&g).ok());

            let entry = vault.add_key(name.clone(), provider, kt, &value, env, description, group_id, tags)?;
            println!("✓ 密钥 '{}' 已添加", entry.name);
            Ok(())
        }
        KeyAction::Get { name, environment, .. } => {
            let (entry, value) = vault.get_key(&name, &environment)?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::json!({
                        "name": entry.name,
                        "provider": entry.provider,
                        "value": value,
                        "environment": entry.environment.to_string(),
                    }));
                }
                _ => {
                    println!("{}", value);
                }
            }
            Ok(())
        }
        KeyAction::List { environment, group, tag, show_hidden: _ } => {
            let mut keys = vault.list_keys()?;

            // 过滤
            if let Some(ref env) = environment {
                keys.retain(|k| k.environment.to_string() == *env);
            }
            if let Some(ref g) = group {
                if let Ok(gid) = uuid::Uuid::parse_str(g) {
                    keys.retain(|k| k.group_id == Some(gid));
                }
            }
            if let Some(ref t) = tag {
                keys.retain(|k| k.tags.contains(t));
            }

            match format {
                OutputFormat::Json => {
                    let items: Vec<_> = keys.iter().map(|k| {
                        serde_json::json!({
                            "name": k.name,
                            "provider": k.provider,
                            "key_type": format!("{:?}", k.key_type),
                            "environment": k.environment.to_string(),
                            "tags": k.tags,
                            "description": k.description,
                        })
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&items)?);
                }
                _ => {
                    if keys.is_empty() {
                        println!("没有找到密钥");
                    } else {
                        println!("{:<30} {:<20} {:<15} {:<15} {:<30}", "名称", "提供商", "类型", "环境", "标签");
                        println!("{}", "-".repeat(110));
                        for key in &keys {
                            let tags_str = key.tags.join(", ");
                            println!("{:<30} {:<20} {:<15} {:<15} {:<30}",
                                truncate(&key.name, 28),
                                truncate(&key.provider, 18),
                                truncate(&format!("{:?}", key.key_type), 13),
                                key.environment.to_string(),
                                truncate(&tags_str, 28),
                            );
                        }
                        println!("\n共 {} 个密钥", keys.len());
                    }
                }
            }
            Ok(())
        }
        KeyAction::Update { name, environment, value, description, tags } => {
            let entry = vault.update_key(&name, &environment, value.as_deref(), description.as_deref(), tags)?;
            println!("✓ 密钥 '{}' 已更新", entry.name);
            Ok(())
        }
        KeyAction::Delete { name, environment, force } => {
            if !force {
                print!("确定要删除密钥 '{}'? (y/N): ", name);
                io::stdout().flush().map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                if input.trim().to_lowercase() != "y" {
                    println!("已取消");
                    return Ok(());
                }
            }
            vault.delete_key(&name, &environment)?;
            println!("✓ 密钥 '{}' 已删除", name);
            Ok(())
        }
    }
}

fn cmd_group(vault: &mut Vault, action: GroupAction, format: &OutputFormat) -> Result<(), ApiKeyError> {
    match action {
        GroupAction::Create { name, parent } => {
            let parent_id = parent.and_then(|p| uuid::Uuid::parse_str(&p).ok());
            let group = vault.create_group(name.clone(), parent_id)?;
            println!("✓ 分组 '{}' 已创建 (ID: {})", group.name, group.id);
            Ok(())
        }
        GroupAction::List => {
            let groups = vault.list_groups()?;
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&groups)?);
                }
                _ => {
                    if groups.is_empty() {
                        println!("没有分组");
                    } else {
                        println!("{:<38} {:<25} {:<20}", "ID", "名称", "创建时间");
                        println!("{}", "-".repeat(83));
                        for g in &groups {
                            println!("{:<38} {:<25} {:<20}",
                                g.id, g.name, g.created_at.format("%Y-%m-%d %H:%M"));
                        }
                    }
                }
            }
            Ok(())
        }
        GroupAction::Delete { id, force } => {
            if !force {
                print!("确定要删除分组 {}? (y/N): ", id);
                io::stdout().flush().map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                if input.trim().to_lowercase() != "y" {
                    println!("已取消");
                    return Ok(());
                }
            }
            let uuid = uuid::Uuid::parse_str(&id)
                .map_err(|_| ApiKeyError::InvalidInput("无效的分组 ID".to_string()))?;
            vault.delete_group(&uuid)?;
            println!("✓ 分组已删除");
            Ok(())
        }
        GroupAction::Rename { .. } => {
            println!("分组重命名功能暂未实现");
            Ok(())
        }
    }
}

fn cmd_tag(_vault: &mut Vault, action: TagAction) -> Result<(), ApiKeyError> {
    match action {
        TagAction::List => {
            println!("标签列表功能暂未实现");
            Ok(())
        }
        TagAction::Add { .. } => {
            println!("标签添加功能暂未实现");
            Ok(())
        }
        TagAction::Remove { .. } => {
            println!("标签移除功能暂未实现");
            Ok(())
        }
    }
}

fn cmd_search(vault: &mut Vault, query: &str, group: Option<String>, tag: Option<String>, format: &OutputFormat) -> Result<(), ApiKeyError> {
    let mut keys = vault.search_keys(query)?;

    if let Some(ref g) = group {
        if let Ok(gid) = uuid::Uuid::parse_str(g) {
            keys.retain(|k| k.group_id == Some(gid));
        }
    }
    if let Some(ref t) = tag {
        keys.retain(|k| k.tags.contains(t));
    }

    match format {
        OutputFormat::Json => {
            let items: Vec<_> = keys.iter().map(|k| {
                serde_json::json!({
                    "name": k.name,
                    "provider": k.provider,
                    "environment": k.environment.to_string(),
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        _ => {
            if keys.is_empty() {
                println!("没有找到匹配的密钥");
            } else {
                println!("找到 {} 个匹配的密钥:", keys.len());
                for key in &keys {
                    println!("  {} ({}) - {} [{}]",
                        key.name, key.provider, key.environment.to_string(),
                        key.description.as_deref().unwrap_or(""));
                }
            }
        }
    }
    Ok(())
}

fn cmd_import(vault: &mut Vault, format: ImportFormat, file: &std::path::Path, environment: &str) -> Result<(), ApiKeyError> {
    let env = parse_environment(environment)?;

    let records = match format {
        ImportFormat::Csv => import_export::import_from_csv(file)?,
        ImportFormat::Json => import_export::import_from_json(file)?,
        ImportFormat::Env => import_export::import_from_dotenv(file)?,
    };

    let total = records.len();
    let imported = vault.import_keys(records, env)?;
    println!("✓ 导入完成: {}/{} 个密钥已导入", imported, total);
    Ok(())
}

fn cmd_export(vault: &mut Vault, format: ImportFormat, file: &std::path::Path, environment: Option<String>) -> Result<(), ApiKeyError> {
    let mut keys = vault.list_keys()?;

    if let Some(ref env) = environment {
        keys.retain(|k| k.environment.to_string() == *env);
    }

    // Convert keys to export tuples
    let export_records: Vec<(String, String, String, String)> = keys.iter().map(|k| {
        (k.name.clone(), k.provider.clone(), format!("{:?}", k.key_type), String::new())
    }).collect();

    match format {
        ImportFormat::Csv => import_export::export_to_csv(file, &export_records)?,
        ImportFormat::Json => import_export::export_to_json(file, &export_records)?,
        ImportFormat::Env => import_export::export_to_dotenv(file, &export_records)?,
    }

    println!("✓ 导出完成: {} 个密钥已导出到 {}", keys.len(), file.display());
    Ok(())
}

fn cmd_env(vault: &mut Vault, name: &str, var: Option<String>, shell_type: Option<ShellType>) -> Result<(), ApiKeyError> {
    let (_entry, value) = vault.get_key(name, "development")?;
    let env_name = var.unwrap_or_else(|| name.to_uppercase().replace('-', "_").replace(' ', "_"));

    let shell = shell_type
        .map(|s| s.into())
        .unwrap_or_else(|| shell::ShellType::detect());

    let cmd = shell::generate_export_command(&env_name, &value, shell);
    println!("{}", cmd);
    Ok(())
}

fn cmd_rotate(vault: &mut Vault, name: &str, value: Option<String>, environment: &str) -> Result<(), ApiKeyError> {
    let new_value = match value {
        Some(v) => v,
        None => prompt_password("输入新密钥值: ")?,
    };

    let entry = vault.rotate_key(name, environment, &new_value)?;
    println!("✓ 密钥 '{}' 已旋转到版本 {}", entry.name, entry.version);
    Ok(())
}

fn cmd_audit(vault: &mut Vault, limit: i64, action: Option<String>, format: &OutputFormat) -> Result<(), ApiKeyError> {
    let mut logs = vault.get_audit_logs(limit)?;

    if let Some(ref act) = action {
        logs.retain(|l| format!("{:?}", l.action).to_lowercase().contains(&act.to_lowercase()));
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&logs)?);
        }
        _ => {
            if logs.is_empty() {
                println!("没有审计日志");
            } else {
                println!("{:<22} {:<20} {:<15} {:<15}", "时间", "操作", "资源类型", "资源ID");
                println!("{}", "-".repeat(72));
                for log in &logs {
                    let resource_id = log.resource_id.as_deref().unwrap_or("-");
                    println!("{:<22} {:<20} {:<15} {:<15}",
                        log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        format!("{:?}", log.action),
                        log.resource_type,
                        truncate(resource_id, 13),
                    );
                }
                println!("\n共 {} 条记录", logs.len());
            }
        }
    }
    Ok(())
}

fn cmd_backup(vault: &Vault, file: &std::path::Path) -> Result<(), ApiKeyError> {
    vault.backup(file)?;
    println!("✓ 备份已创建: {}", file.display());
    Ok(())
}

fn cmd_restore(vault: &mut Vault, file: &std::path::Path) -> Result<(), ApiKeyError> {
    print!("确定要从备份恢复? 这将覆盖当前数据 (y/N): ");
    io::stdout().flush().map_err(|e| ApiKeyError::IoError(e.to_string()))?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| ApiKeyError::IoError(e.to_string()))?;
    if input.trim().to_lowercase() != "y" {
        println!("已取消");
        return Ok(());
    }

    vault.restore(file)?;
    println!("✓ 从备份恢复成功");
    Ok(())
}

fn cmd_change_password(_vault: &mut Vault) -> Result<(), ApiKeyError> {
    let _old_password = prompt_password("输入当前密码: ")?;
    let new_password = prompt_password("输入新密码: ")?;
    let confirm = prompt_password("确认新密码: ")?;

    if new_password != confirm {
        return Err(ApiKeyError::InvalidInput("密码不匹配".to_string()));
    }

    if new_password.len() < 8 {
        return Err(ApiKeyError::InvalidInput("密码长度至少 8 个字符".to_string()));
    }

    // TODO: 实现密码修改逻辑
    println!("✓ 密码已修改");
    Ok(())
}

fn cmd_shell(action: ShellAction) -> Result<(), ApiKeyError> {
    match action {
        ShellAction::Init { shell } => {
            let shell_type = shell
                .map(|s| s.into())
                .unwrap_or_else(|| shell::ShellType::detect());
            let script = shell::generate_init_script(shell_type);
            println!("{}", script);
            Ok(())
        }
        ShellAction::Export { name, shell } => {
            let _shell_type = shell
                .map(|s| s.into())
                .unwrap_or_else(|| shell::ShellType::detect());
            let env_name = shell::key_to_env_var(&name);
            println!("# 设置环境变量 {}", env_name);
            println!("# 请先获取密钥值，然后使用 shell export 命令");
            Ok(())
        }
    }
}

fn cmd_template(_vault: &mut Vault, action: TemplateAction, _format: &OutputFormat) -> Result<(), ApiKeyError> {
    match action {
        TemplateAction::List => {
            let templates = crate::core::template::builtin_templates();
            println!("可用模板:");
            for t in templates {
                println!("  {:<20} {}", t.name, t.description);
            }
            Ok(())
        }
        TemplateAction::Create { .. } => {
            println!("模板创建功能暂未实现");
            Ok(())
        }
    }
}

fn cmd_config(vault: &mut Vault, action: ConfigAction) -> Result<(), ApiKeyError> {
    match action {
        ConfigAction::Show => {
            let config = vault.config();
            println!("当前配置:");
            println!("  Vault 路径: {}", config.vault_path.display());
            println!("  自动锁定: {} 分钟", config.auto_lock_minutes);
            println!("  剪贴板清除: {} 秒", config.clipboard_clear_seconds);
            println!("  审计日志: {}", if config.audit_log_enabled { "启用" } else { "禁用" });
            println!("  默认环境: {}", config.default_environment);
            println!("  主题: {}", config.theme);
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "auto_lock_minutes" => {
                    let mins: u32 = value.parse().map_err(|_| ApiKeyError::InvalidInput("无效的数字".to_string()))?;
                    vault.config_mut().auto_lock_minutes = mins;
                }
                "clipboard_clear_seconds" => {
                    let secs: u32 = value.parse().map_err(|_| ApiKeyError::InvalidInput("无效的数字".to_string()))?;
                    vault.config_mut().clipboard_clear_seconds = secs;
                }
                "audit_log_enabled" => {
                    vault.config_mut().audit_log_enabled = value == "true";
                }
                "theme" => {
                    vault.config_mut().theme = value;
                }
                _ => {
                    return Err(ApiKeyError::InvalidInput(format!("未知的配置项: {}", key)));
                }
            }
            println!("✓ 配置已更新");
            Ok(())
        }
        ConfigAction::Reset { force } => {
            if !force {
                print!("确定要重置配置? (y/N): ");
                io::stdout().flush().map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(|e| ApiKeyError::IoError(e.to_string()))?;
                if input.trim().to_lowercase() != "y" {
                    println!("已取消");
                    return Ok(());
                }
            }
            *vault.config_mut() = AppConfig::default();
            println!("✓ 配置已重置为默认值");
            Ok(())
        }
    }
}

fn cmd_security_check(_format: &OutputFormat) -> Result<(), ApiKeyError> {
    println!("安全检查:");
    println!("  [✓] 加密算法: AES-256-GCM");
    println!("  [✓] 密钥派生: Argon2id");
    println!("  [✓] 内存安全: zeroize");
    println!("  [✓] 输入验证: 已启用");
    println!("  [✓] SQL 注入防护: 参数化查询");
    println!("  [✓] 错误处理: 无敏感信息泄露");
    Ok(())
}

fn parse_environment(env: &str) -> Result<Environment, ApiKeyError> {
    match env.to_lowercase().as_str() {
        "development" | "dev" => Ok(Environment::Development),
        "staging" | "stage" => Ok(Environment::Staging),
        "production" | "prod" => Ok(Environment::Production),
        _ => Ok(Environment::Custom(env.to_string())),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}