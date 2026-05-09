// ============================================================
// src/gui/mod.rs - API Key Vault 桌面 GUI 主文件
// 本文件使用 egui (基于 eframe) 实现即时模式 GUI
// ============================================================

// ==================== 导入区 ====================
// std::path::PathBuf - Rust标准库的文件路径类型，用于指定Vault数据库文件的位置
use std::path::PathBuf;
// chrono::Utc - 时间库，提供UTC时间，用于通知的时间戳和自动锁定判断
use chrono::Utc;

// egui 是 Rust 的即时模式 GUI 框架；eframe 是它的桌面应用框架
// egui - 核心UI框架
// Color32 - RGBA颜色，每个分量u8精度（0-255）
// RichText - 富文本类型，可设置字号、颜色、粗体等样式
// Vec2 - 二维向量，用于指定UI元素尺寸 (x, y)
// Stroke - 描边样式，包含线宽和颜色
// Rounding - 圆角半径，可分别设置四个角
// FontId - 字体标识（大小 + 字体族）
// FontFamily - 字体族枚举：Proportional（比例字体）、Monospace（等宽字体）
use eframe::egui;
use eframe::egui::{Color32, RichText, Vec2, Stroke, Rounding, FontId, FontFamily};

// ==================== 本项目模块导入 ====================
// AppConfig - 应用配置结构体，从 config.toml 文件加载的配置
use crate::config::AppConfig;
// Vault - 核心保险库结构体，管理加密存储、密钥CRUD、认证等核心功能
// VaultState - Vault的状态枚举：Uninitialized(未初始化)、Locked(已锁定)、Unlocked(已解锁)
use crate::core::vault::{Vault, VaultState};
// KeyEntry - 密钥条目的数据结构，包含名称、值、类型、环境等字段
// KeyType - 密钥类型枚举：ApiKey、OAuthToken、SshKey、Certificate、JwtToken、Password、Other
// Environment - 环境枚举：Development、Staging、Production、Testing
use crate::core::key::{KeyEntry, KeyType, Environment};
// Group - 分组数据结构，用于将密钥组织到逻辑分组中
use crate::core::group::Group;
// AuditEntry - 审计日志条目，记录每次对Vault的操作
// AuditAction - 审计操作类型枚举，如KeyCreated、KeyViewed、VaultUnlocked等
use crate::core::audit::{AuditEntry, AuditAction};

// ==================== 视图枚举 ====================
// 【View 枚举】定义了应用中所有可能的页面视图
// egui 是即时模式GUI，每帧都会重新渲染整个UI
// 因此使用枚举来表示当前应显示哪个页面，每帧根据 current_view 渲染对应的视图
#[derive(Debug, Clone, PartialEq)]
enum View {
    // Login - 登录/初始化页面，首次使用或锁定后显示
    Login,
    // Dashboard - 仪表板页面，显示统计概览和快捷操作入口
    Dashboard,
    // KeyList - 密钥列表页面，以表格形式展示所有密钥
    KeyList,
    // KeyDetail(usize) - 密钥详情页，参数是 key_list 中的索引位置
    KeyDetail(usize),
    // KeyEdit(Option<usize>) - 密钥编辑/新建表单页面
    // None = 新建密钥，Some(index) = 编辑现有密钥（index为在key_list中的索引）
    KeyEdit(Option<usize>),
    // GroupList - 分组管理页面，可以查看、新建、删除分组
    GroupList,
    // Settings - 设置页面，包含Vault信息、安全设置、修改密码等
    Settings,
    // AuditLog - 审计日志页面，查看所有操作历史记录
    AuditLog,
    // ImportExport - 导入导出页面，支持CSV/JSON/.env格式的导入导出
    ImportExport,
    // Search - 全局搜索页面，搜索所有密钥的名称、提供商、描述等
    Search,
}

// ==================== 通知系统 ====================
// 【Notification】结构体表示右上角弹出的通知条
// 在egui中使用Area（浮动区域）实现，不参与正常布局流
// 每帧检查是否过期，过期后自动从队列中移除
#[derive(Debug, Clone)]
struct Notification {
    // message - 通知显示的文本内容
    message: String,
    // is_error - 通知类型：true=红色错误通知，false=绿色成功通知
    is_error: bool,
    // created_at - 通知创建的UTC时间戳，用于判断是否过期
    created_at: chrono::DateTime<Utc>,
    // duration_secs - 通知显示的持续时间（秒），超过此时间自动移除
    duration_secs: f64,
}

impl Notification {
    // success() - 创建一个成功类型的通知
    // msg: impl Into<String> - 接受任何可以转换为String的类型（&str, String等）
    // 绿色外观，默认显示3秒后自动消失
    fn success(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),       // 将传入参数转换为String
            is_error: false,            // false表示成功类型（绿色）
            created_at: Utc::now(),     // 记录当前时间作为创建时间
            duration_secs: 3.0,         // 3秒后自动消失
        }
    }

    // error() - 创建一个错误类型的通知
    // 红色外观，默认显示5秒后自动消失（比成功通知稍长，以便用户阅读错误信息）
    fn error(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_error: true,             // true表示错误类型（红色）
            created_at: Utc::now(),
            duration_secs: 5.0,         // 5秒后自动消失
        }
    }

    // is_expired() - 判断通知是否已过期
    // 计算当前时间与创建时间的毫秒差，转换为秒后与 duration_secs 比较
    // 返回 true 表示应移除该通知
    fn is_expired(&self) -> bool {
        // (Utc::now() - self.created_at) 返回 TimeDelta 类型
        // .num_milliseconds() 获取毫秒数，除以1000转为秒
        let elapsed = (Utc::now() - self.created_at).num_milliseconds() as f64 / 1000.0;
        // 如果经过时间大于设定的持续时间，则过期
        elapsed > self.duration_secs
    }
}

// ==================== 主题颜色 ====================
// 【ThemeColors】结构体集中管理所有UI元素的颜色
// 方便后续切换深色/浅色主题，只需通过 settings_theme 字段选择调用 dark_theme() 或 light_theme()
// 每个字段对应一种UI元素的颜色值
struct ThemeColors {
    bg_primary: Color32,      // 主背景色：页面最底层的背景颜色
    bg_secondary: Color32,    // 次要背景色：表头、面板间隔、状态栏背景
    bg_sidebar: Color32,      // 侧边栏背景色：左侧导航栏的底色
    bg_card: Color32,         // 卡片背景色：内容区域/卡片容器的底色
    bg_input: Color32,        // 输入框/按钮背景色：文本输入框和按钮的底色
    accent: Color32,          // 主题强调色：紫色系，用于高亮、选中态、链接文本
    _accent_hover: Color32,   // 强调色的悬停态颜色（以下划线开头表示当前暂未使用）
    text_primary: Color32,    // 主文字颜色：正文的主要文字颜色（白色/深色）
    text_secondary: Color32,  // 次要文字颜色：灰色辅助文字
    text_dim: Color32,        // 最淡文字颜色：用于占位提示、辅助说明等最不重要的文字
    border: Color32,          // 边框/分隔线颜色：容器边框、分隔线的颜色
    success: Color32,         // 成功状态颜色：绿色，用于成功提示
    warning: Color32,         // 警告状态颜色：黄色，用于警告提示
    error: Color32,           // 错误状态颜色：红色，用于错误提示
    danger: Color32,          // 危险操作颜色：深红色，用于"删除"、"重置"等危险操作的按钮
}

// dark_theme() - 深色主题配色方案
// 适合在暗光环境下长时间使用，深色背景可减少视觉疲劳
fn dark_theme() -> ThemeColors {
    ThemeColors {
        bg_primary: Color32::from_rgb(18, 18, 24),            // 接近黑色的深灰底色
        bg_secondary: Color32::from_rgb(25, 25, 35),          // 略亮的深色背景
        bg_sidebar: Color32::from_rgb(20, 20, 30),            // 侧边栏深色底色
        bg_card: Color32::from_rgb(30, 30, 42),               // 卡片稍亮于背景
        bg_input: Color32::from_rgb(35, 35, 50),              // 输入框底色
        accent: Color32::from_rgb(108, 92, 231),              // 紫罗兰色主题色
        _accent_hover: Color32::from_rgb(128, 112, 251),     // 悬停时略亮
        text_primary: Color32::from_rgb(230, 230, 240),       // 近乎白色的主文字
        text_secondary: Color32::from_rgb(160, 160, 180),     // 灰色次要文字
        text_dim: Color32::from_rgb(100, 100, 120),           // 暗淡文字
        border: Color32::from_rgb(50, 50, 65),                // 深色边框
        success: Color32::from_rgb(46, 204, 113),             // 翠绿色
        warning: Color32::from_rgb(241, 196, 15),             // 金黄色
        error: Color32::from_rgb(231, 76, 60),                // 亮红色
        danger: Color32::from_rgb(192, 57, 43),               // 暗红色（危险操作）
    }
}

// light_theme() - 浅色主题配色方案
// 明亮背景，适合在光线充足的环境下使用
fn light_theme() -> ThemeColors {
    ThemeColors {
        bg_primary: Color32::from_rgb(245, 245, 250),         // 浅灰白色背景
        bg_secondary: Color32::from_rgb(235, 235, 242),       // 略深的次要背景
        bg_sidebar: Color32::from_rgb(225, 225, 235),         // 侧边栏浅灰色
        bg_card: Color32::from_rgb(255, 255, 255),            // 纯白卡片底色
        bg_input: Color32::from_rgb(240, 240, 248),           // 输入框底色
        accent: Color32::from_rgb(108, 92, 231),              // 与深色主题相同的紫罗兰色
        _accent_hover: Color32::from_rgb(88, 72, 211),        // 悬停时略暗（浅色主题下相反）
        text_primary: Color32::from_rgb(30, 30, 40),          // 深色主文字
        text_secondary: Color32::from_rgb(80, 80, 100),       // 灰色次要文字
        text_dim: Color32::from_rgb(140, 140, 160),           // 浅灰色辅助文字
        border: Color32::from_rgb(200, 200, 210),             // 浅灰色边框
        success: Color32::from_rgb(39, 174, 96),              // 深绿色
        warning: Color32::from_rgb(211, 170, 10),             // 暗黄色
        error: Color32::from_rgb(192, 57, 43),                // 暗红色
        danger: Color32::from_rgb(169, 50, 38),               // 更深红色（危险操作）
    }
}

// ==================== 密钥编辑表单状态 ====================
// 【KeyEditForm】密钥编辑/新建页面的表单状态结构体
// 在egui的即时模式下，UI每帧重建，表单数据需要存储在应用状态中保持
// 这个结构体封装了所有表单字段以及验证错误信息
#[derive(Debug, Clone)]
struct KeyEditForm {
    name: String,              // 密钥名称 —— 唯一标识，如 "openai-api-key"
    provider: String,          // 提供商名称 —— 如 "OpenAI"、"AWS"、"Google"
    key_type_str: String,      // 密钥类型字符串 —— 用下拉选择框选择，可选值："api_key"/"oauth"/"ssh"/"cert"/"jwt"/"password"
    value: String,             // 密钥值 —— 编辑模式下不会预填充已加密的值，用户手动输入新值
    environment_str: String,   // 环境字符串 —— "development"/"staging"/"production"/"testing"
    description: String,       // 可选描述文本 —— 用户可添加对该密钥的说明
    tags_str: String,          // 标签字符串 —— 逗号分隔，如 "prod, api, v2"，后端会拆分
    group_id_str: String,      // 分组ID字符串 —— UUID格式字符串，空字符串表示无分组
    expires_at_str: String,    // 过期日期字符串 —— "YYYY-MM-DD"格式，空字符串表示永不过期
    show_value: bool,          // 是否显示密钥值 —— 用户通过眼睛图标切换显示/隐藏
    name_error: Option<String>,// 名称验证错误信息 —— 表单验证失败时设置
    value_error: Option<String>,// 密钥值验证错误信息
}

// 为 KeyEditForm 实现 Default trait —— 新建密钥时的默认初始值
impl Default for KeyEditForm {
    fn default() -> Self {
        Self {
            name: String::new(),                    // 空名称
            provider: String::new(),                // 空提供商
            key_type_str: "api_key".to_string(),    // 默认类型为 API Key
            value: String::new(),                   // 空密钥值
            environment_str: "development".to_string(), // 默认环境为 development
            description: String::new(),             // 空描述
            tags_str: String::new(),                // 空标签
            group_id_str: String::new(),            // 空分组
            expires_at_str: String::new(),          // 空过期时间
            show_value: false,                      // 默认隐藏密钥值
            name_error: None,                       // 无名称错误
            value_error: None,                      // 无值错误
        }
    }
}

impl KeyEditForm {
    // from_entry() - 从已有的 KeyEntry 加载表单数据（用于编辑模式）
    // entry: &KeyEntry - 要编辑的密钥条目引用
    // _vault: &Vault - Vault引用（当前未使用，保留用于未来可能的功能，如前缀下划线标记）
    // 注意：value 字段留空，因为出于安全考虑，不预加载已加密的密钥值
    fn from_entry(entry: &KeyEntry, _vault: &Vault) -> Self {
        Self {
            name: entry.name.clone(),               // 复制密钥名称
            provider: entry.provider.clone(),       // 复制提供商
            key_type_str: key_type_to_str(&entry.key_type), // 将KeyType枚举转为字符串
            value: String::new(),                   // 安全考虑：不加载加密值
            environment_str: entry.environment.to_string(), // 环境转为字符串
            description: entry.description.clone().unwrap_or_default(), // Option展开：None则用空字符串
            tags_str: entry.tags.join(", "),        // Vec<String> 用逗号连接为字符串
            group_id_str: entry.group_id.map(|id| id.to_string()).unwrap_or_default(), // Option<Uuid> 转字符串
            expires_at_str: entry.expires_at.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            show_value: false,                      // 默认隐藏值
            name_error: None,                       // 清除之前的错误
            value_error: None,
        }
    }

    // validate() - 表单验证方法
    // 在保存前调用，检查各字段的有效性
    // 返回 true 表示验证通过，可以保存；false 表示验证失败，错误信息已填入 error 字段
    fn validate(&mut self) -> bool {
        let mut valid = true;  // 先假设验证通过

        // ===== 验证名称字段 =====
        if self.name.is_empty() {
            // 名称不能为空
            self.name_error = Some("名称不能为空".to_string());
            valid = false;
        } else if self.name.len() > 128 {
            // 名称长度限制：最多128个字符
            self.name_error = Some("名称长度不能超过128字符".to_string());
            valid = false;
        } else if self.name.chars().any(|c| c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
            // 防止文件名特殊字符（这些字符可能在文件系统中引起问题）
            // 检查的字符: / \ : * ? " < > |
            self.name_error = Some("名称不能包含特殊字符: / \\ : * ? \" < > |".to_string());
            valid = false;
        } else {
            // 名称验证通过，清除之前的错误
            self.name_error = None;
        }

        // ===== 验证密钥值字段 =====
        if self.value.is_empty() {
            // 密钥值不能为空
            self.value_error = Some("密钥值不能为空".to_string());
            valid = false;
        } else {
            self.value_error = None;
        }

        // 返回最终验证结果
        valid
    }
}

// key_type_to_str() - 将 KeyType 枚举转换为字符串表示
// 用于在下拉框和显示中使用字符串形式的密钥类型
fn key_type_to_str(kt: &KeyType) -> String {
    match kt {
        KeyType::ApiKey => "api_key".to_string(),           // API密钥
        KeyType::OAuthToken => "oauth".to_string(),         // OAuth令牌
        KeyType::SshKey => "ssh".to_string(),               // SSH密钥
        KeyType::Certificate => "cert".to_string(),         // 证书
        KeyType::JwtToken => "jwt".to_string(),             // JWT令牌
        KeyType::Password => "password".to_string(),        // 密码
        KeyType::Other(s) => s.clone(),                     // 其他自定义类型（直接克隆字符串）
    }
}

// ==================== 主应用结构 ====================
// 【VaultApp】GUI 应用的核心状态结构体（整个应用的"单一数据源"）
// 在egui的即时模式下，所有UI状态每帧都需要保持存在
// 这个结构体包含了所有页面的状态字段和核心功能引用
pub struct VaultApp {
    // ----- Vault核心引用 -----
    vault: Vault,                    // 核心保险库实例：管理加密存储、认证、密钥CRUD等功能
    current_view: View,              // 当前正在显示的页面视图
    previous_view: View,             // 上一个页面视图（用于实现"返回"导航功能）

    // ----- 登录/初始化状态 -----
    password_input: String,          // 密码输入框的当前文本内容
    password_confirm: String,        // 初始化时的密码确认输入框内容（首次使用需要确认密码）
    show_password: bool,             // 是否显示密码文本（用户点击眼睛图标切换）
    login_error: Option<String>,     // 登录/初始化失败时的错误信息（显示在登录页面）
    _password_strength: Option<(u8, String)>, // 密码强度评估结果 (分数0-4, 反馈文本)，下划线前缀表示暂未使用

    // ----- 密钥列表状态 -----
    key_list: Vec<KeyEntry>,         // 所有密钥的缓存列表（从Vault中加载）
    key_search_query: String,        // 密钥列表页的搜索关键词输入框内容
    key_filter_env: String,          // 按环境过滤的下拉选择（空字符串=不过滤，显示所有环境）
    _key_filter_group: String,       // 按分组过滤（暂未实现功能，前缀下划线标记）
    key_sort_column: usize,          // 当前排序列的索引：0=名称, 1=提供商, 2=类型, 3=环境, 4=创建时间
    key_sort_ascending: bool,        // 排序方向：true=升序, false=降序

    // ----- 密钥详情页状态 -----
    decrypted_value: Option<String>, // 缓存从Vault解密后的密钥值（点击"显示"后获取并缓存）
    show_decrypted_value: bool,      // 是否显示解密后的值（用户在详情页切换显示/隐藏）
    selected_key_index: Option<usize>, // 当前选中的密钥在 key_list 中的索引

    // ----- 密钥编辑页状态 -----
    edit_form: KeyEditForm,          // 编辑表单的所有字段状态
    edit_is_new: bool,               // 区分新建/编辑模式：true=新建, false=编辑现有

    // ----- 分组列表页状态 -----
    group_list: Vec<Group>,          // 所有分组的缓存列表（从Vault中加载）

    // ----- 审计日志页状态 -----
    audit_logs: Vec<AuditEntry>,     // 审计日志列表（从Vault中加载，最多50条）

    // ----- 搜索页状态 -----
    search_query: String,            // 全局搜索页的搜索关键词输入框内容
    search_results: Vec<KeyEntry>,   // 搜索结果列表（调用 vault.search_keys() 后的结果）

    // ----- 导入导出页状态 -----
    import_format: String,           // 导入格式选择：csv/json/dotenv
    import_file_path: String,        // 导入文件的路径（由用户手动输入）
    export_format: String,           // 导出格式选择：csv/json/dotenv
    export_file_path: String,        // 导出文件的目标路径（由用户手动输入）

    // ----- 设置页状态 -----
    settings_auto_lock: u32,         // 自动锁定时间（分钟）：无操作N分钟后自动锁定Vault
    settings_clipboard_clear: u32,   // 剪贴板自动清除时间（秒）：复制到剪贴板后自动清除
    settings_theme: String,          // 主题选择：dark（暗色）或 light（亮色）
    settings_default_env: String,    // 默认环境：新建密钥时自动选择的环境
    settings_audit_enabled: bool,    // 是否启用审计日志记录
    new_password: String,            // 修改密码时的"新密码"输入框内容
    new_password_confirm: String,    // 修改密码时的"确认新密码"输入框内容
    old_password: String,            // 修改密码时的"旧密码"输入框内容（用于验证身份）
    change_password_error: Option<String>, // 修改密码操作失败时的错误信息
    change_password_success: bool,   // 修改密码成功的标志（成功后显示成功消息）

    // ----- 分组编辑状态 -----
    new_group_name: String,          // 新建分组的名称输入框内容
    new_group_error: Option<String>, // 新建分组时的错误信息

    // ----- 通知系统状态 -----
    notifications: Vec<Notification>, // 通知队列：存储所有待显示的通知，每帧检查过期

    // ----- 初始化状态 -----
    is_initialized: bool,            // Vault 是否已经初始化：true=已初始化（有数据库文件），false=首次使用

    // ----- 侧边栏状态 -----
    sidebar_collapsed: bool,         // 侧边栏是否折叠：true=只显示图标，false=显示图标+文字标签

    // ----- 自动锁定追踪 -----
    _last_interaction: Option<chrono::DateTime<Utc>>, // 上次用户交互的时间戳，用于自动锁定判断（暂未直接使用）

    // ----- 确认对话框状态 -----
    confirm_dialog: Option<ConfirmDialog>, // 当前显示的确认对话框：Some=正在显示，None=无对话框
}

// ==================== 确认对话框 ====================
// 【ConfirmDialog】结构体：在执行危险操作前显示确认对话框
// 存储对话框的标题、提示信息和确认后要执行的操作
#[derive(Debug, Clone)]
struct ConfirmDialog {
    title: String,                        // 对话框标题文本
    message: String,                      // 确认提示信息（通知用户即将执行的操作及其后果）
    on_confirm_action: ConfirmAction,     // 用户点击"确认"后要执行的枚举动作
}

// 【ConfirmAction】枚举：确认对话框中可执行的操作类型
// 每种变体携带执行该操作所需的参数
#[derive(Debug, Clone)]
enum ConfirmAction {
    // DeleteKey(String, String) - 删除指定的密钥
    // 参数：(密钥名称, 环境字符串)
    DeleteKey(String, String),
    // DeleteGroup(String) - 删除指定的分组
    // 参数：分组ID的字符串表示
    DeleteGroup(String),
    // ResetVault - 重置整个Vault（清空所有加密数据、分组、审计日志）
    ResetVault,
    // _LockVault - 锁定Vault（退出到登录界面），暂未在确认框中使用
    _LockVault,
}

// ==================== VaultApp 实现 ====================
impl VaultApp {
    // new() - 构造函数：创建 VaultApp 实例并初始化所有状态
    // vault_path: PathBuf - Vault 数据库文件的路径
    pub fn new(vault_path: PathBuf) -> Self {
        // 加载应用配置（从 config.toml 文件读取）
        let mut config = AppConfig::load();
        // 覆盖配置中的 vault_path 为传入的路径
        config.vault_path = vault_path;
        // 创建 Vault 核心实例（此时还没解锁，只是加载配置）
        let vault = Vault::new(config);
        // 检查 Vault 是否已初始化（数据库文件是否存在并已初始化）
        let is_initialized = vault.is_initialized();
        // 设置初始视图：无论是否初始化，都显示登录页面
        // （已初始化=需要密码解锁，未初始化=需要设置主密码）
        let initial_state = if is_initialized {
            View::Login
        } else {
            View::Login
        };

        // 返回完整的 VaultApp 实例，所有字段初始化为默认值
        Self {
            vault,                        // Vault 核心实例
            current_view: initial_state,  // 初始页面为登录页
            previous_view: View::Login,   // 上一个页面也是登录页

            // 登录相关字段全部清空
            password_input: String::new(),
            password_confirm: String::new(),
            show_password: false,
            login_error: None,
            _password_strength: None,

            // 密钥列表相关字段初始化
            key_list: Vec::new(),
            key_search_query: String::new(),
            key_filter_env: String::new(),
            _key_filter_group: String::new(),
            key_sort_column: 0,         // 默认按第一列（名称）排序
            key_sort_ascending: true,   // 默认升序排列

            // 密钥详情初始化为空
            decrypted_value: None,
            show_decrypted_value: false,
            selected_key_index: None,

            // 编辑表单使用默认值（新建模式）
            edit_form: KeyEditForm::default(),
            edit_is_new: true,

            // 分组、审计日志、搜索结果都为空
            group_list: Vec::new(),
            audit_logs: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),

            // 导入导出默认格式为csv
            import_format: "csv".to_string(),
            import_file_path: String::new(),
            export_format: "csv".to_string(),
            export_file_path: String::new(),

            // 设置默认值
            settings_auto_lock: 15,               // 默认15分钟自动锁定
            settings_clipboard_clear: 30,          // 默认30秒清除剪贴板
            settings_theme: "dark".to_string(),    // 默认深色主题
            settings_default_env: "development".to_string(), // 默认开发环境
            settings_audit_enabled: true,          // 默认启用审计日志
            new_password: String::new(),
            new_password_confirm: String::new(),
            old_password: String::new(),
            change_password_error: None,
            change_password_success: false,

            // 新建分组输入框为空
            new_group_name: String::new(),
            new_group_error: None,

            // 通知队列为空
            notifications: Vec::new(),

            // 初始化状态标记
            is_initialized,
            sidebar_collapsed: false,              // 侧边栏默认展开
            _last_interaction: Some(Utc::now()),   // 记录当前时间为上次交互时间
            confirm_dialog: None,                  // 无确认对话框
        }
    }

    // ==================== 数据刷新方法 ====================

    // refresh_data() - 刷新所有数据（密钥列表、分组、审计日志）
    // 在切换视图或执行修改操作后调用，确保UI显示最新数据
    fn refresh_data(&mut self) {
        // 先检查Vault是否应该自动锁定（比较当前时间与上次交互时间）
        self.vault.check_auto_lock();
        // 如果Vault状态不是Unlocked（已解锁），则跳转到登录页
        if *self.vault.state() != VaultState::Unlocked {
            self.current_view = View::Login;
            self.login_error = Some("Vault 已自动锁定，请重新输入密码".to_string());
            return; // 不再继续刷新数据
        }

        // 刷新密钥列表：从Vault获取所有密钥
        if let Ok(keys) = self.vault.list_keys() {
            self.key_list = keys;
        }
        // 刷新分组列表：从Vault获取所有分组
        if let Ok(groups) = self.vault.list_groups() {
            self.group_list = groups;
        }
        // 刷新审计日志：获取最近50条操作记录
        if let Ok(logs) = self.vault.get_audit_logs(50) {
            self.audit_logs = logs;
        }
    }

    // refresh_keys() - 仅刷新密钥列表
    // 密钥操作（增/删/改）后调用，比 refresh_data() 更轻量
    fn refresh_keys(&mut self) {
        if let Ok(keys) = self.vault.list_keys() {
            self.key_list = keys;
        }
    }

    // refresh_audit_logs() - 仅刷新审计日志列表
    fn refresh_audit_logs(&mut self) {
        if let Ok(logs) = self.vault.get_audit_logs(50) {
            self.audit_logs = logs;
        }
    }

    // refresh_groups() - 仅刷新分组列表
    fn refresh_groups(&mut self) {
        if let Ok(groups) = self.vault.list_groups() {
            self.group_list = groups;
        }
    }

    // ==================== 通知管理方法 ====================

    // add_notification() - 向通知队列中添加一条通知
    fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    // cleanup_notifications() - 清理已过期的通知
    // 保留所有未过期的通知，移除已过期的
    fn cleanup_notifications(&mut self) {
        // retain() 方法：只保留闭包返回 true 的元素
        // |n| !n.is_expired() 表示只保留未过期的通知
        self.notifications.retain(|n| !n.is_expired());
    }

    // ==================== 剪贴板操作方法 ====================

    // copy_to_clipboard() - 将文本复制到系统剪贴板
    // 使用 arboard 库访问系统剪贴板（跨平台）
    // 操作结果通过通知反馈给用户
    fn copy_to_clipboard(&mut self, text: &str) {
        // arboard::Clipboard::new() 创建剪贴板实例（可能失败）
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                // clipboard.set_text() 将文本设置到剪贴板
                if let Err(e) = clipboard.set_text(text.to_string()) {
                    // 复制失败，显示错误通知
                    self.add_notification(Notification::error(format!("复制到剪贴板失败: {}", e)));
                } else {
                    // 复制成功，显示成功通知
                    self.add_notification(Notification::success("已复制到剪贴板"));
                }
            }
            Err(e) => {
                // 无法获取剪贴板实例（可能是系统限制或无剪贴板）
                self.add_notification(Notification::error(format!("无法访问剪贴板: {}", e)));
            }
        }
    }

    // ==================== 导航方法 ====================

    // navigate_to() - 切换到指定视图
    // view: View - 要切换到的目标视图
    // 切换前保存当前视图到 previous_view（用于返回功能）
    // 切换后根据目标视图类型自动刷新所需的数据
    fn navigate_to(&mut self, view: View) {
        // 保存当前视图作为"上一个视图"
        self.previous_view = self.current_view.clone();
        // 切换到目标视图
        self.current_view = view;

        // 根据目标视图类型，刷新对应的数据
        match &self.current_view {
            View::Dashboard | View::KeyList => {
                // 仪表板和密钥列表需要所有数据
                self.refresh_data();
            }
            View::GroupList => {
                // 分组管理只需刷新分组列表
                self.refresh_groups();
            }
            View::AuditLog => {
                // 审计日志只需刷新日志列表
                self.refresh_audit_logs();
            }
            View::KeyEdit(_) => {
                // 编辑页面需要分组列表来填充分组下拉框
                self.refresh_groups();
            }
            _ => {} // 其他视图不需要刷新数据
        }
    }

    // ==================== 样式设置方法 ====================

    // _setup_style() - 设置egui的全局样式（当前暂未使用）
    // ctx: &egui::Context - egui的上下文，用于设置全局样式
    // theme: &ThemeColors - 当前主题颜色
    fn _setup_style(ctx: &egui::Context, theme: &ThemeColors) {
        // 获取当前样式并克隆（避免修改全局默认样式）
        let mut style = (*ctx.style()).clone();
        // 设置窗口填充色为主背景色
        style.visuals.window_fill = theme.bg_primary;
        // 设置面板填充色为主背景色
        style.visuals.panel_fill = theme.bg_primary;
        // 应用修改后的样式
        ctx.set_style(style);
    }

    // ==================== 侧边栏渲染 ====================

    // show_sidebar() - 渲染左侧导航侧边栏
    // ui: &mut egui::Ui - egui 的 UI 实例，所有UI组件通过它渲染
    // theme: &ThemeColors - 当前主题颜色引用
    fn show_sidebar(&mut self, ui: &mut egui::Ui, theme: &ThemeColors) {
        // 计算侧边栏宽度：折叠时56px，展开时200px
        // 当前只用于注释，实际宽度在 eframe::SidePanel::exact_width 中设置
        let _sidebar_width = if self.sidebar_collapsed { 56.0 } else { 200.0 };

        // vertical() 创建一个垂直布局容器，所有子元素垂直排列
        ui.vertical(|ui| {
            // ===== Logo/标题区域 =====
            ui.add_space(12.0);  // 顶部留白
            ui.horizontal(|ui| { // 水平布局放图标和标题
                ui.add_space(12.0);  // 左侧留白
                let icon_text = RichText::new("🔒").size(22.0);  // 🔒 锁图标，字号22
                ui.label(icon_text);
                if !self.sidebar_collapsed {
                    // 侧边栏展开时显示应用标题
                    ui.label(RichText::new("API Key Vault").size(16.0).strong().color(theme.accent));
                }
            });
            ui.add_space(16.0);  // 与下方折叠按钮的间距

            // ===== 侧边栏折叠/展开按钮 =====
            // 点击切换 sidebar_collapsed 状态
            if ui.add(egui::Button::new(
                if self.sidebar_collapsed {
                    // 折叠时显示 ▶（表示可以展开）
                    RichText::new("▶").size(14.0).color(theme.text_secondary)
                } else {
                    // 展开时显示 ◀（表示可以折叠）
                    RichText::new("◀").size(14.0).color(theme.text_secondary)
                }
            ).frame(false)).clicked() {
                self.sidebar_collapsed = !self.sidebar_collapsed;  // 切换折叠状态
            }
            ui.add_space(8.0);

            // ===== 分隔线 =====
            // 使用 painter 直接绘制一条水平线
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            // line_segment() 在两点之间画线
            painter.line_segment(
                [egui::pos2(rect.left() + 12.0, ui.cursor().top()),  // 起点：左侧+12px偏移
                 egui::pos2(rect.right() - 12.0, ui.cursor().top())], // 终点：右侧-12px偏移
                Stroke::new(1.0, theme.border),  // 线宽1px，颜色为边框色
            );
            ui.add_space(8.0);

            // ===== 导航菜单项 =====
            // 定义导航项列表：(对应的View枚举, 图标emoji, 中文标签)
            let nav_items: Vec<(View, &str, &str)> = vec![
                (View::Dashboard, "📊", "仪表板"),   // 仪表板
                (View::KeyList, "🔑", "密钥管理"),   // 密钥管理
                (View::GroupList, "📁", "分组管理"), // 分组管理
                (View::Search, "🔍", "搜索"),        // 搜索
                (View::AuditLog, "📋", "审计日志"),  // 审计日志
                (View::ImportExport, "📦", "导入导出"), // 导入导出
                (View::Settings, "⚙", "设置"),       // 设置
            ];

            // 遍历并渲染每个导航项
            for (view, icon, label) in nav_items {
                // 判断当前项是否激活（与 current_view 的类型是否匹配）
                // 使用 std::mem::discriminant 比较枚举的变体（忽略参数值）
                let is_active = std::mem::discriminant(&self.current_view) == std::mem::discriminant(&view);

                // 根据折叠状态决定显示内容
                let btn_text = if self.sidebar_collapsed {
                    RichText::new(icon).size(20.0)  // 折叠时只显示图标
                } else {
                    RichText::new(format!("{}  {}", icon, label)).size(14.0)  // 展开时显示 图标+标签
                };

                // 根据激活状态设置按钮样式
                let btn = if is_active {
                    // 激活状态：主题色文字 + 高亮背景
                    egui::Button::new(btn_text.color(theme.accent))
                        .fill(Color32::from_rgb(40, 38, 65))      // 深紫灰色背景
                        .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0))
                        .rounding(Rounding::same(6.0))           // 6px圆角
                } else {
                    // 非激活状态：次要色文字 + 透明背景
                    egui::Button::new(btn_text.color(theme.text_secondary))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0))
                        .rounding(Rounding::same(6.0))
                };

                // 渲染按钮并处理点击事件
                let resp = ui.add(btn);
                if resp.clicked() {
                    self.navigate_to(view);  // 点击后导航到对应视图
                }

                // Hover 效果：鼠标悬停时背景变亮
                if resp.hovered() && !is_active {
                    let style = ui.style_mut();
                    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 33, 55);
                }
            }

            // ===== 底部锁定按钮 =====
            // 使用 bottom_up 布局将锁定按钮固定在侧边栏底部
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                // 根据折叠状态显示不同内容
                let lock_text = if self.sidebar_collapsed {
                    RichText::new("🔓").size(18.0)  // 折叠时只显示图标
                } else {
                    RichText::new("🔓  锁定 Vault").size(14.0).color(theme.warning)  // 展开时显示文字
                };
                let lock_btn = egui::Button::new(lock_text)
                    .fill(Color32::TRANSPARENT)
                    .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0));
                if ui.add(lock_btn).clicked() {
                    // 点击锁定：调用 vault.lock() 锁定Vault，跳转到登录页
                    self.vault.lock();
                    self.current_view = View::Login;
                    self.password_input.clear();  // 清空密码输入
                    self.login_error = None;       // 清除登录错误
                }
                ui.add_space(8.0);
            });
        });
    }

    // ==================== 状态栏渲染 ====================

    // show_status_bar() - 渲染底部状态栏
    // 显示Vault状态、密钥数量、分组数量、自动锁定信息
    fn show_status_bar(&mut self, ui: &mut egui::Ui, theme: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // ===== Vault 状态指示器 =====
            let state_text = match self.vault.state() {
                VaultState::Uninitialized => "⚪ 未初始化",  // 白点表示未初始化
                VaultState::Locked => "🔴 已锁定",           // 红点表示已锁定
                VaultState::Unlocked => "🟢 已解锁",         // 绿点表示已解锁
            };
            ui.label(RichText::new(state_text).size(11.0).color(theme.text_dim));

            ui.add_space(16.0);

            // ===== 密钥和分组数量统计 =====
            // 仅在解锁状态下显示，因为锁定后无法获取数据
            if *self.vault.state() == VaultState::Unlocked {
                ui.label(RichText::new(format!("🔑 {} 个密钥", self.key_list.len())).size(11.0).color(theme.text_dim));
                ui.add_space(16.0);
                ui.label(RichText::new(format!("📁 {} 个分组", self.group_list.len())).size(11.0).color(theme.text_dim));
            }

            // ===== 右侧内容（右对齐）=====
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                // 显示自动锁定时间
                if *self.vault.state() == VaultState::Unlocked {
                    let auto_lock = self.vault.config().auto_lock_minutes;
                    ui.label(RichText::new(format!("自动锁定: {} 分钟", auto_lock)).size(11.0).color(theme.text_dim));
                }
            });
        });
    }

    // ==================== 通知渲染 ====================

    // show_notifications() - 在屏幕右上角渲染浮动通知
    // 使用 egui::Area 创建不受布局限制的浮动层
    // 每个通知作为一个独立的 Area，从右上角向下排列
    fn show_notifications(&mut self, ctx: &egui::Context) {
        // 先清理已过期的通知
        self.cleanup_notifications();

        // 如果没有通知，直接返回，不浪费渲染
        if self.notifications.is_empty() {
            return;
        }

        let _screen_rect = ctx.screen_rect();  // 获取屏幕尺寸（暂未使用）
        // 根据当前主题设置选择对应的配色
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        // 遍历所有通知，从最新的开始渲染
        for (i, notification) in self.notifications.iter().enumerate() {
            // 垂直偏移：每条通知间隔50px
            let y_offset = 16.0 + (i as f32) * 50.0;
            // 根据通知类型选择背景色
            let bg_color = if notification.is_error {
                Color32::from_rgb(60, 20, 20)  // 错误：深红色背景
            } else {
                Color32::from_rgb(20, 50, 30)  // 成功：深绿色背景
            };
            // 根据通知类型选择边框色
            let border_color = if notification.is_error { theme.error } else { theme.success };

            // 使用 egui::Area 创建浮动层（不参与常规布局）
            egui::Area::new(egui::Id::new(format!("notification_{}", i)))  // 每个Area需要唯一Id
                .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-16.0, y_offset))  // 固定在右上角，带偏移
                .show(ctx, |ui| {
                    // 使用 Frame::none() 自定义背景和边框
                    egui::Frame::none()
                        .fill(bg_color)
                        .stroke(Stroke::new(1.0, border_color))  // 1px边框
                        .rounding(Rounding::same(8.0))           // 8px圆角
                        .inner_margin(egui::Margin::symmetric(16.0, 10.0))  // 左右16px上下10px内边距
                        .show(ui, |ui| {
                            // 显示图标和消息文本
                            let icon = if notification.is_error { "❌" } else { "✅" };
                            ui.label(RichText::new(format!("{} {}", icon, notification.message)).size(13.0).color(theme.text_primary));
                        });
                });
        }
    }

    // ==================== 确认对话框 ====================

    // show_confirm_dialog() - 显示确认对话框（模态窗口）
    // 用于危险操作前的二次确认，如删除密钥、重置Vault等
    fn show_confirm_dialog(&mut self, ctx: &egui::Context) {
        // 克隆确认对话框的状态，避免借用冲突
        let dialog = match self.confirm_dialog.clone() {
            Some(d) => d,   // 有对话框则显示
            None => return, // 无对话框则直接返回
        };

        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        let mut close = false;      // 是否关闭对话框
        let mut confirmed = false;  // 用户是否点击了"确认"

        // 使用 egui::Window 创建模态窗口
        egui::Window::new(&dialog.title)  // 窗口标题使用对话框的title
            .collapsible(false)            // 不可折叠
            .resizable(false)              // 不可调整大小
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)  // 屏幕居中
            .fixed_size(Vec2::new(380.0, 160.0))  // 固定窗口大小（宽380，高160）
            .show(ctx, |ui| {
                ui.add_space(8.0);
                // 显示确认消息
                ui.label(RichText::new(&dialog.message).size(14.0).color(theme.text_primary));
                ui.add_space(16.0);
                // 按钮行：取消 | 确认
                ui.horizontal(|ui| {
                    // "取消"按钮（次要操作）
                    if ui.add(
                        egui::Button::new(RichText::new("取消").size(13.0))
                            .min_size(Vec2::new(80.0, 32.0))
                    ).clicked() {
                        close = true;  // 关闭对话框，不执行操作
                    }
                    ui.add_space(8.0);
                    // "确认"按钮（危险操作，红色背景）
                    if ui.add(
                        egui::Button::new(RichText::new("确认").size(13.0).color(Color32::WHITE))
                            .fill(theme.danger)       // 危险红色按钮
                            .min_size(Vec2::new(80.0, 32.0))
                            .rounding(Rounding::same(4.0))  // 4px圆角
                    ).clicked() {
                        confirmed = true;  // 用户确认
                        close = true;      // 关闭对话框
                    }
                });
            });

        // 对话框关闭后的处理
        if close {
            // 取出要执行的操作（确认后执行的 Action）
            let action = self.confirm_dialog.as_ref().map(|d| d.on_confirm_action.clone());
            self.confirm_dialog = None;  // 清空对话框状态

            if confirmed {
                if let Some(action) = action {
                    self.execute_confirm_action(action);  // 执行确认后的操作
                }
            }
        }
    }

    // execute_confirm_action() - 执行确认对话框确定后的操作
    // action: ConfirmAction - 要执行的枚举操作
    fn execute_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            // ===== 删除密钥 =====
            ConfirmAction::DeleteKey(name, env) => {
                match self.vault.delete_key(&name, &env) {
                    Ok(()) => {
                        self.add_notification(Notification::success(format!("密钥 '{}' 已删除", name)));
                        self.refresh_keys();  // 刷新密钥列表
                        // 如果当前在详情页，则跳转回列表页
                        if matches!(self.current_view, View::KeyDetail(_)) {
                            self.current_view = View::KeyList;
                        }
                    }
                    Err(e) => {
                        self.add_notification(Notification::error(format!("删除密钥失败: {}", e)));
                    }
                }
            }
            // ===== 删除分组 =====
            ConfirmAction::DeleteGroup(id_str) => {
                if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                    match self.vault.delete_group(&id) {
                        Ok(()) => {
                            self.add_notification(Notification::success("分组已删除"));
                            self.refresh_groups();
                        }
                        Err(e) => {
                            self.add_notification(Notification::error(format!("删除分组失败: {}", e)));
                        }
                    }
                }
            }
            // ===== 重置Vault =====
            ConfirmAction::ResetVault => {
                match self.vault.reset() {
                    Ok(()) => {
                        self.add_notification(Notification::success("Vault 已重置"));
                        self.is_initialized = false;  // 标记为未初始化
                        self.current_view = View::Login;  // 跳转到登录页
                        self.password_input.clear();
                        // 清空所有缓存数据
                        self.key_list.clear();
                        self.group_list.clear();
                        self.audit_logs.clear();
                    }
                    Err(e) => {
                        self.add_notification(Notification::error(format!("重置失败: {}", e)));
                    }
                }
            }
            // ===== 锁定Vault（暂未在前端对话框中使用）=====
            ConfirmAction::_LockVault => {
                self.vault.lock();
                self.current_view = View::Login;
                self.password_input.clear();
                self.login_error = None;
            }
        }
    }

    // ==================== 登录/初始化视图 ====================

    // show_login_view() - 渲染登录或初始化页面
    // 已初始化：显示密码输入框和解锁按钮
    // 未初始化：显示密码设置、确认密码和初始化按钮，带密码强度指示器
    fn show_login_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        let screen_rect = ctx.screen_rect();  // 获取屏幕区域
        let center = screen_rect.center();     // 计算屏幕中心点

        // ===== 登录面板尺寸 =====
        let panel_width = 400.0;                        // 面板宽400px
        let panel_height = if self.is_initialized { 300.0 } else { 380.0 };  // 已初始化=300px，未初始化需更多空间=380px
        let panel_rect = egui::Rect::from_center_size(
            center,
            Vec2::new(panel_width, panel_height),
        );

        // 在面板矩形区域内创建新的UI上下文
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(panel_rect), |ui| {
            ui.vertical_centered(|ui| {  // 垂直居中布局
                ui.add_space(20.0);

                // ===== 标题区域 =====
                ui.label(RichText::new("🔒").size(48.0));  // 大锁图标
                ui.add_space(8.0);
                ui.label(RichText::new("API Key Vault").size(28.0).strong().color(theme.accent));
                ui.add_space(4.0);

                // 根据初始化状态显示不同提示文字
                if self.is_initialized {
                    ui.label(RichText::new("输入主密码以解锁 Vault").size(14.0).color(theme.text_secondary));
                } else {
                    ui.label(RichText::new("首次使用，请设置主密码").size(14.0).color(theme.text_secondary));
                }

                ui.add_space(24.0);

                // ===== 密码输入框 =====
                let password_width = 300.0;  // 输入框宽度
                ui.horizontal(|ui| {
                    ui.add_space((panel_width - password_width) / 2.0);  // 水平居中偏移
                    // 密码输入框（可切换密码/明文模式）
                    let text_edit = egui::TextEdit::singleline(&mut self.password_input)
                        .password(!self.show_password)      // show_password为false时显示为密码
                        .desired_width(password_width)  // 宽度减去眼睛按钮的空间
                        .hint_text("主密码")
                        .font(FontId::new(20.0, FontFamily::Proportional));
                    ui.add(text_edit);

                    // 密码可见性切换按钮（眼睛图标）
                    let eye_icon = if self.show_password { "🙉" } else { "🙈" };
                    if ui.add(
                        egui::Button::new(RichText::new(eye_icon).size(24.0))
                            .fill(Color32::TRANSPARENT)
                            .frame(false)
                    ).clicked() {
                        self.show_password = !self.show_password;  // 切换显示/隐藏密码
                    }
                });

                // ===== 密码强度指示器（未初始化时或已有输入时显示）=====
                if !self.password_input.is_empty() {
                    // 使用 zxcvbn 库计算密码强度
                    let strength = calculate_password_strength(&self.password_input);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        // 根据强度分数映射为 等级、标签、颜色
                        let (score, label, color) = match strength {
                            0 => (0.2, "非常弱", theme.error),
                            1 => (0.4, "弱", Color32::from_rgb(230, 126, 34)),   // 橙色
                            2 => (0.6, "中等", theme.warning),                      // 黄色
                            3 => (0.8, "强", Color32::from_rgb(39, 174, 96)),       // 绿色
                            _ => (1.0, "非常强", theme.success),                    // 翠绿色
                        };

                        // 绘制强度进度条
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(password_width, 8.0), egui::Sense::hover());
                        let bg_rect = rect;
                        // 首先画灰色背景条
                        ui.painter().rect_filled(bg_rect, Rounding::same(4.0), theme.bg_input);
                        // 然后根据强度分数画填充条
                        let fill_rect = egui::Rect::from_min_size(
                            bg_rect.min,
                            Vec2::new(bg_rect.width() * score, bg_rect.height()),
                        );
                        ui.painter().rect_filled(fill_rect, Rounding::same(4.0), color);

                        ui.add_space(4.0);
                        ui.label(RichText::new(label).size(11.0).color(color));  // 强度等级文字
                    });
                }

                // ===== 密码确认输入框（仅初始化时显示）=====
                if !self.is_initialized {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0 - 10.0);
                        let text_edit = egui::TextEdit::singleline(&mut self.password_confirm)
                            .password(!self.show_password)
                            .desired_width(password_width - 40.0)
                            .hint_text("确认密码")
                            .font(FontId::new(16.0, FontFamily::Proportional));
                        ui.add(text_edit);
                    });
                }

                // ===== 错误信息显示 =====
                if let Some(ref error) = self.login_error {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        ui.label(RichText::new(format!("⚠ {}", error)).size(12.0).color(theme.error));
                    });
                }

                ui.add_space(20.0);

                // ===== 主要按钮 =====
                let btn_width = 300.0;
                if self.is_initialized {
                    // ---------- 解锁模式 ----------
                    let unlock_btn = egui::Button::new(
                        RichText::new("🔓  解锁").size(16.0).color(Color32::WHITE)
                    )
                        .fill(theme.accent)       // 主题色背景
                        .min_size(Vec2::new(btn_width, 44.0))  // 大按钮尺寸
                        .rounding(Rounding::same(8.0));

                    // 点击按钮或按下回车键触发解锁
                    let resp = ui.add(unlock_btn);
                    if resp.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.password_input.is_empty() {
                            self.login_error = Some("请输入密码".to_string());
                        } else {
                            match self.vault.unlock(&self.password_input) {
                                Ok(()) => {
                                    self.login_error = None;
                                    self.password_input.clear();
                                    self.navigate_to(View::Dashboard);  // 解锁成功后进入仪表板
                                }
                                Err(e) => {
                                    self.login_error = Some(format!("解锁失败: {}", e));
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                    // "重置 Vault" 链接（危险操作，需要二次确认）
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        if ui.add(
                            egui::Button::new(RichText::new("重置 Vault").size(12.0).color(theme.error))
                                .fill(Color32::TRANSPARENT)
                                .frame(false)  // 无边框，显示为链接样式
                        ).clicked() {
                            // 显示确认对话框
                            self.confirm_dialog = Some(ConfirmDialog {
                                title: "重置 Vault".to_string(),
                                message: "确定要重置 Vault 吗？这将删除所有数据，此操作不可恢复！".to_string(),
                                on_confirm_action: ConfirmAction::ResetVault,
                            });
                        }
                    });
                } else {
                    // ---------- 初始化模式 ----------
                    let init_btn = egui::Button::new(
                        RichText::new("🚀  初始化 Vault").size(16.0).color(Color32::WHITE)
                    )
                        .fill(theme.accent)
                        .min_size(Vec2::new(btn_width, 44.0))
                        .rounding(Rounding::same(8.0));

                    if ui.add(init_btn).clicked() {
                        // 验证流程
                        if self.password_input.is_empty() {
                            self.login_error = Some("请输入密码".to_string());
                        } else if self.password_input != self.password_confirm {
                            self.login_error = Some("两次密码不一致".to_string());
                        } else if self.password_input.len() < 8 {
                            self.login_error = Some("密码至少需要 8 个字符".to_string());
                        } else {
                            match self.vault.init(&self.password_input) {
                                Ok(()) => {
                                    self.login_error = None;
                                    self.is_initialized = true;
                                    self.password_input.clear();
                                    self.password_confirm.clear();
                                    self.navigate_to(View::Dashboard);
                                    self.add_notification(Notification::success("Vault 初始化成功！"));
                                }
                                Err(e) => {
                                    self.login_error = Some(format!("初始化失败: {}", e));
                                }
                            }
                        }
                    }
                }
            });
        });
    }

    // ==================== Dashboard 视图 ====================

    // show_dashboard_view() - 渲染仪表板页面
    // 显示统计概览卡片、环境分布、提供商分布、快捷操作和最近操作记录
    fn show_dashboard_view(&mut self, ui: &mut egui::Ui) {
        // 根据当前主题设置选择配色
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // ===== 页面标题 =====
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊 仪表板").size(22.0).strong().color(theme.text_primary));
            });
            ui.add_space(16.0);

            // ===== 统计卡片行（4个指标卡片）=====
            let total_keys = self.key_list.len();
            let total_groups = self.group_list.len();
            let total_logs = self.audit_logs.len();
            // 收集所有不同的提供商，计算提供商数量
            let providers: std::collections::HashSet<String> = self.key_list.iter().map(|k| k.provider.clone()).collect();

            // 计算每个卡片的宽度（总宽度减去3个间隙略多一点保持美观，再除以4）
            let card_width = (ui.available_width() - 72.0) / 4.0;

            // 水平排列4个统计卡片
            ui.horizontal(|ui| {
                self.show_stat_card(ui, &theme, "🔑", "密钥总数", &total_keys.to_string(), card_width);
                ui.add_space(16.0);
                self.show_stat_card(ui, &theme, "📁", "分组总数", &total_groups.to_string(), card_width);
                ui.add_space(16.0);
                self.show_stat_card(ui, &theme, "🏢", "提供商数", &providers.len().to_string(), card_width);
                ui.add_space(16.0);
                self.show_stat_card(ui, &theme, "📋", "操作记录", &total_logs.to_string(), card_width);
            });

            ui.add_space(20.0);

            // ===== 环境统计和提供商统计并排 =====
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) / 2.0;  // 各占一半宽度
                self.show_env_stats_card(ui, &theme, half_width);      // 环境分布柱状图
                ui.add_space(16.0);
                self.show_provider_stats_card(ui, &theme, half_width); // 提供商分布柱状图
            });

            ui.add_space(20.0);

            // ===== 快捷操作和最近日志（纵向排列）=====
            // 快捷操作面板上方留白内边距
            let panel_inner_width = (ui.available_width() - 32.0).max(40.0);

            // ===== 上：快捷操作面板 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.set_min_width(panel_inner_width);
                    ui.label(RichText::new("⚡ 快捷操作").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(12.0);

                    // 三个快捷按钮：添加密钥、搜索、导入
                    ui.horizontal(|ui| {
                        // "添加密钥"按钮（主题色突出显示）
                        if ui.add(
                            egui::Button::new(RichText::new("➕ 添加密钥").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(120.0, 36.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.edit_form = KeyEditForm::default();  // 清空表单
                            self.edit_is_new = true;                   // 标记为新建模式
                            self.navigate_to(View::KeyEdit(None));
                        }
                        ui.add_space(8.0);
                        // "搜索"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("🔍 搜索").size(13.0).color(theme.text_primary))
                                .fill(theme.bg_input)
                                .min_size(Vec2::new(120.0, 36.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.navigate_to(View::Search);
                        }
                        ui.add_space(8.0);
                        // "导入"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("📦 导入").size(13.0).color(theme.text_primary))
                                .fill(theme.bg_input)
                                .min_size(Vec2::new(120.0, 36.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.navigate_to(View::ImportExport);
                        }
                    });
                });

            ui.add_space(16.0);

            // ===== 下：最近审计日志面板 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.set_min_width(panel_inner_width);
                    ui.label(RichText::new("📋 最近操作").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

                    // 取前5条最近的操作记录
                    let recent_logs: Vec<_> = self.audit_logs.iter().take(5).collect();
                    if recent_logs.is_empty() {
                        ui.label(RichText::new("暂无操作记录").size(13.0).color(theme.text_dim));
                    } else {
                        // 计算最近操作面板内的可用宽度分配：
                        // 可用总宽 = panel_inner_width，减去图标(20px)、时间戳(65px)、间距(20px)后给操作名称
                        let action_w = (panel_inner_width - 105.0).max(60.0);
                        let time_w = 65.0;

                        for log in &recent_logs {
                            ui.horizontal(|ui| {
                                // 根据操作类型显示对应的图标
                                let action_icon = match log.action {
                                    AuditAction::KeyCreated => "➕",
                                    AuditAction::KeyViewed => "👁",
                                    AuditAction::KeyUpdated => "✏",
                                    AuditAction::KeyDeleted => "🗑",
                                    AuditAction::KeyRotated => "🔄",
                                    AuditAction::KeyCopied => "📋",
                                    AuditAction::VaultUnlocked => "🔓",
                                    AuditAction::VaultLocked => "🔒",
                                    _ => "•",  // 其他操作使用圆点
                                };
                                ui.label(RichText::new(action_icon).size(12.0));
                                // 操作名称（限制宽度，防止撑出面板）
                                ui.add_sized(
                                    Vec2::new(action_w, 18.0),
                                    egui::Label::new(
                                        RichText::new(format!("{}", log.action))
                                            .size(12.0)
                                            .color(theme.text_secondary),
                                    ),
                                );
                                // 时间戳格式化为 "月-日 时:分"（限制宽度，右对齐）
                                ui.add_sized(
                                    Vec2::new(time_w, 18.0),
                                    egui::Label::new(
                                        RichText::new(log.timestamp.format("%m-%d %H:%M").to_string())
                                            .size(11.0)
                                            .color(theme.text_dim),
                                    ),
                                );
                            });
                        }
                    }
                });
        });
    }

    // show_stat_card() - 渲染单个统计卡片
    // 用于 Dashboard 顶部的4个统计指标
    // total_width: f32 - 卡片的**总宽度**（含左右内边距），由调用方根据可用宽度均匀分配
    //                 注意：Frame 的 inner_margin(16.0) 使左右各有16px内边距（合计32px）
    //                 所以实际内容区宽度 = total_width - 32.0，该值传给 set_min_width 以确保内容不溢出
    fn show_stat_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, icon: &str, title: &str, value: &str, total_width: f32) {
        // 减去左右内边距（16 + 16 = 32），得到实际内容区最小宽度
        let inner_width = (total_width - 32.0).max(40.0);

        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)                       // 左右各16px内边距，合计32px
            .show(ui, |ui| {
                ui.set_min_width(inner_width);        // 设置内容区最小宽度（减去内边距后的值）
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(20.0));     // 图标（大号）
                        ui.label(RichText::new(title).size(13.0).color(theme.text_secondary));  // 标题
                    });
                    ui.add_space(4.0);
                    // 数值（大号、粗体、主题色）
                    ui.label(RichText::new(value).size(28.0).strong().color(theme.accent));
                });
            });
    }

    // show_env_stats_card() - 渲染环境分布统计卡片
    // 以水平柱状图展示各个环境（development/staging/production等）的密钥数量
    // total_width: f32 - 卡片总宽度（含左右内边距），左右各16px合计32px，内容区 = total_width - 32
    fn show_env_stats_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, total_width: f32) {
        // 减去左右内边距（16 + 16 = 32），得到实际内容区最小宽度
        let inner_width = (total_width - 32.0).max(40.0);

        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(inner_width);
                ui.label(RichText::new("🌍 环境分布").size(15.0).strong().color(theme.text_primary));
                ui.add_space(8.0);

                // 统计每个环境的密钥数量
                let mut env_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    let env = key.environment.to_string();
                    *env_counts.entry(env).or_insert(0) += 1;  // 计数+1
                }

                if env_counts.is_empty() {
                    ui.label(RichText::new("暂无密钥").size(13.0).color(theme.text_dim));
                } else {
                    let max_count = env_counts.values().max().copied().unwrap_or(1);  // 最大数量（用于归一化）
                    let mut envs: Vec<_> = env_counts.into_iter().collect();
                    envs.sort_by(|a, b| b.1.cmp(&a.1));  // 按数量从大到小排序

                    for (env, count) in envs {
                        ui.horizontal(|ui| {
                            let env_label_width = 100.0;
                            // 环境名称（等宽字体）
                            ui.label(RichText::new(&env).size(12.0).color(theme.text_secondary).family(FontFamily::Monospace));
                            ui.add_space(8.0);

                            // 柱状图
                            let bar_width = (inner_width - env_label_width - 60.0).max(50.0);
                            let fraction = count as f32 / max_count as f32;  // 计算占比
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), egui::Sense::hover());
                            // 背景条
                            ui.painter().rect_filled(rect, Rounding::same(3.0), theme.bg_input);
                            // 填充条（根据占比绘制）
                            let fill_rect = egui::Rect::from_min_size(
                                rect.min,
                                Vec2::new(rect.width() * fraction, rect.height()),
                            );
                            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), theme.accent);

                            // 数量数字
                            ui.label(RichText::new(count.to_string()).size(12.0).color(theme.text_primary));
                        });
                        ui.add_space(4.0);
                    }
                }
            });
    }

    // show_provider_stats_card() - 渲染提供商分布统计卡片
    // 以水平柱状图展示各个提供商的密钥数量（最多显示前8个）
    // total_width: f32 - 卡片总宽度（含左右内边距），左右各16px合计32px，内容区 = total_width - 32，为了美观，多减了7px，实际内容区 = total_width - 39
    fn show_provider_stats_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, total_width: f32) {
        let inner_width = (total_width - 39.0).max(40.0);

        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(inner_width);
                ui.label(RichText::new("🏢 提供商分布").size(15.0).strong().color(theme.text_primary));
                ui.add_space(8.0);

                // 统计每个提供商的密钥数量
                let mut provider_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    *provider_counts.entry(key.provider.clone()).or_insert(0) += 1;
                }

                if provider_counts.is_empty() {
                    ui.label(RichText::new("暂无密钥").size(13.0).color(theme.text_dim));
                } else {
                    let max_count = provider_counts.values().max().copied().unwrap_or(1);
                    let mut providers: Vec<_> = provider_counts.into_iter().collect();
                    providers.sort_by(|a, b| b.1.cmp(&a.1));  // 按数量降序

                    // 只显示前8个提供商（避免面板过长）
                    for (provider, count) in providers.iter().take(8) {
                        ui.horizontal(|ui| {
                            let label_width = 100.0f32;
                            ui.label(RichText::new(provider).size(12.0).color(theme.text_secondary));
                            ui.add_space(8.0);

                            // 柱状图（使用绿色主题而不是accent色，以示区别）
                            let bar_width = (inner_width - label_width - 60.0).max(50.0);
                            let fraction = *count as f32 / max_count as f32;
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, Rounding::same(3.0), theme.bg_input);
                            let fill_rect = egui::Rect::from_min_size(
                                rect.min,
                                Vec2::new(rect.width() * fraction, rect.height()),
                            );
                            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), Color32::from_rgb(46, 204, 113)); // 翠绿色

                            // 数量
                            ui.label(RichText::new(count.to_string()).size(12.0).color(theme.text_primary));
                        });
                        ui.add_space(4.0);
                    }
                }
            });
    }

    // ==================== 密钥列表视图 ====================

    // show_key_list_view() - 渲染密钥列表页面
    // 包含搜索/过滤、排序表格、行操作按钮（复制/编辑/删除）
    fn show_key_list_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // ===== 标题行：标题 + 搜索框 + 过滤 + 添加按钮 =====
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔑 密钥管理").size(22.0).strong().color(theme.text_primary));
                ui.add_space(16.0);

                // ===== 搜索输入框（自适应宽度）=====
                let available_w = ui.available_width();
                let search_w = (available_w * 0.35).max(120.0);  // 占剩余宽度的35%，最小120px
                let search_edit = egui::TextEdit::singleline(&mut self.key_search_query)
                    .desired_width(search_w)
                    .hint_text("搜索密钥...");
                ui.add(search_edit);

                // ===== 环境过滤下拉框 =====
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("filter_env")
                    .selected_text(if self.key_filter_env.is_empty() { "所有环境" } else { &self.key_filter_env })
                    .show_ui(ui, |ui| {
                        let mut all_env_clicked = false;
                        // "所有环境"选项（选中时清空过滤条件）
                        if ui.selectable_value(&mut self.key_filter_env, String::new(), "所有环境").clicked() {
                            all_env_clicked = true;
                        }
                        // 预定义的4个环境选项
                        let envs = ["development", "staging", "production", "testing"];
                        for env in &envs {
                            if ui.selectable_value(&mut self.key_filter_env, env.to_string(), *env).clicked() {
                                all_env_clicked = false;
                            }
                        }
                        if all_env_clicked { self.key_filter_env.clear(); }
                    });

                // ===== "添加密钥"按钮（右对齐）=====
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("➕ 添加密钥").size(13.0).color(Color32::WHITE))
                            .fill(theme.accent)
                            .min_size(Vec2::new(120.0, 34.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
                        self.edit_form = KeyEditForm::default();
                        self.edit_is_new = true;
                        self.navigate_to(View::KeyEdit(None));
                    }
                });
            });

            ui.add_space(12.0);

            // ===== 过滤和排序密钥列表 =====
            // 先应用搜索关键词过滤和按环境过滤
            let filtered_keys: Vec<(usize, KeyEntry)> = self.key_list.iter().enumerate().filter(|(_, key)| {
                // 搜索匹配：名称、提供商、描述中包含搜索关键词（不区分大小写）
                let matches_search = self.key_search_query.is_empty()
                    || key.name.to_lowercase().contains(&self.key_search_query.to_lowercase())
                    || key.provider.to_lowercase().contains(&self.key_search_query.to_lowercase())
                    || key.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&self.key_search_query.to_lowercase()));

                // 环境过滤匹配
                let matches_env = self.key_filter_env.is_empty()
                    || key.environment.to_string() == self.key_filter_env;

                matches_search && matches_env
            }).map(|(idx, key)| (idx, key.clone())).collect();

            // ===== 密钥表格渲染 =====
            if filtered_keys.is_empty() {
                // 没有数据时显示空状态提示
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🔑").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    if self.key_list.is_empty() {
                        // Vault中没有任何密钥
                        ui.label(RichText::new("暂无密钥").size(16.0).color(theme.text_dim));
                        ui.add_space(8.0);
                        if ui.add(
                            egui::Button::new(RichText::new("➕ 添加第一个密钥").size(14.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(160.0, 36.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.edit_form = KeyEditForm::default();
                            self.edit_is_new = true;
                            self.navigate_to(View::KeyEdit(None));
                        }
                    } else {
                        // 有密钥但搜索过滤没有匹配
                        ui.label(RichText::new("没有匹配的密钥").size(14.0).color(theme.text_dim));
                    }
                });
            } else {
                // ===== 表头渲染 =====
                let table_w = ui.available_width();
                // 6列宽度比例：名称20%, 提供商14%, 类型12%, 环境12%, 标签24%, 操作18%
                let tbl_col_widths = [
                    table_w * 0.20,
                    table_w * 0.14,
                    table_w * 0.12,
                    table_w * 0.12,
                    table_w * 0.24,
                    table_w * 0.18,
                ];
                // 表头背景（只有上方圆角）
                egui::Frame::none()
                    .fill(theme.bg_secondary)
                    .rounding(Rounding {
                        nw: 8.0,
                        ne: 8.0,
                        sw: 0.0,
                        se: 0.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            let headers = ["名称", "提供商", "类型", "环境", "标签", "操作"];

                            for (i, header) in headers.iter().enumerate() {
                                // 表头项是可点击的排序按钮
                                let resp = ui.add_sized(
                                    Vec2::new(tbl_col_widths[i], 28.0),
                                    egui::Button::new(
                                        RichText::new(*header).size(12.0).strong().color(theme.text_secondary)
                                    ).fill(Color32::TRANSPARENT).frame(false),
                                );
                                if resp.clicked() {
                                    // 点击表头切换排序：点击同一列切换升降序，点击不同列切换为该列升序
                                    if self.key_sort_column == i {
                                        self.key_sort_ascending = !self.key_sort_ascending;
                                    } else {
                                        self.key_sort_column = i;
                                        self.key_sort_ascending = true;
                                    }
                                }
                            }
                        });
                    });

                // ===== 表格数据行（可滚动）=====
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 遍历过滤后的密钥列表
                    for (list_idx, (orig_idx, key)) in filtered_keys.iter().enumerate() {
                        // 交替行背景色（斑马纹）
                        let row_bg = if list_idx % 2 == 0 { theme.bg_card } else { theme.bg_secondary };

                        egui::Frame::none()
                            .fill(row_bg)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(8.0);

                                    // ------ 名称列（蓝色链接样式，可点击进入详情）------
                                    if ui.add_sized(
                                        Vec2::new(tbl_col_widths[0], 32.0),
                                        egui::Button::new(
                                            RichText::new(&key.name).size(13.0).color(theme.accent)
                                        ).fill(Color32::TRANSPARENT).frame(false),
                                    ).clicked() {
                                        let idx = *orig_idx;
                                        self.decrypted_value = None;
                                        self.show_decrypted_value = false;
                                        self.selected_key_index = Some(idx);
                                        self.current_view = View::KeyDetail(idx);
                                    }

                                    // ------ 提供商列 ------
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[1], 32.0),
                                        egui::Label::new(RichText::new(&key.provider).size(13.0).color(theme.text_primary)),
                                    );

                                    // ------ 类型列 ------
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[2], 32.0),
                                        egui::Label::new(RichText::new(key.key_type.to_string()).size(13.0).color(theme.text_secondary)),
                                    );

                                    // ------ 环境列（带颜色标记）------
                                    let env_color = match key.environment.to_string().as_str() {
                                        "production" => theme.error,               // 生产环境=红色
                                        "staging" => theme.warning,                 // 预发布环境=黄色
                                        "development" => theme.success,             // 开发环境=绿色
                                        _ => theme.text_secondary,
                                    };
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[3], 32.0),
                                        egui::Label::new(
                                            RichText::new(key.environment.to_string()).size(12.0).color(env_color).family(FontFamily::Monospace)
                                        ),
                                    );

                                    // ------ 标签列 ------
                                    let tags_str = if key.tags.is_empty() {
                                        "-".to_string()  // 无标签显示"-"
                                    } else {
                                        key.tags.join(", ")  // 多标签用逗号连接
                                    };
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[4], 32.0),
                                        egui::Label::new(RichText::new(tags_str).size(12.0).color(theme.text_dim)),
                                    );

                                    // ------ 操作按钮列 ------
                                    ui.horizontal(|ui| {
                                        // 复制按钮
                                        if ui.add(
                                            egui::Button::new(RichText::new("📋").size(14.0))
                                                .fill(Color32::TRANSPARENT)
                                                .frame(false)
                                        ).on_hover_text("复制密钥值").clicked() {
                                            match self.vault.get_key(&key.name, &key.environment.to_string()) {
                                                Ok((_, value)) => self.copy_to_clipboard(&value),
                                                Err(e) => self.add_notification(Notification::error(format!("获取密钥失败: {}", e))),
                                            }
                                        }

                                        // 编辑按钮
                                        if ui.add(
                                            egui::Button::new(RichText::new("✏").size(14.0))
                                                .fill(Color32::TRANSPARENT)
                                                .frame(false)
                                        ).on_hover_text("编辑").clicked() {
                                            let idx = *orig_idx;
                                            self.edit_form = KeyEditForm::from_entry(key, &self.vault);
                                            self.edit_is_new = false;
                                            self.navigate_to(View::KeyEdit(Some(idx)));
                                        }

                                        // 删除按钮（需要确认）
                                        if ui.add(
                                            egui::Button::new(RichText::new("🗑").size(14.0))
                                                .fill(Color32::TRANSPARENT)
                                                .frame(false)
                                        ).on_hover_text("删除").clicked() {
                                            self.confirm_dialog = Some(ConfirmDialog {
                                                title: "删除密钥".to_string(),
                                                message: format!("确定要删除密钥 '{}' 吗？此操作不可恢复。", key.name),
                                                on_confirm_action: ConfirmAction::DeleteKey(
                                                    key.name.clone(),
                                                    key.environment.to_string(),
                                                ),
                                            });
                                        }
                                    });
                                });
                            });

                        // ===== 行间分隔线 =====
                        ui.painter().line_segment(
                            [
                                egui::pos2(ui.cursor().left() + 8.0, ui.cursor().top()),
                                egui::pos2(ui.cursor().right() - 8.0, ui.cursor().top()),
                            ],
                            Stroke::new(0.5, theme.border),  // 0.5px细线
                        );
                    }
                });
            }
        });
    }

    // ==================== 密钥详情视图 ====================

    // show_key_detail_view() - 渲染密钥详情页面
    // 显示密钥的所有信息：基本信息、密钥值（可显示/隐藏）、时间信息
    // 提供复制值、编辑、删除等操作按钮
    fn show_key_detail_view(&mut self, ui: &mut egui::Ui, index: usize) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        // 检查索引是否有效
        if index >= self.key_list.len() {
            ui.label(RichText::new("密钥不存在").size(16.0).color(theme.error));
            return;
        }

        // 克隆要显示的密钥（避免借用冲突）
        let key = self.key_list[index].clone();

        ui.vertical(|ui| {
            // ===== 导航标题行：返回按钮 + 密钥名称标题 =====
            ui.horizontal(|ui| {
                // "← 返回" 无边框按钮
                if ui.add(
                    egui::Button::new(RichText::new("← 返回").size(13.0).color(theme.text_secondary))
                        .fill(Color32::TRANSPARENT)
                        .frame(false)
                ).clicked() {
                    self.navigate_to(View::KeyList);
                }
                ui.add_space(8.0);
                ui.label(RichText::new(format!("🔑 {}", key.name)).size(22.0).strong().color(theme.text_primary));
            });

            ui.add_space(16.0);

            // ===== 操作按钮行：复制值、编辑、删除 =====
            ui.horizontal(|ui| {
                // "复制值"按钮（主题色填充）
                if ui.add(
                    egui::Button::new(RichText::new("📋 复制值").size(13.0).color(Color32::WHITE))
                        .fill(theme.accent)
                        .min_size(Vec2::new(100.0, 34.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() {
                    match self.vault.get_key(&key.name, &key.environment.to_string()) {
                        Ok((_, value)) => self.copy_to_clipboard(&value),
                        Err(e) => self.add_notification(Notification::error(format!("获取密钥失败: {}", e))),
                    }
                }
                ui.add_space(8.0);
                // "编辑"按钮
                if ui.add(
                    egui::Button::new(RichText::new("✏ 编辑").size(13.0).color(theme.text_primary))
                        .fill(theme.bg_input)
                        .min_size(Vec2::new(80.0, 34.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() {
                    self.edit_form = KeyEditForm::from_entry(&key, &self.vault);
                    self.edit_is_new = false;
                    self.navigate_to(View::KeyEdit(Some(index)));
                }
                ui.add_space(8.0);
                // "删除"按钮（红色边框线框样式）
                if ui.add(
                    egui::Button::new(RichText::new("🗑 删除").size(13.0).color(theme.error))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme.error))  // 红色边框
                        .min_size(Vec2::new(80.0, 34.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() {
                    // 弹出确认对话框
                    self.confirm_dialog = Some(ConfirmDialog {
                        title: "删除密钥".to_string(),
                        message: format!("确定要删除密钥 '{}' 吗？此操作不可恢复。", key.name),
                        on_confirm_action: ConfirmAction::DeleteKey(
                            key.name.clone(),
                            key.environment.to_string(),
                        ),
                    });
                }
            });

            ui.add_space(20.0);

            // ===== 详情信息卡片 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    let _label_width = 120.0;

                    // ---------- 基本信息区域 ----------
                    ui.label(RichText::new("基本信息").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

                    // 信息列表：标签+值网格
                    let info_items = [
                        ("名称", key.name.clone()),
                        ("提供商", key.provider.clone()),
                        ("类型", key.key_type.to_string()),
                        ("环境", key.environment.to_string()),
                        ("版本", key.version.to_string()),
                        ("标签", if key.tags.is_empty() { "-".to_string() } else { key.tags.join(", ") }),
                        ("描述", key.description.clone().unwrap_or_else(|| "-".to_string())),
                        ("分组", key.group_id.map(|id| id.to_string()).unwrap_or_else(|| "-".to_string())),
                    ];

                    // 使用 Grid 布局：2列（标签: 值）
                    egui::Grid::new("key_detail_info")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            for (label, value) in &info_items {
                                ui.label(RichText::new(*label).size(13.0).color(theme.text_secondary));
                                ui.label(RichText::new(value).size(13.0).color(theme.text_primary));
                                ui.end_row();
                            }
                        });

                    ui.add_space(16.0);
                    ui.separator();  // 分隔线
                    ui.add_space(16.0);

                    // ---------- 密钥值区域 ----------
                    ui.label(RichText::new("密钥值").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        // 决定显示的内容
                        let display_value = if self.show_decrypted_value {
                            if let Some(ref v) = self.decrypted_value {
                                v.clone()  // 已解密，显示实际值
                            } else {
                                "（点击 '显示' 获取密钥值）".to_string()  // 尚未获取
                            }
                        } else {
                            "••••••••••••••••".to_string()  // 隐藏状态显示星号
                        };

                        let value_w = (ui.available_width() - 160.0).max(150.0);  // 自适应宽度
                        // 只读文本框显示密钥值
                        ui.add(
                            egui::TextEdit::singleline(&mut display_value.clone())
                                .desired_width(value_w)
                                .font(FontId::new(14.0, FontFamily::Monospace))  // 等宽字体显示
                                .interactive(false),  // 不可编辑
                        );

                        // "显示/隐藏"按钮
                        if ui.add(
                            egui::Button::new(RichText::new(if self.show_decrypted_value { "隐藏" } else { "显示" }).size(12.0))
                                .min_size(Vec2::new(60.0, 28.0))
                        ).clicked() {
                            if !self.show_decrypted_value && self.decrypted_value.is_none() {
                                // 首次点击"显示"，需要从Vault解密获取值
                                match self.vault.get_key(&key.name, &key.environment.to_string()) {
                                    Ok((_, value)) => {
                                        self.decrypted_value = Some(value);
                                        self.show_decrypted_value = true;
                                    }
                                    Err(e) => {
                                        self.add_notification(Notification::error(format!("解密失败: {}", e)));
                                    }
                                }
                            } else {
                                // 已有值，切换显示/隐藏
                                self.show_decrypted_value = !self.show_decrypted_value;
                            }
                        }

                        // "复制"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("📋 复制").size(12.0))
                                .min_size(Vec2::new(70.0, 28.0))
                        ).clicked() {
                            match self.vault.get_key(&key.name, &key.environment.to_string()) {
                                Ok((_, value)) => self.copy_to_clipboard(&value),
                                Err(e) => self.add_notification(Notification::error(format!("获取密钥失败: {}", e))),
                            }
                        }
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // ---------- 时间信息区域 ----------
                    ui.label(RichText::new("时间信息").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

                    let time_items = [
                        ("创建时间", key.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                        ("更新时间", key.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                        ("过期时间", key.expires_at.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "无".to_string())),
                        ("最后使用", key.last_used_at.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "从未".to_string())),
                        ("使用次数", key.usage_count.to_string()),
                    ];

                    egui::Grid::new("key_detail_time")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            for (label, value) in &time_items {
                                ui.label(RichText::new(*label).size(13.0).color(theme.text_secondary));
                                ui.label(RichText::new(value).size(13.0).color(theme.text_primary));
                                ui.end_row();
                            }
                        });
                });
        });
    }

    // ==================== 密钥编辑视图 ====================

    // show_key_edit_view() - 渲染密钥编辑/新建页面
    // index: Option<usize> - None=新建, Some(index)=编辑现有密钥
    // 包含完整的表单：名称、提供商、类型、值、环境、分组、标签、描述、过期时间
    fn show_key_edit_view(&mut self, ui: &mut egui::Ui, index: Option<usize>) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        // 根据模式显示不同标题
        let title = if self.edit_is_new { "➕ 添加密钥" } else { "✏ 编辑密钥" };

        ui.vertical(|ui| {
            // ===== 返回按钮和标题 =====
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(RichText::new("← 返回").size(13.0).color(theme.text_secondary))
                        .fill(Color32::TRANSPARENT)
                        .frame(false)
                ).clicked() {
                    self.navigate_to(View::KeyList);
                }
                ui.add_space(8.0);
                ui.label(RichText::new(title).size(22.0).strong().color(theme.text_primary));
            });

            ui.add_space(16.0);

            // ===== 表单卡片 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(24.0)
                .show(ui, |ui| {
                    let available_w = ui.available_width();
                    let input_width = (available_w * 0.65).max(200.0);  // 输入框占65%宽度

                    // 使用 Grid 布局：2列（标签 | 输入框）
                    egui::Grid::new("key_edit_form")
                        .num_columns(2)
                        .spacing([12.0, 16.0])
                        .show(ui, |ui| {
                            // ------ 名称（必填 *）------
                            ui.label(RichText::new("名称 *").size(13.0).color(theme.text_secondary));
                            ui.vertical(|ui| {
                                let name_edit = egui::TextEdit::singleline(&mut self.edit_form.name)
                                    .desired_width(input_width)
                                    .hint_text("例如: openai-api-key");
                                if self.edit_is_new {
                                    ui.add(name_edit);  // 新建模式可编辑
                                } else {
                                    ui.add(name_edit.interactive(false));  // 编辑模式禁止修改名称
                                }
                                // 显示名称验证错误
                                if let Some(ref err) = self.edit_form.name_error {
                                    ui.label(RichText::new(err).size(11.0).color(theme.error));
                                }
                            });
                            ui.end_row();

                            // ------ 提供商（必填 *）------
                            ui.label(RichText::new("提供商 *").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.provider)
                                    .desired_width(input_width)
                                    .hint_text("例如: OpenAI, AWS, Google"),
                            );
                            ui.end_row();

                            // ------ 密钥类型（下拉选择框）------
                            ui.label(RichText::new("类型").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("key_type_combo")
                                .selected_text(&self.edit_form.key_type_str)
                                .width(input_width)
                                .show_ui(ui, |ui| {
                                    let types = ["api_key", "oauth", "ssh", "cert", "jwt", "password"];
                                    for t in &types {
                                        ui.selectable_value(&mut self.edit_form.key_type_str, t.to_string(), *t);
                                    }
                                });
                            ui.end_row();

                            // ------ 密钥值（必填 *，不可见模式）------
                            ui.label(RichText::new("密钥值 *").size(13.0).color(theme.text_secondary));
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let value_edit = egui::TextEdit::singleline(&mut self.edit_form.value)
                                        .password(!self.edit_form.show_value)  // 密码模式
                                        .desired_width(input_width - 40.0)
                                        .hint_text(if self.edit_is_new { "输入密钥值" } else { "输入新的密钥值（留空则不更新）" });
                                    ui.add(value_edit);

                                    // 眼睛图标切换可见性
                                    let eye = if self.edit_form.show_value { "🙉" } else { "🙈" };
                                    if ui.add(
                                        egui::Button::new(RichText::new(eye).size(14.0))
                                            .fill(Color32::TRANSPARENT).frame(false)
                                    ).clicked() {
                                        self.edit_form.show_value = !self.edit_form.show_value;
                                    }
                                });
                                // 显示值验证错误
                                if let Some(ref err) = self.edit_form.value_error {
                                    ui.label(RichText::new(err).size(11.0).color(theme.error));
                                }
                            });
                            ui.end_row();

                            // ------ 环境（下拉选择框）------
                            ui.label(RichText::new("环境").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("env_combo")
                                .selected_text(&self.edit_form.environment_str)
                                .width(input_width)
                                .show_ui(ui, |ui| {
                                    let envs = ["development", "staging", "production", "testing"];
                                    for env in &envs {
                                        ui.selectable_value(&mut self.edit_form.environment_str, env.to_string(), *env);
                                    }
                                });
                            ui.end_row();

                            // ------ 分组（下拉选择框，显示已有分组）------
                            ui.label(RichText::new("分组").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("group_combo")
                                .selected_text(if self.edit_form.group_id_str.is_empty() { "无分组" } else { &self.edit_form.group_id_str })
                                .width(input_width)
                                .show_ui(ui, |ui| {
                                    // "无分组"选项
                                    ui.selectable_value(&mut self.edit_form.group_id_str, String::new(), "无分组");
                                    // 列出所有可用分组
                                    for group in &self.group_list {
                                        ui.selectable_value(
                                            &mut self.edit_form.group_id_str,
                                            group.id.to_string(),
                                            &group.name,
                                        );
                                    }
                                });
                            ui.end_row();

                            // ------ 标签（逗号分隔输入）------
                            ui.label(RichText::new("标签").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.tags_str)
                                    .desired_width(input_width)
                                    .hint_text("用逗号分隔，例如: production, api, v2"),
                            );
                            ui.end_row();

                            // ------ 描述（多行文本）------
                            ui.label(RichText::new("描述").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::multiline(&mut self.edit_form.description)
                                    .desired_width(input_width)
                                    .desired_rows(3)  // 默认显示3行高度
                                    .hint_text("可选描述"),
                            );
                            ui.end_row();

                            // ------ 过期日期（YYYY-MM-DD格式）------
                            ui.label(RichText::new("过期日期").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.expires_at_str)
                                    .desired_width(input_width)
                                    .hint_text("格式: YYYY-MM-DD（留空表示无过期时间）"),
                            );
                            ui.end_row();
                        });

                    ui.add_space(24.0);

                    // ===== 底部按钮行：保存 + 取消 =====
                    ui.horizontal(|ui| {
                        // "保存"按钮（主题色填充）
                        if ui.add(
                            egui::Button::new(RichText::new("💾  保存").size(14.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(120.0, 38.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.save_key_form(index);  // 调用保存逻辑
                        }

                        ui.add_space(16.0);

                        // "取消"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("取消").size(14.0).color(theme.text_secondary))
                                .fill(theme.bg_input)
                                .min_size(Vec2::new(80.0, 38.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.navigate_to(View::KeyList);
                        }
                    });
                });
        });
    }

    // save_key_form() - 处理密钥表单的保存逻辑
    // 先验证表单，然后根据新建/编辑模式调用不同的Vault方法
    fn save_key_form(&mut self, index: Option<usize>) {
        // 表单验证（检查名称非空、值非空等）
        if !self.edit_form.validate() {
            return;  // 验证失败，不保存
        }

        // ===== 解析表单字段 =====
        // 将字符串转换为 KeyType 枚举
        let key_type = KeyType::from_str(&self.edit_form.key_type_str);
        // 将字符串转换为 Environment 枚举
        let environment = Environment::from_str(&self.edit_form.environment_str);
        // 解析标签：逗号分隔，去除空白，过滤空串
        let tags: Vec<String> = self.edit_form.tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 解析分组ID（空字符串=None，否则尝试解析UUID）
        let group_id = if self.edit_form.group_id_str.is_empty() {
            None
        } else {
            uuid::Uuid::parse_str(&self.edit_form.group_id_str).ok()
        };

        // 解析描述（空字符串=None）
        let description = if self.edit_form.description.is_empty() {
            None
        } else {
            Some(self.edit_form.description.clone())
        };

        // 解析过期时间（空字符串=None，非空则尝试按YYYY-MM-DD解析）
        let expires_at = if self.edit_form.expires_at_str.is_empty() {
            None
        } else {
            chrono::NaiveDate::parse_from_str(&self.edit_form.expires_at_str, "%Y-%m-%d")
                .ok()
                .map(|d| chrono::DateTime::<Utc>::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), Utc))
        };

        if self.edit_is_new {
            // ===== 新建密钥模式 =====
            match self.vault.add_key(
                self.edit_form.name.clone(),    // 名称
                self.edit_form.provider.clone(),// 提供商
                key_type,                        // 密钥类型
                &self.edit_form.value,           // 密钥值（明文）
                environment,                     // 环境
                description,                     // 描述（Option）
                group_id,                        // 分组ID（Option）
                tags,                            // 标签列表
            ) {
                Ok(entry) => {
                    // 如果设置了过期时间，尝试更新
                    if let Some(_exp) = expires_at {
                        // 当前 API 不支持直接设置过期时间，保留未来扩展
                        let _ = self.vault.update_key_full(
                            &entry.name, &entry.environment.to_string(),
                            None, None, None, None, None,
                        );
                        // 注意：update_key_full 不支持更新 expires_at，需要额外处理
                    }
                    self.add_notification(Notification::success(format!("密钥 '{}' 已创建", self.edit_form.name)));
                    self.refresh_keys();
                    self.navigate_to(View::KeyList);
                }
                Err(e) => {
                    self.add_notification(Notification::error(format!("创建密钥失败: {}", e)));
                }
            }
        } else if let Some(idx) = index {
            // ===== 编辑现有密钥模式 =====
            let key = &self.key_list[idx];
            // 如果值为空，表示用户不更新值（保留原值）
            let new_value = if self.edit_form.value.is_empty() { None } else { Some(self.edit_form.value.as_str()) };
            let new_desc = if self.edit_form.description.is_empty() { None } else { Some(self.edit_form.description.as_str()) };

            match self.vault.update_key(
                &key.name,                         // 密钥名称（不可修改）
                &key.environment.to_string(),       // 环境（不可修改）
                new_value,                          // 新值（可选）
                new_desc,                           // 新描述（可选）
                Some(tags),                         // 新标签
            ) {
                Ok(_) => {
                    self.add_notification(Notification::success(format!("密钥 '{}' 已更新", key.name)));
                    self.refresh_keys();
                    self.navigate_to(View::KeyList);
                }
                Err(e) => {
                    self.add_notification(Notification::error(format!("更新密钥失败: {}", e)));
                }
            }
        }
    }

    // ==================== 分组管理视图 ====================

    // show_group_list_view() - 渲染分组管理页面
    // 显示所有分组及其密钥数量，支持新建分组和删除分组
    fn show_group_list_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // ===== 标题行：标题 + 新建分组输入区域 =====
            ui.horizontal(|ui| {
                ui.label(RichText::new("📁 分组管理").size(22.0).strong().color(theme.text_primary));
                // 右对齐：新建分组控件
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        // "新建分组"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("➕ 新建分组").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(110.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.new_group_name.clear();
                            self.new_group_error = None;
                        }

                        // 分组名称输入框（始终可见）
                        let group_edit = egui::TextEdit::singleline(&mut self.new_group_name)
                            .desired_width(200.0)
                            .hint_text("分组名称");
                        let resp = ui.add(group_edit);
                        // 输入框失去焦点且按下回车时触发生成
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !self.new_group_name.is_empty() {
                                match self.vault.create_group(self.new_group_name.clone(), None) {
                                    Ok(_) => {
                                        self.add_notification(Notification::success(format!("分组 '{}' 已创建", self.new_group_name)));
                                        self.new_group_name.clear();  // 清空输入框
                                        self.refresh_groups();
                                    }
                                    Err(e) => {
                                        self.new_group_error = Some(format!("{}", e));
                                    }
                                }
                            }
                        }
                    });
                });
            });

            // 显示新建分组错误
            if let Some(ref err) = self.new_group_error {
                ui.label(RichText::new(err).size(12.0).color(theme.error));
            }

            ui.add_space(16.0);

            // ===== 分组列表 =====
            if self.group_list.is_empty() {
                // 空状态
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("📁").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("暂无分组").size(16.0).color(theme.text_dim));
                    ui.add_space(4.0);
                    ui.label(RichText::new("在上方输入分组名称并按回车创建").size(13.0).color(theme.text_dim));
                });
            } else {
                // 统计每个分组下的密钥数量
                let mut group_key_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    if let Some(gid) = key.group_id {
                        *group_key_counts.entry(gid.to_string()).or_insert(0) += 1;
                    }
                }

                // 遍历并渲染每个分组卡片
                for group in &self.group_list {
                    egui::Frame::none()
                        .fill(theme.bg_card)
                        .stroke(Stroke::new(1.0, theme.border))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📁").size(18.0));
                                ui.label(RichText::new(&group.name).size(15.0).strong().color(theme.text_primary));

                                // 显示分组内的密钥数量
                                let count = group_key_counts.get(&group.id.to_string()).copied().unwrap_or(0);
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("{} 个密钥", count))
                                        .size(12.0)
                                        .color(theme.text_dim),
                                );

                                // 右对齐：删除按钮
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(
                                        egui::Button::new(RichText::new("🗑").size(14.0))
                                            .fill(Color32::TRANSPARENT).frame(false)
                                    ).on_hover_text("删除分组").clicked() {
                                        // 弹出确认对话框
                                        self.confirm_dialog = Some(ConfirmDialog {
                                            title: "删除分组".to_string(),
                                            message: format!("确定要删除分组 '{}' 吗？分组内的密钥不会被删除。", group.name),
                                            on_confirm_action: ConfirmAction::DeleteGroup(group.id.to_string()),
                                        });
                                    }
                                });
                            });
                            // 分组描述（如果有）
                            if let Some(ref desc) = group.description {
                                ui.horizontal(|ui| {
                                    ui.add_space(26.0);
                                    ui.label(RichText::new(desc).size(12.0).color(theme.text_secondary));
                                });
                            }
                            // 创建时间
                            ui.horizontal(|ui| {
                                ui.add_space(26.0);
                                ui.label(
                                    RichText::new(format!("创建于 {}", group.created_at.format("%Y-%m-%d %H:%M")))
                                        .size(11.0)
                                        .color(theme.text_dim),
                                );
                            });
                        });

                    ui.add_space(6.0);  // 卡片间间距
                }
            }
        });
    }

    // ==================== 搜索视图 ====================

    // show_search_view() - 渲染全局搜索页面
    // 支持按名称、提供商、描述搜索，显示搜索结果和复制操作
    fn show_search_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("🔍 搜索密钥").size(22.0).strong().color(theme.text_primary));
            ui.add_space(16.0);

            // ===== 搜索输入框 =====
            ui.horizontal(|ui| {
                let search_w = (ui.available_width() - 100.0).max(200.0);
                let search_edit = egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(search_w)
                    .hint_text("输入搜索关键词（名称、提供商、描述）...");
                let resp = ui.add(search_edit);

                // "搜索"按钮
                if ui.add(
                    egui::Button::new(RichText::new("🔍 搜索").size(13.0).color(Color32::WHITE))
                        .fill(theme.accent)
                        .min_size(Vec2::new(80.0, 32.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() || (resp.changed()) {
                    // 触发搜索（点击按钮或输入内容变化时）
                    if !self.search_query.is_empty() {
                        match self.vault.search_keys(&self.search_query) {
                            Ok(results) => {
                                self.search_results = results;
                            }
                            Err(e) => {
                                self.add_notification(Notification::error(format!("搜索失败: {}", e)));
                            }
                        }
                    } else {
                        self.search_results.clear();
                    }
                }
            });

            ui.add_space(16.0);

            // ===== 搜索结果 =====
            if self.search_query.is_empty() {
                // 未输入搜索词时的提示
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🔍").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("输入关键词开始搜索").size(16.0).color(theme.text_dim));
                });
            } else if self.search_results.is_empty() {
                // 无匹配结果
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("😕").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("没有找到匹配 '{}' 的密钥", self.search_query)).size(14.0).color(theme.text_dim));
                });
            } else {
                // 显示结果数量
                ui.label(
                    RichText::new(format!("找到 {} 个结果", self.search_results.len()))
                        .size(13.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                // 克隆搜索结果以避免借位冲突（用于滚动区域）
                let search_results_clone: Vec<KeyEntry> = self.search_results.clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (_i, key) in search_results_clone.iter().enumerate() {
                        // 每个搜索结果是一个卡片
                        egui::Frame::none()
                            .fill(theme.bg_card)
                            .stroke(Stroke::new(1.0, theme.border))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&key.name).size(14.0).strong().color(theme.accent));
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(format!("({})", key.provider)).size(12.0).color(theme.text_secondary));
                                            ui.add_space(8.0);

                                            // 环境标签（带颜色）
                                            let env_color = match key.environment.to_string().as_str() {
                                                "production" => theme.error,
                                                "staging" => theme.warning,
                                                _ => theme.success,
                                            };
                                            ui.label(
                                                RichText::new(key.environment.to_string())
                                                    .size(11.0).color(env_color).family(FontFamily::Monospace),
                                            );
                                        });

                                        // 描述（如果有）
                                        if let Some(ref desc) = key.description {
                                            ui.label(RichText::new(desc).size(12.0).color(theme.text_dim));
                                        }
                                    });

                                    // 右对齐：复制按钮
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(
                                            egui::Button::new(RichText::new("📋 复制").size(12.0).color(Color32::WHITE))
                                                .fill(theme.accent)
                                                .min_size(Vec2::new(70.0, 28.0))
                                                .rounding(Rounding::same(4.0))
                                        ).clicked() {
                                            match self.vault.get_key(&key.name, &key.environment.to_string()) {
                                                Ok((_, value)) => self.copy_to_clipboard(&value),
                                                Err(e) => self.add_notification(Notification::error(format!("获取密钥失败: {}", e))),
                                            }
                                        }
                                    });
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
            }
        });
    }

    // ==================== 审计日志视图 ====================

    // show_audit_log_view() - 渲染审计日志页面
    // 表格展示：时间、操作（带颜色）、资源类型、资源ID
    fn show_audit_log_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // ===== 标题行 =====
            ui.horizontal(|ui| {
                ui.label(RichText::new("📋 审计日志").size(22.0).strong().color(theme.text_primary));
                // 右对齐：刷新按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("🔄 刷新").size(13.0))
                            .fill(theme.bg_input)
                            .min_size(Vec2::new(70.0, 32.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
                        self.refresh_audit_logs();
                    }
                });
            });

            ui.add_space(16.0);

            if self.audit_logs.is_empty() {
                // 空状态
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("📋").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("暂无审计日志").size(16.0).color(theme.text_dim));
                });
            } else {
                // ===== 表头：自适应列宽 =====
                let audit_w = ui.available_width();
                let audit_cols = [
                    audit_w * 0.22,  // 时间列（22%）
                    audit_w * 0.22,  // 操作列（22%）
                    audit_w * 0.18,  // 资源类型列（18%）
                    audit_w * 0.38,  // 资源ID列（38%）
                ];
                egui::Frame::none()
                    .fill(theme.bg_secondary)
                    .rounding(Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            ui.add_sized(Vec2::new(audit_cols[0], 28.0), egui::Label::new(RichText::new("时间").size(12.0).strong().color(theme.text_secondary)));
                            ui.add_sized(Vec2::new(audit_cols[1], 28.0), egui::Label::new(RichText::new("操作").size(12.0).strong().color(theme.text_secondary)));
                            ui.add_sized(Vec2::new(audit_cols[2], 28.0), egui::Label::new(RichText::new("资源类型").size(12.0).strong().color(theme.text_secondary)));
                            ui.add_sized(Vec2::new(audit_cols[3], 28.0), egui::Label::new(RichText::new("资源ID").size(12.0).strong().color(theme.text_secondary)));
                        });
                    });

                // ===== 日志数据行（可滚动）=====
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, log) in self.audit_logs.iter().enumerate() {
                        // 交替行背景
                        let row_bg = if i % 2 == 0 { theme.bg_card } else { theme.bg_secondary };

                        egui::Frame::none()
                            .fill(row_bg)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(12.0);

                                    // 时间（等宽字体显示）
                                    let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                                    ui.add_sized(Vec2::new(audit_cols[0], 28.0), egui::Label::new(RichText::new(time_str).size(12.0).color(theme.text_secondary).family(FontFamily::Monospace)));

                                    // 操作（带颜色分类）
                                    let action_color = match log.action {
                                        AuditAction::KeyCreated | AuditAction::GroupCreated => theme.success,     // 创建=绿色
                                        AuditAction::KeyDeleted | AuditAction::GroupDeleted => theme.error,       // 删除=红色
                                        AuditAction::KeyUpdated | AuditAction::GroupUpdated | AuditAction::KeyRotated => theme.warning, // 更新=黄色
                                        AuditAction::KeyViewed | AuditAction::KeyCopied => theme.accent,          // 查看/复制=紫色
                                        AuditAction::VaultLocked => Color32::from_rgb(230, 126, 34),              // 锁定=橙色
                                        AuditAction::VaultUnlocked => theme.success,                              // 解锁=绿色
                                        _ => theme.text_secondary,
                                    };
                                    ui.add_sized(Vec2::new(audit_cols[1], 28.0), egui::Label::new(RichText::new(format!("{}", log.action)).size(12.0).color(action_color)));

                                    // 资源类型
                                    ui.add_sized(Vec2::new(audit_cols[2], 28.0), egui::Label::new(RichText::new(&log.resource_type).size(12.0).color(theme.text_secondary)));

                                    // 资源ID（等宽字体）
                                    let res_id = log.resource_id.as_deref().unwrap_or("-");
                                    ui.add_sized(Vec2::new(audit_cols[3], 28.0), egui::Label::new(
                                        RichText::new(res_id).size(11.0).color(theme.text_dim).family(FontFamily::Monospace),
                                    ));
                                });
                            });
                    }
                });
            }
        });
    }

    // ==================== 导入导出视图 ====================

    // show_import_export_view() - 渲染导入导出页面
    // 包括：导入（CSV/JSON/.env）、导出（CSV/JSON/.env）、备份/恢复
    fn show_import_export_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("📦 导入导出").size(22.0).strong().color(theme.text_primary));
            ui.add_space(20.0);

            // ===== 导入和导出并排 =====
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) / 2.0;  // 各占一半宽度
                let inner_half = (half_width - 40.0).max(40.0);  // inner_margin(20.0) 左右各20px合计40px

                // ----- 导入面板 -----
                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_width(inner_half);
                        ui.label(RichText::new("📥 导入密钥").size(16.0).strong().color(theme.text_primary));
                        ui.add_space(12.0);

                        // 格式选择下拉框
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("格式:").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("import_format")
                                .selected_text(&self.import_format)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.import_format, "csv".to_string(), "CSV");
                                    ui.selectable_value(&mut self.import_format, "json".to_string(), "JSON");
                                    ui.selectable_value(&mut self.import_format, "dotenv".to_string(), ".env");
                                });
                        });

                        ui.add_space(8.0);

                        // 文件路径输入
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("文件:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.import_file_path)
                                    .desired_width(half_width - 120.0)
                                    .hint_text("文件路径"),
                            );
                        });

                        ui.add_space(12.0);

                        // "导入"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("📥 导入").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(100.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.do_import();  // 执行导入
                        }
                    });

                ui.add_space(16.0);

                // ----- 导出面板 -----
                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_width(inner_half);
                        ui.label(RichText::new("📤 导出密钥").size(16.0).strong().color(theme.text_primary));
                        ui.add_space(12.0);

                        // 格式选择
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("格式:").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("export_format")
                                .selected_text(&self.export_format)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.export_format, "csv".to_string(), "CSV");
                                    ui.selectable_value(&mut self.export_format, "json".to_string(), "JSON");
                                    ui.selectable_value(&mut self.export_format, "dotenv".to_string(), ".env");
                                });
                        });

                        ui.add_space(8.0);

                        // 文件路径输入
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("文件:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.export_file_path)
                                    .desired_width(half_width - 120.0)
                                    .hint_text("导出文件路径"),
                            );
                        });

                        ui.add_space(12.0);

                        // "导出"按钮（绿色）
                        if ui.add(
                            egui::Button::new(RichText::new("📤 导出").size(13.0).color(Color32::WHITE))
                                .fill(theme.success)  // 绿色表示导出
                                .min_size(Vec2::new(100.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.do_export();  // 执行导出
                        }
                    });
            });

            ui.add_space(20.0);

            // ===== 备份与恢复区域 =====
            ui.label(RichText::new("💾 备份与恢复").size(16.0).strong().color(theme.text_primary));
            ui.add_space(12.0);

            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // "创建备份"按钮
                        if ui.add(
                            egui::Button::new(RichText::new("💾 创建备份").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            // 创建时间戳命名的备份文件
                            let backup_dir = self.vault.config().vault_path.join("backups");
                            let _ = std::fs::create_dir_all(&backup_dir);  // 确保备份目录存在
                            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                            let backup_path = backup_dir.join(format!("backup_{}.db", timestamp));
                            match self.vault.backup(&backup_path) {
                                Ok(()) => {
                                    self.add_notification(Notification::success(format!("备份已创建: {}", backup_path.display())));
                                }
                                Err(e) => {
                                    self.add_notification(Notification::error(format!("备份失败: {}", e)));
                                }
                            }
                        }

                        ui.add_space(16.0);
                        ui.label(RichText::new("备份到 Vault 目录下的 backups/ 子目录").size(12.0).color(theme.text_dim));
                    });
                });
        });
    }

    // do_import() - 执行导入操作
    // 读取文件，解析内容，调用 vault.import_keys() 导入密钥
    fn do_import(&mut self) {
        // 验证文件路径
        if self.import_file_path.is_empty() {
            self.add_notification(Notification::error("请输入文件路径"));
            return;
        }

        let path = std::path::Path::new(&self.import_file_path);
        if !path.exists() {
            self.add_notification(Notification::error("文件不存在"));
            return;
        }

        // 读取文件内容
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                self.add_notification(Notification::error(format!("读取文件失败: {}", e)));
                return;
            }
        };

        // 根据格式解析文件内容
        let records = match self.import_format.as_str() {
            "csv" => parse_csv_import(&content),       // CSV格式解析
            "json" => parse_json_import(&content),     // JSON格式解析
            "dotenv" => parse_dotenv_import(&content), // .env格式解析
            _ => {
                self.add_notification(Notification::error("不支持的格式"));
                return;
            }
        };

        // 使用默认环境导入密钥
        let env = Environment::from_str(&self.vault.config().default_environment);
        match self.vault.import_keys(records, env) {
            Ok(count) => {
                self.add_notification(Notification::success(format!("成功导入 {} 个密钥", count)));
                self.refresh_keys();
            }
            Err(e) => {
                self.add_notification(Notification::error(format!("导入失败: {}", e)));
            }
        }
    }

    // do_export() - 执行导出操作
    // 将密钥列表导出为指定格式的文件（不含密钥值，仅元数据）
    fn do_export(&mut self) {
        if self.export_file_path.is_empty() {
            self.add_notification(Notification::error("请输入导出文件路径"));
            return;
        }

        // 遍历所有密钥，生成导出内容（不包含密钥值）
        let keys = &self.key_list;
        let output = match self.export_format.as_str() {
            "json" => {
                // JSON格式：每个密钥导出为JSON对象
                let export_data: Vec<serde_json::Value> = keys.iter().map(|k| {
                    serde_json::json!({
                        "name": k.name,
                        "provider": k.provider,
                        "key_type": k.key_type.to_string(),
                        "environment": k.environment.to_string(),
                        "tags": k.tags,
                        "description": k.description,
                        "version": k.version,
                        "created_at": k.created_at.to_rfc3339(),
                    })
                }).collect();
                serde_json::to_string_pretty(&export_data).unwrap_or_default()
            }
            "csv" => {
                // CSV格式：写入表头和每行数据
                let mut wtr = csv::Writer::from_writer(vec![]);
                let _ = wtr.write_record(&["name", "provider", "key_type", "environment", "tags", "description", "version"]);
                for k in keys {
                    let _ = wtr.write_record(&[
                        &k.name,
                        &k.provider,
                        &k.key_type.to_string(),
                        &k.environment.to_string(),
                        &k.tags.join(";"),           // 多个标签用;分隔
                        k.description.as_deref().unwrap_or(""),
                        &k.version.to_string(),
                    ]);
                }
                String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
            }
            "dotenv" => {
                // .env格式：KEY=YOUR_VALUE_HERE  # 注释
                keys.iter().map(|k| {
                    format!("{}=YOUR_VALUE_HERE  # {} ({})", k.name.to_uppercase().replace('-', "_"), k.name, k.provider)
                }).collect::<Vec<_>>().join("\n")
            }
            _ => String::new(),
        };

        // 写入文件
        match std::fs::write(&self.export_file_path, &output) {
            Ok(()) => {
                self.add_notification(Notification::success(format!("已导出到 {}", self.export_file_path)));
            }
            Err(e) => {
                self.add_notification(Notification::error(format!("导出失败: {}", e)));
            }
        }
    }

    // ==================== 设置视图 ====================

    // show_settings_view() - 渲染设置页面
    // 包括：Vault信息、安全设置（自动锁定/剪贴板清除/主题等）、修改密码、危险操作
    fn show_settings_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("⚙ 设置").size(22.0).strong().color(theme.text_primary));
            ui.add_space(20.0);

            // ===== Vault 信息区域 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("🗄 Vault 信息").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(12.0);

                    egui::Grid::new("settings_vault_info")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Vault 路径:").size(13.0).color(theme.text_secondary));
                            ui.label(
                                RichText::new(self.vault.config().vault_path.display().to_string())
                                    .size(13.0).color(theme.text_primary).family(FontFamily::Monospace),
                            );
                            ui.end_row();
                            ui.label(RichText::new("状态:").size(13.0).color(theme.text_secondary));
                            let state_str = match self.vault.state() {
                                VaultState::Uninitialized => "未初始化",
                                VaultState::Locked => "已锁定",
                                VaultState::Unlocked => "已解锁",
                            };
                            ui.label(RichText::new(state_str).size(13.0).color(theme.success));
                            ui.end_row();
                            ui.label(RichText::new("密钥总数:").size(13.0).color(theme.text_secondary));
                            ui.label(RichText::new(self.key_list.len().to_string()).size(13.0).color(theme.text_primary));
                            ui.end_row();
                        });
                });

            ui.add_space(16.0);

            // ===== 安全设置区域 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("🔐 安全设置").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(12.0);

                    egui::Grid::new("settings_security")
                        .num_columns(2)
                        .spacing([12.0, 12.0])
                        .show(ui, |ui| {
                            // 自动锁定时间（滑动条）
                            ui.label(RichText::new("自动锁定时间（分钟）:").size(13.0).color(theme.text_secondary));
                            ui.add(egui::Slider::new(&mut self.settings_auto_lock, 1..=60).suffix(" 分钟"));
                            ui.end_row();

                            // 剪贴板清除时间（滑动条）
                            ui.label(RichText::new("剪贴板自动清除（秒）:").size(13.0).color(theme.text_secondary));
                            ui.add(egui::Slider::new(&mut self.settings_clipboard_clear, 5..=120).suffix(" 秒"));
                            ui.end_row();

                            // 主题选择（下拉框）
                            ui.label(RichText::new("主题:").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("theme_combo")
                                .selected_text(&self.settings_theme)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.settings_theme, "dark".to_string(), "暗色");
                                    ui.selectable_value(&mut self.settings_theme, "light".to_string(), "亮色");
                                });
                            ui.end_row();

                            // 默认环境（下拉框）
                            ui.label(RichText::new("默认环境:").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("default_env_combo")
                                .selected_text(&self.settings_default_env)
                                .show_ui(ui, |ui| {
                                    let envs = ["development", "staging", "production", "testing"];
                                    for env in &envs {
                                        ui.selectable_value(&mut self.settings_default_env, env.to_string(), *env);
                                    }
                                });
                            ui.end_row();

                            // 审计日志开关（复选框）
                            ui.label(RichText::new("审计日志:").size(13.0).color(theme.text_secondary));
                            ui.checkbox(&mut self.settings_audit_enabled, "启用审计日志");
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    // "保存设置"按钮
                    if ui.add(
                        egui::Button::new(RichText::new("💾 保存设置").size(13.0).color(Color32::WHITE))
                            .fill(theme.accent)
                            .min_size(Vec2::new(120.0, 34.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
                        // 将设置写入Vault配置
                        self.vault.config_mut().auto_lock_minutes = self.settings_auto_lock;
                        self.vault.config_mut().clipboard_clear_seconds = self.settings_clipboard_clear;
                        self.vault.config_mut().theme = self.settings_theme.clone();
                        self.vault.config_mut().default_environment = self.settings_default_env.clone();
                        self.vault.config_mut().audit_log_enabled = self.settings_audit_enabled;
                        match self.vault.config().save() {
                            Ok(()) => self.add_notification(Notification::success("设置已保存")),
                            Err(e) => self.add_notification(Notification::error(format!("保存失败: {}", e))),
                        }
                    }
                });

            ui.add_space(16.0);

            // ===== 修改密码区域 =====
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("🔑 修改主密码").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(12.0);

                    let pwd_input_w = (ui.available_width() * 0.5).max(200.0);
                    // 三个密码输入框：当前密码 新密码 确认新密码
                    egui::Grid::new("change_password")
                        .num_columns(2)
                        .spacing([12.0, 12.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("当前密码:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.old_password)
                                    .password(true)    // 密码模式（隐藏输入）
                                    .desired_width(pwd_input_w),
                            );
                            ui.end_row();
                            ui.label(RichText::new("新密码:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_password)
                                    .password(true)
                                    .desired_width(pwd_input_w),
                            );
                            ui.end_row();
                            ui.label(RichText::new("确认新密码:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_password_confirm)
                                    .password(true)
                                    .desired_width(pwd_input_w),
                            );
                            ui.end_row();
                        });

                    // 显示错误/成功消息
                    if let Some(ref err) = self.change_password_error {
                        ui.add_space(4.0);
                        ui.label(RichText::new(err).size(12.0).color(theme.error));
                    }
                    if self.change_password_success {
                        ui.add_space(4.0);
                        ui.label(RichText::new("✅ 密码已修改").size(12.0).color(theme.success));
                    }

                    ui.add_space(12.0);
                    // "修改密码"按钮（黄色警告色）
                    if ui.add(
                        egui::Button::new(RichText::new("🔑 修改密码").size(13.0).color(Color32::WHITE))
                            .fill(theme.warning)  // 黄色表示需要谨慎操作
                            .min_size(Vec2::new(120.0, 34.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
                        self.change_password_error = None;
                        self.change_password_success = false;

                        // 验证输入
                        if self.old_password.is_empty() || self.new_password.is_empty() {
                            self.change_password_error = Some("请填写所有字段".to_string());
                        } else if self.new_password != self.new_password_confirm {
                            self.change_password_error = Some("新密码两次输入不一致".to_string());
                        } else if self.new_password.len() < 8 {
                            self.change_password_error = Some("新密码至少需要 8 个字符".to_string());
                        } else {
                            // TODO: 实际调用 vault.change_password 时需要实现该方法
                            self.change_password_error = Some("修改密码功能尚未实现".to_string());
                        }
                    }
                });

            ui.add_space(16.0);

            // ===== 危险操作区域（红色背景警告）=====
            egui::Frame::none()
                .fill(Color32::from_rgb(40, 20, 20))  // 深红色背景，视觉警告
                .stroke(Stroke::new(1.0, theme.error))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠ 危险操作").size(15.0).strong().color(theme.error));
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        // "锁定 Vault" 按钮
                        if ui.add(
                            egui::Button::new(RichText::new("🔒 锁定 Vault").size(13.0).color(theme.warning))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, theme.warning))  // 黄色边框
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.vault.lock();
                            self.current_view = View::Login;
                            self.password_input.clear();
                            self.login_error = None;
                        }

                        ui.add_space(16.0);

                        // "重置 Vault" 按钮（需要确认）
                        if ui.add(
                            egui::Button::new(RichText::new("🗑 重置 Vault").size(13.0).color(theme.error))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, theme.error))  // 红色边框
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            // 需要确认对话框
                            self.confirm_dialog = Some(ConfirmDialog {
                                title: "⚠ 重置 Vault".to_string(),
                                message: "确定要重置 Vault 吗？这将永久删除所有密钥、分组和审计日志！此操作不可恢复！".to_string(),
                                on_confirm_action: ConfirmAction::ResetVault,
                            });
                        }
                    });
                });
        });
    }
}

// ==================== 辅助函数 ====================

// calculate_password_strength() - 计算密码强度分数
// 使用 zxcvbn 密码强度评估库
// 返回 0-4 的分数：0=非常弱, 1=弱, 2=中等, 3=强, 4=非常强
fn calculate_password_strength(password: &str) -> u8 {
    let result = zxcvbn::zxcvbn(password, &[]);
    result.score() as u8
}

// parse_csv_import() - 解析CSV格式的导入内容
// 返回 (名称, 提供商, 密钥类型, 密钥值) 元组的列表
// CSV格式预期：name, provider, key_type, value（每行一个密钥）
fn parse_csv_import(content: &str) -> Vec<(String, String, String, String)> {
    let mut records = Vec::new();
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    for result in rdr.records().flatten() {  // flatten() 忽略解析错误的行
        let name = result.get(0).unwrap_or("").to_string();
        let provider = result.get(1).unwrap_or("Unknown").to_string();
        let key_type = result.get(2).unwrap_or("api_key").to_string();
        let value = result.get(3).unwrap_or("").to_string();
        // 跳过名称或值为空的行
        if !name.is_empty() && !value.is_empty() {
            records.push((name, provider, key_type, value));
        }
    }
    records
}

// parse_json_import() - 解析JSON格式的导入内容
// 预期格式：[{"name": "...", "provider": "...", "key_type": "...", "value": "..."}]
fn parse_json_import(content: &str) -> Vec<(String, String, String, String)> {
    let mut records = Vec::new();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(arr) = json.as_array() {
            for item in arr {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let provider = item.get("provider").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                let key_type = item.get("key_type").and_then(|v| v.as_str()).unwrap_or("api_key").to_string();
                let value = item.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !name.is_empty() && !value.is_empty() {
                    records.push((name, provider, key_type, value));
                }
            }
        }
    }
    records
}

// parse_dotenv_import() - 解析 .env 格式的导入内容
// 预期格式：KEY_NAME="value"（每个键值对一行）
// 跳过空行和#开头的注释行
fn parse_dotenv_import(content: &str) -> Vec<(String, String, String, String)> {
    let mut records = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;  // 跳过空行和注释行
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();  // 去除引号
            if !key.is_empty() && !value.is_empty() {
                records.push((key.clone(), "Imported".to_string(), "api_key".to_string(), value));
            }
        }
    }
    records
}

// ==================== eframe::App trait 实现 ====================

// 为 VaultApp 实现 eframe::App trait
// eframe 框架会每帧调用这个 update 方法，所以我们在这里渲染整个UI
impl eframe::App for VaultApp {
    // update() - eframe 每帧调用的更新方法
    // ctx: &egui::Context - egui 上下文，用于样式设置、布局、输入等
    // _frame: &mut eframe::Frame - eframe 框架句柄（当前未使用）
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== 1. 应用主题样式 =====
        let theme = if self.settings_theme == "dark" {
            dark_theme()
        } else {
            light_theme()
        };

        // 设置全局样式：覆盖 egui 默认样式以匹配我们的主题
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = theme.bg_primary;         // 窗口背景色
        style.visuals.panel_fill = theme.bg_primary;          // 面板背景色
        style.visuals.override_text_color = Some(theme.text_primary);  // 全局文字颜色

        // Button 默认/悬停/按下 三种状态的样式
        style.visuals.widgets.inactive.bg_fill = theme.bg_input;          // 默认态背景
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_secondary);  // 默认态边框/前景
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 65);  // 悬停态背景（稍微变亮）
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);    // 悬停态文字色
        style.visuals.widgets.active.bg_fill = theme.accent;              // 按下态背景=主题色
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);         // 按下态文字=白色

        // TextEdit 输入框样式
        style.visuals.extreme_bg_color = theme.bg_input;       // 输入框背景色
        style.visuals.faint_bg_color = theme.bg_secondary;     // 非活跃控件背景

        // Window 窗口样式
        style.visuals.window_stroke = Stroke::new(1.0, theme.border);   // 窗口边框
        style.visuals.window_rounding = Rounding::same(8.0);            // 窗口圆角

        ctx.set_style(style);

        // ===== 2. 清理过期通知 =====
        self.cleanup_notifications();

        // ===== 3. 检查自动锁定 =====
        self.vault.check_auto_lock();
        // 如果Vault已锁定且当前不在登录页，则强制跳转到登录页
        if *self.vault.state() == VaultState::Locked && self.current_view != View::Login {
            self.current_view = View::Login;
            self.login_error = Some("Vault 已自动锁定".to_string());
            self.password_input.clear();
        }

        // ===== 4. 屏幕尺寸检测（用于响应式布局）=====
        let screen = ctx.screen_rect();
        let _is_compact = screen.width() < 900.0;  // 当屏幕宽度<900px时可切换紧凑布局（暂未使用）

        // ===== 5. 根据当前视图渲染主界面 =====
        if self.current_view == View::Login {
            // ---------- 登录页面：全屏居中 ----------
            egui::CentralPanel::default().show(ctx, |ui| {
                self.show_login_view(ui, ctx);
            });
        } else {
            // ---------- 主界面：侧边栏 + 状态栏 + 内容区 ----------

            // ===== 左侧导航栏（侧边栏）=====
            egui::SidePanel::left("sidebar")
                .resizable(false)  // 不可调整宽度
                .exact_width(if self.sidebar_collapsed { 56.0 } else { 200.0 })  // 折叠/展开宽度
                .frame(egui::Frame::none().fill(theme.bg_sidebar).inner_margin(0.0))
                .show(ctx, |ui| {
                    self.show_sidebar(ui, &theme);
                });

            // ===== 底部状态栏 =====
            egui::TopBottomPanel::bottom("status_bar")
                .exact_height(28.0)  // 固定高度28px
                .frame(egui::Frame::none().fill(theme.bg_secondary).inner_margin(egui::Margin::symmetric(8.0, 4.0)))
                .show(ctx, |ui| {
                    self.show_status_bar(ui, &theme);
                });

            // ===== 中央主内容区 =====
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(theme.bg_primary).inner_margin(20.0))  // 20px内边距
                .show(ctx, |ui| {
                    // 根据当前视图枚举渲染对应的页面
                    match self.current_view.clone() {
                        View::Dashboard => self.show_dashboard_view(ui),
                        View::KeyList => self.show_key_list_view(ui),
                        View::KeyDetail(idx) => self.show_key_detail_view(ui, idx),
                        View::KeyEdit(idx) => self.show_key_edit_view(ui, idx),
                        View::GroupList => self.show_group_list_view(ui),
                        View::Search => self.show_search_view(ui),
                        View::AuditLog => self.show_audit_log_view(ui),
                        View::ImportExport => self.show_import_export_view(ui),
                        View::Settings => self.show_settings_view(ui),
                        _ => {
                            // 未实现的视图显示占位信息
                            ui.label(RichText::new("未实现的视图").size(16.0));
                        }
                    }
                });
        }

        // ===== 6. 渲染浮动通知层（不受布局影响）=====
        self.show_notifications(ctx);

        // ===== 7. 渲染确认对话框（模态窗口）=====
        self.show_confirm_dialog(ctx);

        // ===== 8. 请求持续重绘 =====
        // egui 在无操作时默认停止重绘以节省CPU
        // 但我们有通知动画和自动锁定计时，所以请求每秒重绘一次
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
