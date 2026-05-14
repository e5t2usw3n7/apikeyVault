pub mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// apikey_etorer - 安全的 API 密钥管理工具
#[derive(Parser)]
#[command(name = "apikey-etorer")]
#[command(version = "1.0.0")]
#[command(about = "安全的 API 密钥管理工具", long_about = None)]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Vault 数据目录
    #[arg(long)]
    pub vault_path: Option<PathBuf>,

    /// 详细输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 输出格式
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 初始化 Vault
    Init {
        /// 跳过确认提示
        #[arg(long)]
        force: bool,
    },

    /// 解锁 Vault
    Unlock,

    /// 锁定 Vault
    Lock,

    /// 查看 Vault 状态
    Status,

    /// 密钥管理
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },

    /// 分组管理
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },

    /// 标签管理
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// 搜索密钥
    Search {
        /// 搜索查询
        query: String,

        /// 过滤分组
        #[arg(short, long)]
        group: Option<String>,

        /// 过滤标签
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// 导入/导出
    Import {
        /// 导入格式
        #[arg(value_enum)]
        format: ImportFormat,

        /// 文件路径
        file: PathBuf,

        /// 目标环境
        #[arg(short, long, default_value = "development")]
        environment: String,

        /// 跳过已存在的密钥
        #[arg(long)]
        skip_existing: bool,
    },

    Export {
        /// 导出格式
        #[arg(value_enum)]
        format: ImportFormat,

        /// 文件路径
        file: PathBuf,

        /// 过滤环境
        #[arg(short, long)]
        environment: Option<String>,

        /// 包含密钥值
        #[arg(long)]
        include_values: bool,
    },

    /// 设置环境变量
    Env {
        /// 密钥名称
        name: String,

        /// 环境变量名（可选）
        #[arg(short, long)]
        var: Option<String>,

        /// Shell 类型
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },

    /// 旋转密钥
    Rotate {
        /// 密钥名称
        name: String,

        /// 新密钥值（如果不提供则交互输入）
        #[arg(short, long)]
        value: Option<String>,

        /// 环境（不指定则自动搜索所有环境）
        #[arg(short, long)]
        environment: Option<String>,
    },

    /// 审计日志
    Audit {
        /// 显示条数
        #[arg(short, long, default_value = "20")]
        limit: i64,

        /// 过滤操作类型
        #[arg(long)]
        action: Option<String>,
    },

    /// 备份 Vault
    Backup {
        /// 备份文件路径
        file: PathBuf,

        /// 加密备份
        #[arg(long)]
        encrypt: bool,
    },

    /// 从备份恢复
    Restore {
        /// 备份文件路径
        file: PathBuf,
    },

    /// 修改主密码
    ChangePassword,

    /// Shell 集成
    Shell {
        #[command(subcommand)]
        action: ShellAction,
    },

    /// 模板管理
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// 管理配置
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 安全检查
    SecurityCheck,

    /// 启动桌面 GUI 应用
    Gui,
}

#[derive(Subcommand)]
pub enum KeyAction {
    /// 添加密钥
    Add {
        /// 密钥名称
        name: String,

        /// 服务提供商
        #[arg(short, long)]
        provider: String,

        /// 密钥类型
        #[arg(short, long, value_enum, default_value = "api-key")]
        key_type: CliKeyType,

        /// 密钥值（如果不提供则交互输入）
        #[arg(short, long)]
        value: Option<String>,

        /// 环境
        #[arg(short, long, default_value = "development")]
        environment: String,

        /// 描述
        #[arg(long)]
        description: Option<String>,

        /// 分组 ID
        #[arg(short, long)]
        group: Option<String>,

        /// 标签
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// 获取密钥值
    Get {
        /// 密钥名称
        name: String,

        /// 环境（不指定则自动搜索所有环境）
        #[arg(short, long)]
        environment: Option<String>,

        /// 复制到剪贴板
        #[arg(long)]
        copy: bool,

        /// 显示完整值
        #[arg(long)]
        full: bool,
    },

    /// 列出密钥
    List {
        /// 过滤环境
        #[arg(short, long)]
        environment: Option<String>,

        /// 过滤分组
        #[arg(short, long)]
        group: Option<String>,

        /// 过滤标签
        #[arg(short, long)]
        tag: Option<String>,

        /// 显示隐藏的密钥
        #[arg(long)]
        show_hidden: bool,
    },

    /// 更新密钥
    Update {
        /// 密钥名称
        name: String,

        /// 环境（不指定则自动搜索所有环境）
        #[arg(short, long)]
        environment: Option<String>,

        /// 新密钥值
        #[arg(short, long)]
        value: Option<String>,

        /// 新描述
        #[arg(long)]
        description: Option<String>,

        /// 新标签
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// 删除密钥
    Delete {
        /// 密钥名称
        name: String,

        /// 环境（不指定则自动搜索所有环境）
        #[arg(short, long)]
        environment: Option<String>,

        /// 跳过确认
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum GroupAction {
    /// 创建分组
    Create {
        /// 分组名称
        name: String,
    },

    /// 列出分组
    List,

    /// 删除分组
    Delete {
        /// 分组 ID 或名称
        id: String,

        /// 跳过确认
        #[arg(long)]
        force: bool,
    },

    /// 重命名分组
    Rename {
        /// 分组 ID
        id: String,

        /// 新名称
        name: String,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    /// 列出所有标签
    List,

    /// 添加标签到密钥
    Add {
        /// 密钥名称
        key_name: String,

        /// 标签名
        tag: String,
    },

    /// 从密钥移除标签
    Remove {
        /// 密钥名称
        key_name: String,

        /// 标签名
        tag: String,
    },
}

#[derive(Subcommand)]
pub enum ShellAction {
    /// 生成 Shell 初始化脚本
    Init {
        /// Shell 类型
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },

    /// 生成环境变量导出命令
    Export {
        /// 密钥名称
        name: String,

        /// Shell 类型
        #[arg(long, value_enum)]
        shell: Option<ShellType>,
    },
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// 列出可用模板
    List,

    /// 从模板创建密钥
    Create {
        /// 模板名称
        template: String,

        /// 密钥名称
        name: String,

        /// 环境
        #[arg(short, long, default_value = "development")]
        environment: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 显示当前配置
    Show,

    /// 设置配置值
    Set {
        /// 配置键
        key: String,

        /// 配置值
        value: String,
    },

    /// 重置为默认配置
    Reset {
        /// 跳过确认
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ImportFormat {
    Csv,
    Json,
    Env,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliKeyType {
    #[value(name = "api-key")]
    ApiKey,
    #[value(name = "oauth-token")]
    OAuthToken,
    #[value(name = "ssh-key")]
    SshKey,
    #[value(name = "certificate")]
    Certificate,
    #[value(name = "jwt-token")]
    JwtToken,
    #[value(name = "password")]
    Password,
    #[value(name = "other")]
    Other,
}

impl From<CliKeyType> for crate::core::key::KeyType {
    fn from(kt: CliKeyType) -> Self {
        match kt {
            CliKeyType::ApiKey => crate::core::key::KeyType::ApiKey,
            CliKeyType::OAuthToken => crate::core::key::KeyType::OAuthToken,
            CliKeyType::SshKey => crate::core::key::KeyType::SshKey,
            CliKeyType::Certificate => crate::core::key::KeyType::Certificate,
            CliKeyType::JwtToken => crate::core::key::KeyType::JwtToken,
            CliKeyType::Password => crate::core::key::KeyType::Password,
            CliKeyType::Other => crate::core::key::KeyType::Other("custom".to_string()),
        }
    }
}

// impl From<ShellType> for crate::shell::ShellType {
//     fn from(st: ShellType) -> Self {
//         match st {
//             ShellType::Bash => crate::shell::ShellType::Bash,
//             ShellType::Zsh => crate::shell::ShellType::Zsh,
//             ShellType::Fish => crate::shell::ShellType::Fish,
//             ShellType::PowerShell => crate::shell::ShellType::PowerShell,
//         }
//     }
// }