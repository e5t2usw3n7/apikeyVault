// ============================================================
// src/gui/mod.rs - API Key Vault 桌面 GUI 主文件
// ============================================================
// 【文件整体结构】
// L1-14    : 导入区 - 所有外部依赖和本项目模块
// L15-35   : View 枚举 - 定义所有页面视图
// L37-68   : Notification 通知系统 - 右上角弹出式通知
// L70-122  : ThemeColors 主题颜色 - 深色/浅色两套配色
// L124-218 : KeyEditForm 密钥编辑表单 - 表单字段和验证逻辑
// L220-306 : VaultApp 主结构体 - 存储所有GUI状态的中心数据结构
// L308-345 : ConfirmDialog 确认对话框 - 删除/重置等危险操作前的确认
// L347-432 : VaultApp::new() - 构造函数，初始化所有状态
// L434-480 : refresh_data() 等 - 数据刷新辅助方法
// L482-530 : 通知管理 + 剪贴板 + 导航方法
// L532-600 : show_sidebar() - 左侧导航栏渲染
// L602-640 : show_status_bar() - 底部状态栏渲染
// L642-700 : show_notifications() + show_confirm_dialog() - 浮层UI
// L700-870 : show_login_view() - 登录/初始化页面（居中面板）
// L870-1050: show_dashboard_view() - 仪表板（统计卡片+图表）
// L1050-1250: show_key_list_view() - 密钥列表（表格+搜索过滤）
// L1250-1420: show_key_detail_view() - 密钥详情页（查看/复制值）
// L1420-1600: show_key_edit_view() - 密钥编辑页（表单+保存）
// L1600-1750: show_group_list_view() - 分组管理页
// L1750-1870: show_search_view() - 全局搜索页
// L1870-1970: show_audit_log_view() - 审计日志页
// L1970-2150: show_import_export_view() - 导入导出页
// L2150-2450: show_settings_view() - 设置页（安全/密码/危险操作）
// L2450-2520: 辅助函数 - 密码强度计算、CSV/JSON/dotenv解析
// L2520-2912: eframe::App::update() - 每帧渲染的入口函数
// ============================================================

// 【导入区】
// std::path::PathBuf - 文件路径类型，用于指定Vault数据库文件位置
use std::path::PathBuf;
// chrono::Utc - UTC时间，用于通知时间戳
use chrono::Utc;

// egui 核心UI组件导入
// egui - 即时模式GUI框架
// Color32 - RGBA颜色(u8精度)
// RichText - 富文本，可设置字号/颜色/粗体等
// Vec2 - 二维向量，用于指定尺寸
// Stroke - 描边样式（宽度+颜色）
// Rounding - 圆角半径
// FontId - 字体标识（大小+字体族）
// FontFamily - 字体族枚举（Proportional/Monospace）
use eframe::egui;
use eframe::egui::{Color32, RichText, Vec2, Stroke, Rounding, FontId, FontFamily};

// 【本项目模块】
// AppConfig - 应用配置（从config.toml加载）
use crate::config::AppConfig;
// Vault - 核心保险库（加密存储/密钥CRUD/认证等）, VaultState - 状态枚举(Uninitialized/Locked/Unlocked)
use crate::core::vault::{Vault, VaultState};
// KeyEntry - 密钥条目数据结构, KeyType - 密钥类型(ApiKey/OAuth/SSH等), Environment - 环境枚举
use crate::core::key::{KeyEntry, KeyType, Environment};
// Group - 分组数据结构
use crate::core::group::Group;
// AuditEntry - 审计日志条目, AuditAction - 审计操作类型枚举
use crate::core::audit::{AuditEntry, AuditAction};

// ==================== 视图枚举 ====================
// 【View 枚举】定义应用中所有可能的页面视图
// egui 是即时模式GUI，每帧都重新渲染，所以用枚举表示当前显示哪个页面
#[derive(Debug, Clone, PartialEq)]
enum View {
    Login,                  // 登录/初始化页面（首次使用或锁定后）
    Dashboard,              // 仪表板（统计概览+快捷操作）
    KeyList,                // 密钥列表（表格展示所有密钥）
    KeyDetail(usize),       // 密钥详情（参数是 key_list 中的索引）
    KeyEdit(Option<usize>), // 密钥编辑表单（None=新建, Some(idx)=编辑现有密钥）
    GroupList,              // 分组管理
    Settings,               // 设置页面
    AuditLog,               // 审计日志
    ImportExport,           // 导入导出
    Search,                 // 全局搜索
}

// ==================== 通知系统 ====================
// 【Notification】右上角弹出式通知条
// 在 egui 中用 Area（浮动区域）实现，不参与正常布局
// 每帧检查是否过期，过期后自动移除
#[derive(Debug, Clone)]
struct Notification {
    message: String,                       // 通知文本内容
    is_error: bool,                        // true=红色错误, false=绿色成功
    created_at: chrono::DateTime<Utc>,     // 创建时间戳
    duration_secs: f64,                    // 显示持续时间（秒）
}

impl Notification {
    // 创建成功通知（绿色，显示3秒）
    fn success(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_error: false,
            created_at: Utc::now(),
            duration_secs: 3.0,
        }
    }

    // 创建错误通知（红色，显示5秒）
    fn error(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_error: true,
            created_at: Utc::now(),
            duration_secs: 5.0,
        }
    }

    // 判断通知是否已过期（超过 duration_secs 则移除）
    fn is_expired(&self) -> bool {
        let elapsed = (Utc::now() - self.created_at).num_milliseconds() as f64 / 1000.0;
        elapsed > self.duration_secs
    }
}

// ==================== 主题颜色 ====================
// 【ThemeColors】集中管理所有UI颜色，方便切换深色/浅色主题
// 每个字段对应一种UI元素颜色，函数通过 settings_theme 字段选择使用哪个主题
struct ThemeColors {
    bg_primary: Color32,      // 主背景色（页面底层）
    bg_secondary: Color32,    // 次要背景色（表头、面板间隔）
    bg_sidebar: Color32,      // 侧边栏背景色
    bg_card: Color32,         // 卡片/内容区背景色
    bg_input: Color32,        // 输入框/按钮背景色
    accent: Color32,          // 主题强调色（紫色，用于高亮/选中/链接）
    _accent_hover: Color32,   // 强调色的悬停态（前缀_表示暂未使用）
    text_primary: Color32,    // 主文字颜色（白色/深色）
    text_secondary: Color32,  // 次要文字颜色（灰色）
    text_dim: Color32,        // 最淡文字颜色（辅助说明）
    border: Color32,          // 边框/分隔线颜色
    success: Color32,         // 成功状态颜色（绿色）
    warning: Color32,         // 警告状态颜色（黄色）
    error: Color32,           // 错误状态颜色（红色）
    danger: Color32,          // 危险操作颜色（深红，用于确认按钮）
}

// 【dark_theme】深色主题配色 - 适合长时间使用，减少眼疲劳
fn dark_theme() -> ThemeColors {
    ThemeColors {
        bg_primary: Color32::from_rgb(18, 18, 24),
        bg_secondary: Color32::from_rgb(25, 25, 35),
        bg_sidebar: Color32::from_rgb(20, 20, 30),
        bg_card: Color32::from_rgb(30, 30, 42),
        bg_input: Color32::from_rgb(35, 35, 50),
        accent: Color32::from_rgb(108, 92, 231),
        _accent_hover: Color32::from_rgb(128, 112, 251),
        text_primary: Color32::from_rgb(230, 230, 240),
        text_secondary: Color32::from_rgb(160, 160, 180),
        text_dim: Color32::from_rgb(100, 100, 120),
        border: Color32::from_rgb(50, 50, 65),
        success: Color32::from_rgb(46, 204, 113),
        warning: Color32::from_rgb(241, 196, 15),
        error: Color32::from_rgb(231, 76, 60),
        danger: Color32::from_rgb(192, 57, 43),
    }
}

// 【light_theme】浅色主题配色 - 明亮背景，适合光线充足环境
fn light_theme() -> ThemeColors {
    ThemeColors {
        bg_primary: Color32::from_rgb(245, 245, 250),
        bg_secondary: Color32::from_rgb(235, 235, 242),
        bg_sidebar: Color32::from_rgb(225, 225, 235),
        bg_card: Color32::from_rgb(255, 255, 255),
        bg_input: Color32::from_rgb(240, 240, 248),
        accent: Color32::from_rgb(108, 92, 231),
        _accent_hover: Color32::from_rgb(88, 72, 211),
        text_primary: Color32::from_rgb(30, 30, 40),
        text_secondary: Color32::from_rgb(80, 80, 100),
        text_dim: Color32::from_rgb(140, 140, 160),
        border: Color32::from_rgb(200, 200, 210),
        success: Color32::from_rgb(39, 174, 96),
        warning: Color32::from_rgb(211, 170, 10),
        error: Color32::from_rgb(192, 57, 43),
        danger: Color32::from_rgb(169, 50, 38),
    }
}

// ==================== 密钥编辑表单状态 ====================
// 【KeyEditForm】密钥编辑/新建页面的表单状态
// egui 即时模式下，表单数据需要在每帧保持，所以存在 VaultApp 中
// 这个结构体封装了所有表单字段和验证错误信息
#[derive(Debug, Clone)]
struct KeyEditForm {
    name: String,              // 密钥名称（唯一标识，如 "openai-api-key"）
    provider: String,          // 提供商（如 "OpenAI", "AWS"）
    key_type_str: String,      // 密钥类型字符串（"api_key"/"oauth"/"ssh"等，用下拉框选择）
    value: String,             // 密钥值（编辑时不预填充，用户输入新值）
    environment_str: String,   // 环境字符串（"development"/"staging"/"production"等）
    description: String,       // 可选描述
    tags_str: String,          // 标签（逗号分隔的字符串，如 "prod, api, v2"）
    group_id_str: String,      // 分组ID字符串（UUID格式，空=无分组）
    expires_at_str: String,    // 过期日期字符串（"YYYY-MM-DD"格式，空=无过期）
    show_value: bool,          // 是否显示密钥值（眼睛图标切换）
    name_error: Option<String>,// 名称验证错误信息
    value_error: Option<String>,// 密钥值验证错误信息
}

// Default trait 实现 - 新建密钥时的默认值
impl Default for KeyEditForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider: String::new(),
            key_type_str: "api_key".to_string(),
            value: String::new(),
            environment_str: "development".to_string(),
            description: String::new(),
            tags_str: String::new(),
            group_id_str: String::new(),
            expires_at_str: String::new(),
            show_value: false,
            name_error: None,
            value_error: None,
        }
    }
}

impl KeyEditForm {
    // 从已有 KeyEntry 加载表单数据（编辑模式）
    // 注意：value 字段留空，因为不预加载加密后的值
    fn from_entry(entry: &KeyEntry, _vault: &Vault) -> Self {
        Self {
            name: entry.name.clone(),
            provider: entry.provider.clone(),
            key_type_str: key_type_to_str(&entry.key_type),
            value: String::new(), // 不加载已加密的值
            environment_str: entry.environment.to_string(),
            description: entry.description.clone().unwrap_or_default(),
            tags_str: entry.tags.join(", "),
            group_id_str: entry.group_id.map(|id| id.to_string()).unwrap_or_default(),
            expires_at_str: entry.expires_at.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            show_value: false,
            name_error: None,
            value_error: None,
        }
    }

    // 表单验证：检查名称非空、长度、特殊字符；值非空
    // 返回 true 表示验证通过
    fn validate(&mut self) -> bool {
        let mut valid = true;

        // 验证名称
        if self.name.is_empty() {
            self.name_error = Some("名称不能为空".to_string());
            valid = false;
        } else if self.name.len() > 128 {
            self.name_error = Some("名称长度不能超过128字符".to_string());
            valid = false;
        } else if self.name.chars().any(|c| c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
            self.name_error = Some("名称不能包含特殊字符: / \\ : * ? \" < > |".to_string());
            valid = false;
        } else {
            self.name_error = None;
        }

        // 验证值
        if self.value.is_empty() {
            self.value_error = Some("密钥值不能为空".to_string());
            valid = false;
        } else {
            self.value_error = None;
        }

        valid
    }
}

fn key_type_to_str(kt: &KeyType) -> String {
    match kt {
        KeyType::ApiKey => "api_key".to_string(),
        KeyType::OAuthToken => "oauth".to_string(),
        KeyType::SshKey => "ssh".to_string(),
        KeyType::Certificate => "cert".to_string(),
        KeyType::JwtToken => "jwt".to_string(),
        KeyType::Password => "password".to_string(),
        KeyType::Other(s) => s.clone(),
    }
}

// ==================== 主应用结构 ====================
// 【VaultApp】GUI 应用的核心状态结构体
// egui 即时模式下，所有 UI 状态都存储在这个结构中，每帧传递给渲染函数
// 这是整个应用的"单一数据源"，包含所有页面的字段
pub struct VaultApp {
    vault: Vault,                    // 核心保险库（加密存储/认证/密钥管理）
    current_view: View,              // 当前显示的页面
    previous_view: View,             // 上一个页面（用于返回导航）

    // 登录状态
    password_input: String,          // 密码输入框内容
    password_confirm: String,        // 初始化时的密码确认框
    show_password: bool,             // 是否显示密码（眼睛图标切换）
    login_error: Option<String>,     // 登录/初始化错误信息
    _password_strength: Option<(u8, String)>, // (score 0-4, feedback) 密码强度

    // 密钥列表状态
    key_list: Vec<KeyEntry>,         // 所有密钥的缓存列表
    key_search_query: String,        // 密钥列表页的搜索关键词
    key_filter_env: String,          // 按环境过滤（空=不过滤）
    _key_filter_group: String,       // 按分组过滤（暂未实现）
    key_sort_column: usize,          // 排序列索引 (0=name, 1=provider, 2=type, 3=env, 4=created)
    key_sort_ascending: bool,        // 升序/降序

    // 密钥详情
    decrypted_value: Option<String>, // 解密后的密钥值（点击"显示"后缓存）
    show_decrypted_value: bool,      // 是否显示解密值
    selected_key_index: Option<usize>, // 当前选中的密钥索引

    // 密钥编辑
    edit_form: KeyEditForm,          // 编辑表单状态
    edit_is_new: bool,               // true=新建, false=编辑

    // 分组列表
    group_list: Vec<Group>,          // 所有分组

    // 审计日志
    audit_logs: Vec<AuditEntry>,     // 审计日志列表

    // 搜索
    search_query: String,            // 全局搜索关键词
    search_results: Vec<KeyEntry>,   // 搜索结果

    // 导入导出
    import_format: String,           // 导入格式（csv/json/dotenv）
    import_file_path: String,        // 导入文件路径
    export_format: String,           // 导出格式
    export_file_path: String,        // 导出文件路径

    // 设置
    settings_auto_lock: u32,         // 自动锁定时间（分钟）
    settings_clipboard_clear: u32,   // 剪贴板清除时间（秒）
    settings_theme: String,          // 主题（dark/light）
    settings_default_env: String,    // 默认环境
    settings_audit_enabled: bool,    // 是否启用审计
    new_password: String,            // 修改密码时的新密码
    new_password_confirm: String,    // 新密码确认
    old_password: String,            // 旧密码（修改密码时验证）
    change_password_error: Option<String>, // 修改密码错误
    change_password_success: bool,   // 修改密码成功标志

    // 分组编辑
    new_group_name: String,          // 新建分组名称输入框
    new_group_error: Option<String>, // 新建分组错误

    // 通知
    notifications: Vec<Notification>, // 通知队列（右上角弹出）

    // 标记是否首次使用
    is_initialized: bool,            // Vault 是否已初始化（有数据库文件）

    // 侧边栏折叠
    sidebar_collapsed: bool,         // 侧边栏是否折叠

    // 自动锁定追踪
    _last_interaction: Option<chrono::DateTime<Utc>>, // 上次用户交互时间

    // 确认对话框
    confirm_dialog: Option<ConfirmDialog>, // 当前显示的确认对话框
}

// ==================== 确认对话框 ====================
// 【ConfirmDialog】危险操作前的确认对话框状态
// 存储对话框的标题、提示信息和确认后要执行的操作
#[derive(Debug, Clone)]
struct ConfirmDialog {
    title: String,                        // 对话框标题
    message: String,                      // 确认提示信息
    on_confirm_action: ConfirmAction,     // 用户点击"确认"后执行的操作
}

// 【ConfirmAction】确认对话框可执行的操作枚举
#[derive(Debug, Clone)]
enum ConfirmAction {
    DeleteKey(String, String),   // 删除密钥 (name, env)
    DeleteGroup(String),          // 删除分组 (group id)
    ResetVault,                    // 重置整个Vault（清空所有数据）
    _LockVault,                    // 锁定Vault（暂未在确认框中使用）
}

impl VaultApp {
    pub fn new(vault_path: PathBuf) -> Self {
        let mut config = AppConfig::load();
        config.vault_path = vault_path;
        let vault = Vault::new(config);
        let is_initialized = vault.is_initialized();
        let initial_state = if is_initialized {
            View::Login
        } else {
            View::Login
        };

        Self {
            vault,
            current_view: initial_state,
            previous_view: View::Login,

            password_input: String::new(),
            password_confirm: String::new(),
            show_password: false,
            login_error: None,
            _password_strength: None,

            key_list: Vec::new(),
            key_search_query: String::new(),
            key_filter_env: String::new(),
            _key_filter_group: String::new(),
            key_sort_column: 0,
            key_sort_ascending: true,

            decrypted_value: None,
            show_decrypted_value: false,
            selected_key_index: None,

            edit_form: KeyEditForm::default(),
            edit_is_new: true,

            group_list: Vec::new(),
            audit_logs: Vec::new(),

            search_query: String::new(),
            search_results: Vec::new(),

            import_format: "csv".to_string(),
            import_file_path: String::new(),
            export_format: "csv".to_string(),
            export_file_path: String::new(),

            settings_auto_lock: 15,
            settings_clipboard_clear: 30,
            settings_theme: "dark".to_string(),
            settings_default_env: "development".to_string(),
            settings_audit_enabled: true,
            new_password: String::new(),
            new_password_confirm: String::new(),
            old_password: String::new(),
            change_password_error: None,
            change_password_success: false,

            new_group_name: String::new(),
            new_group_error: None,

            notifications: Vec::new(),

            is_initialized,
            sidebar_collapsed: false,
            _last_interaction: Some(Utc::now()),
            confirm_dialog: None,
        }
    }

    // ==================== 数据刷新 ====================

    fn refresh_data(&mut self) {
        self.vault.check_auto_lock();
        if *self.vault.state() != VaultState::Unlocked {
            self.current_view = View::Login;
            self.login_error = Some("Vault 已自动锁定，请重新输入密码".to_string());
            return;
        }

        if let Ok(keys) = self.vault.list_keys() {
            self.key_list = keys;
        }
        if let Ok(groups) = self.vault.list_groups() {
            self.group_list = groups;
        }
        if let Ok(logs) = self.vault.get_audit_logs(50) {
            self.audit_logs = logs;
        }
    }

    fn refresh_keys(&mut self) {
        if let Ok(keys) = self.vault.list_keys() {
            self.key_list = keys;
        }
    }

    fn refresh_audit_logs(&mut self) {
        if let Ok(logs) = self.vault.get_audit_logs(50) {
            self.audit_logs = logs;
        }
    }

    fn refresh_groups(&mut self) {
        if let Ok(groups) = self.vault.list_groups() {
            self.group_list = groups;
        }
    }

    // ==================== 通知管理 ====================

    fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    fn cleanup_notifications(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    // ==================== 剪贴板操作 ====================

    fn copy_to_clipboard(&mut self, text: &str) {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text.to_string()) {
                    self.add_notification(Notification::error(format!("复制到剪贴板失败: {}", e)));
                } else {
                    self.add_notification(Notification::success("已复制到剪贴板"));
                }
            }
            Err(e) => {
                self.add_notification(Notification::error(format!("无法访问剪贴板: {}", e)));
            }
        }
    }

    // ==================== 导航 ====================

    fn navigate_to(&mut self, view: View) {
        self.previous_view = self.current_view.clone();
        self.current_view = view;

        // 进入需要数据的视图时刷新
        match &self.current_view {
            View::Dashboard | View::KeyList => {
                self.refresh_data();
            }
            View::GroupList => {
                self.refresh_groups();
            }
            View::AuditLog => {
                self.refresh_audit_logs();
            }
            View::KeyEdit(_) => {
                self.refresh_groups();
            }
            _ => {}
        }
    }

    // ==================== 设置样式 ====================

    fn _setup_style(ctx: &egui::Context, theme: &ThemeColors) {
        let mut style = (*ctx.style()).clone();
        // 使用默认spacing
        style.visuals.window_fill = theme.bg_primary;
        style.visuals.panel_fill = theme.bg_primary;
        ctx.set_style(style);
    }

    // ==================== 侧边栏渲染 ====================

    fn show_sidebar(&mut self, ui: &mut egui::Ui, theme: &ThemeColors) {
        let _sidebar_width = if self.sidebar_collapsed { 56.0 } else { 200.0 };

        ui.vertical(|ui| {
            // Logo / 标题
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                let icon_text = RichText::new("🔒").size(22.0);
                ui.label(icon_text);
                if !self.sidebar_collapsed {
                    ui.label(RichText::new("API Key Vault").size(16.0).strong().color(theme.accent));
                }
            });
            ui.add_space(16.0);

            // 折叠按钮
            if ui.add(egui::Button::new(
                if self.sidebar_collapsed { RichText::new("▶").size(14.0).color(theme.text_secondary) }
                else { RichText::new("◀").size(14.0).color(theme.text_secondary) }
            ).frame(false)).clicked() {
                self.sidebar_collapsed = !self.sidebar_collapsed;
            }
            ui.add_space(8.0);

            // 分隔线
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            painter.line_segment(
                [egui::pos2(rect.left() + 12.0, ui.cursor().top()), egui::pos2(rect.right() - 12.0, ui.cursor().top())],
                Stroke::new(1.0, theme.border),
            );
            ui.add_space(8.0);

            // 导航项
            let nav_items: Vec<(View, &str, &str)> = vec![
                (View::Dashboard, "📊", "仪表板"),
                (View::KeyList, "🔑", "密钥管理"),
                (View::GroupList, "📁", "分组管理"),
                (View::Search, "🔍", "搜索"),
                (View::AuditLog, "📋", "审计日志"),
                (View::ImportExport, "📦", "导入导出"),
                (View::Settings, "⚙", "设置"),
            ];

            for (view, icon, label) in nav_items {
                let is_active = std::mem::discriminant(&self.current_view) == std::mem::discriminant(&view);
                let btn_text = if self.sidebar_collapsed {
                    RichText::new(icon).size(20.0)
                } else {
                    RichText::new(format!("{}  {}", icon, label)).size(14.0)
                };

                let btn = if is_active {
                    egui::Button::new(btn_text.color(theme.accent))
                        .fill(Color32::from_rgb(40, 38, 65))
                        .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0))
                        .rounding(Rounding::same(6.0))
                } else {
                    egui::Button::new(btn_text.color(theme.text_secondary))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0))
                        .rounding(Rounding::same(6.0))
                };

                let resp = ui.add(btn);
                if resp.clicked() {
                    self.navigate_to(view);
                }
                // Hover 效果
                if resp.hovered() && !is_active {
                    let style = ui.style_mut();
                    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 33, 55);
                }
            }

            // 底部锁定按钮
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                let lock_text = if self.sidebar_collapsed {
                    RichText::new("🔓").size(18.0)
                } else {
                    RichText::new("🔓  锁定 Vault").size(14.0).color(theme.warning)
                };
                let lock_btn = egui::Button::new(lock_text)
                    .fill(Color32::TRANSPARENT)
                    .min_size(Vec2::new(if self.sidebar_collapsed { 40.0 } else { 176.0 }, 36.0));
                if ui.add(lock_btn).clicked() {
                    self.vault.lock();
                    self.current_view = View::Login;
                    self.password_input.clear();
                    self.login_error = None;
                }
                ui.add_space(8.0);
            });
        });
    }

    // ==================== 状态栏 ====================

    fn show_status_bar(&mut self, ui: &mut egui::Ui, theme: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Vault 状态
            let state_text = match self.vault.state() {
                VaultState::Uninitialized => "⚪ 未初始化",
                VaultState::Locked => "🔴 已锁定",
                VaultState::Unlocked => "🟢 已解锁",
            };
            ui.label(RichText::new(state_text).size(11.0).color(theme.text_dim));

            ui.add_space(16.0);

            // 密钥数量
            if *self.vault.state() == VaultState::Unlocked {
                ui.label(RichText::new(format!("🔑 {} 个密钥", self.key_list.len())).size(11.0).color(theme.text_dim));
                ui.add_space(16.0);
                ui.label(RichText::new(format!("📁 {} 个分组", self.group_list.len())).size(11.0).color(theme.text_dim));
            }

            // 右对齐
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                // 自动锁定信息
                if *self.vault.state() == VaultState::Unlocked {
                    let auto_lock = self.vault.config().auto_lock_minutes;
                    ui.label(RichText::new(format!("自动锁定: {} 分钟", auto_lock)).size(11.0).color(theme.text_dim));
                }
            });
        });
    }

    // ==================== 通知渲染 ====================

    fn show_notifications(&mut self, ctx: &egui::Context) {
        self.cleanup_notifications();

        if self.notifications.is_empty() {
            return;
        }

        let _screen_rect = ctx.screen_rect();
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        for (i, notification) in self.notifications.iter().enumerate() {
            let y_offset = 16.0 + (i as f32) * 50.0;
            let bg_color = if notification.is_error {
                Color32::from_rgb(60, 20, 20)
            } else {
                Color32::from_rgb(20, 50, 30)
            };
            let border_color = if notification.is_error { theme.error } else { theme.success };

            egui::Area::new(egui::Id::new(format!("notification_{}", i)))
                .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-16.0, y_offset))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(bg_color)
                        .stroke(Stroke::new(1.0, border_color))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                        .show(ui, |ui| {
                            let icon = if notification.is_error { "❌" } else { "✅" };
                            ui.label(RichText::new(format!("{} {}", icon, notification.message)).size(13.0).color(theme.text_primary));
                        });
                });
        }
    }

    // ==================== 确认对话框 ====================

    fn show_confirm_dialog(&mut self, ctx: &egui::Context) {
        let dialog = match self.confirm_dialog.clone() {
            Some(d) => d,
            None => return,
        };

        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        let mut close = false;
        let mut confirmed = false;

        egui::Window::new(&dialog.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(380.0, 160.0))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(&dialog.message).size(14.0).color(theme.text_primary));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("取消").size(13.0))
                            .min_size(Vec2::new(80.0, 32.0))
                    ).clicked() {
                        close = true;
                    }
                    ui.add_space(8.0);
                    if ui.add(
                        egui::Button::new(RichText::new("确认").size(13.0).color(Color32::WHITE))
                            .fill(theme.danger)
                            .min_size(Vec2::new(80.0, 32.0))
                            .rounding(Rounding::same(4.0))
                    ).clicked() {
                        confirmed = true;
                        close = true;
                    }
                });
            });

        if close {
            let action = self.confirm_dialog.as_ref().map(|d| d.on_confirm_action.clone());
            self.confirm_dialog = None;

            if confirmed {
                if let Some(action) = action {
                    self.execute_confirm_action(action);
                }
            }
        }
    }

    fn execute_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteKey(name, env) => {
                match self.vault.delete_key(&name, &env) {
                    Ok(()) => {
                        self.add_notification(Notification::success(format!("密钥 '{}' 已删除", name)));
                        self.refresh_keys();
                        if matches!(self.current_view, View::KeyDetail(_)) {
                            self.current_view = View::KeyList;
                        }
                    }
                    Err(e) => {
                        self.add_notification(Notification::error(format!("删除密钥失败: {}", e)));
                    }
                }
            }
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
            ConfirmAction::ResetVault => {
                match self.vault.reset() {
                    Ok(()) => {
                        self.add_notification(Notification::success("Vault 已重置"));
                        self.is_initialized = false;
                        self.current_view = View::Login;
                        self.password_input.clear();
                        self.key_list.clear();
                        self.group_list.clear();
                        self.audit_logs.clear();
                    }
                    Err(e) => {
                        self.add_notification(Notification::error(format!("重置失败: {}", e)));
                    }
                }
            }
            ConfirmAction::_LockVault => {
                self.vault.lock();
                self.current_view = View::Login;
                self.password_input.clear();
                self.login_error = None;
            }
        }
    }

    // ==================== 登录视图 ====================

    fn show_login_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        let screen_rect = ctx.screen_rect();
        let center = screen_rect.center();

        // 居中显示登录面板
        let panel_width = 400.0;
        let panel_height = if self.is_initialized { 300.0 } else { 380.0 };
        let panel_rect = egui::Rect::from_center_size(
            center,
            Vec2::new(panel_width, panel_height),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(panel_rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // 标题
                ui.label(RichText::new("🔒").size(48.0));
                ui.add_space(8.0);
                ui.label(RichText::new("API Key Vault").size(28.0).strong().color(theme.accent));
                ui.add_space(4.0);

                if self.is_initialized {
                    ui.label(RichText::new("输入主密码以解锁 Vault").size(14.0).color(theme.text_secondary));
                } else {
                    ui.label(RichText::new("首次使用，请设置主密码").size(14.0).color(theme.text_secondary));
                }

                ui.add_space(24.0);

                // 密码输入
                let password_width = 300.0;
                ui.horizontal(|ui| {
                    ui.add_space((panel_width - password_width) / 2.0 - 10.0);
                    let text_edit = egui::TextEdit::singleline(&mut self.password_input)
                        .password(!self.show_password)
                        .desired_width(password_width - 40.0)
                        .hint_text("主密码")
                        .font(FontId::new(16.0, FontFamily::Proportional));
                    ui.add(text_edit);

                    let eye_icon = if self.show_password { "可见" } else { "不可见" };
                    if ui.add(
                        egui::Button::new(RichText::new(eye_icon).size(16.0))
                            .fill(Color32::TRANSPARENT)
                            .frame(false)
                    ).clicked() {
                        self.show_password = !self.show_password;
                    }
                });

                // 密码强度（初始化时或已有输入时显示）
                if !self.password_input.is_empty() {
                    let strength = calculate_password_strength(&self.password_input);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        let (score, label, color) = match strength {
                            0 => (0.2, "非常弱", theme.error),
                            1 => (0.4, "弱", Color32::from_rgb(230, 126, 34)),
                            2 => (0.6, "中等", theme.warning),
                            3 => (0.8, "强", Color32::from_rgb(39, 174, 96)),
                            _ => (1.0, "非常强", theme.success),
                        };

                        // 强度条
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(password_width, 8.0), egui::Sense::hover());
                        let bg_rect = rect;
                        ui.painter().rect_filled(bg_rect, Rounding::same(4.0), theme.bg_input);
                        let fill_rect = egui::Rect::from_min_size(
                            bg_rect.min,
                            Vec2::new(bg_rect.width() * score, bg_rect.height()),
                        );
                        ui.painter().rect_filled(fill_rect, Rounding::same(4.0), color);

                        ui.add_space(4.0);
                        ui.label(RichText::new(label).size(11.0).color(color));
                    });
                }

                // 初始化时需要确认密码
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

                // 错误信息
                if let Some(ref error) = self.login_error {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        ui.label(RichText::new(format!("⚠ {}", error)).size(12.0).color(theme.error));
                    });
                }

                ui.add_space(20.0);

                // 按钮
                let btn_width = 300.0;
                if self.is_initialized {
                    // 解锁按钮
                    let unlock_btn = egui::Button::new(
                        RichText::new("🔓  解锁").size(16.0).color(Color32::WHITE)
                    )
                        .fill(theme.accent)
                        .min_size(Vec2::new(btn_width, 44.0))
                        .rounding(Rounding::same(8.0));

                    let resp = ui.add(unlock_btn);
                    if resp.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.password_input.is_empty() {
                            self.login_error = Some("请输入密码".to_string());
                        } else {
                            match self.vault.unlock(&self.password_input) {
                                Ok(()) => {
                                    self.login_error = None;
                                    self.password_input.clear();
                                    self.navigate_to(View::Dashboard);
                                }
                                Err(e) => {
                                    self.login_error = Some(format!("解锁失败: {}", e));
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                    // 重置选项
                    ui.horizontal(|ui| {
                        ui.add_space((panel_width - password_width) / 2.0);
                        if ui.add(
                            egui::Button::new(RichText::new("重置 Vault").size(12.0).color(theme.error))
                                .fill(Color32::TRANSPARENT)
                                .frame(false)
                        ).clicked() {
                            self.confirm_dialog = Some(ConfirmDialog {
                                title: "重置 Vault".to_string(),
                                message: "确定要重置 Vault 吗？这将删除所有数据，此操作不可恢复！".to_string(),
                                on_confirm_action: ConfirmAction::ResetVault,
                            });
                        }
                    });
                } else {
                    // 初始化按钮
                    let init_btn = egui::Button::new(
                        RichText::new("🚀  初始化 Vault").size(16.0).color(Color32::WHITE)
                    )
                        .fill(theme.accent)
                        .min_size(Vec2::new(btn_width, 44.0))
                        .rounding(Rounding::same(8.0));

                    if ui.add(init_btn).clicked() {
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

    fn show_dashboard_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊 仪表板").size(22.0).strong().color(theme.text_primary));
            });
            ui.add_space(16.0);

            // 统计卡片
            let total_keys = self.key_list.len();
            let total_groups = self.group_list.len();
            let total_logs = self.audit_logs.len();
            let providers: std::collections::HashSet<String> = self.key_list.iter().map(|k| k.provider.clone()).collect();

            let card_width = (ui.available_width() - 48.0) / 4.0;

            ui.horizontal(|ui| {
                // 密钥总数卡片
                self.show_stat_card(ui, &theme, "🔑", "密钥总数", &total_keys.to_string(), card_width);
                ui.add_space(16.0);
                // 分组总数
                self.show_stat_card(ui, &theme, "📁", "分组总数", &total_groups.to_string(), card_width);
                ui.add_space(16.0);
                // 提供商数
                self.show_stat_card(ui, &theme, "🏢", "提供商数", &providers.len().to_string(), card_width);
                ui.add_space(16.0);
                // 审计日志
                self.show_stat_card(ui, &theme, "📋", "操作记录", &total_logs.to_string(), card_width);
            });

            ui.add_space(20.0);

            // 按环境统计
            ui.horizontal(|ui| {
                // 环境统计卡片
                let half_width = (ui.available_width() - 16.0) / 2.0;
                self.show_env_stats_card(ui, &theme, half_width);
                ui.add_space(16.0);
                // 提供商统计卡片
                self.show_provider_stats_card(ui, &theme, half_width);
            });

            ui.add_space(20.0);

            // 快捷操作和最近日志
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) / 2.0;

                // 快捷操作
                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_min_width(half_width);
                        ui.label(RichText::new("⚡ 快捷操作").size(15.0).strong().color(theme.text_primary));
                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            if ui.add(
                                egui::Button::new(RichText::new("➕ 添加密钥").size(13.0).color(Color32::WHITE))
                                    .fill(theme.accent)
                                    .min_size(Vec2::new(120.0, 36.0))
                                    .rounding(Rounding::same(6.0))
                            ).clicked() {
                                self.edit_form = KeyEditForm::default();
                                self.edit_is_new = true;
                                self.navigate_to(View::KeyEdit(None));
                            }
                            ui.add_space(8.0);
                            if ui.add(
                                egui::Button::new(RichText::new("🔍 搜索").size(13.0).color(theme.text_primary))
                                    .fill(theme.bg_input)
                                    .min_size(Vec2::new(120.0, 36.0))
                                    .rounding(Rounding::same(6.0))
                            ).clicked() {
                                self.navigate_to(View::Search);
                            }
                            ui.add_space(8.0);
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

                // 最近审计日志
                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_min_width(half_width);
                        ui.label(RichText::new("📋 最近操作").size(15.0).strong().color(theme.text_primary));
                        ui.add_space(8.0);

                        let recent_logs: Vec<_> = self.audit_logs.iter().take(5).collect();
                        if recent_logs.is_empty() {
                            ui.label(RichText::new("暂无操作记录").size(13.0).color(theme.text_dim));
                        } else {
                            for log in &recent_logs {
                                ui.horizontal(|ui| {
                                    let action_icon = match log.action {
                                        AuditAction::KeyCreated => "➕",
                                        AuditAction::KeyViewed => "👁",
                                        AuditAction::KeyUpdated => "✏",
                                        AuditAction::KeyDeleted => "🗑",
                                        AuditAction::KeyRotated => "🔄",
                                        AuditAction::KeyCopied => "📋",
                                        AuditAction::VaultUnlocked => "🔓",
                                        AuditAction::VaultLocked => "🔒",
                                        _ => "•",
                                    };
                                    ui.label(RichText::new(action_icon).size(12.0));
                                    ui.label(
                                        RichText::new(format!("{}", log.action))
                                            .size(12.0)
                                            .color(theme.text_secondary),
                                    );
                                    ui.label(
                                        RichText::new(log.timestamp.format("%m-%d %H:%M").to_string())
                                            .size(11.0)
                                            .color(theme.text_dim),
                                    );
                                });
                            }
                        }
                    });
            });
        });
    }

    fn show_stat_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, icon: &str, title: &str, value: &str, width: f32) {
        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(width);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(20.0));
                        ui.label(RichText::new(title).size(13.0).color(theme.text_secondary));
                    });
                    ui.add_space(4.0);
                    ui.label(RichText::new(value).size(28.0).strong().color(theme.accent));
                });
            });
    }

    fn show_env_stats_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, width: f32) {
        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(width);
                ui.label(RichText::new("🌍 环境分布").size(15.0).strong().color(theme.text_primary));
                ui.add_space(8.0);

                let mut env_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    let env = key.environment.to_string();
                    *env_counts.entry(env).or_insert(0) += 1;
                }

                if env_counts.is_empty() {
                    ui.label(RichText::new("暂无密钥").size(13.0).color(theme.text_dim));
                } else {
                    let max_count = env_counts.values().max().copied().unwrap_or(1);
                    let mut envs: Vec<_> = env_counts.into_iter().collect();
                    envs.sort_by(|a, b| b.1.cmp(&a.1));

                    for (env, count) in envs {
                        ui.horizontal(|ui| {
                            let env_label_width = 100.0;
                            ui.label(RichText::new(&env).size(12.0).color(theme.text_secondary).family(FontFamily::Monospace));
                            ui.add_space(8.0);

                            let bar_width = (width - env_label_width - 60.0).max(50.0);
                            let fraction = count as f32 / max_count as f32;
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, Rounding::same(3.0), theme.bg_input);
                            let fill_rect = egui::Rect::from_min_size(
                                rect.min,
                                Vec2::new(rect.width() * fraction, rect.height()),
                            );
                            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), theme.accent);

                            ui.label(RichText::new(count.to_string()).size(12.0).color(theme.text_primary));
                        });
                        ui.add_space(4.0);
                    }
                }
            });
    }

    fn show_provider_stats_card(&self, ui: &mut egui::Ui, theme: &ThemeColors, width: f32) {
        egui::Frame::none()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(width);
                ui.label(RichText::new("🏢 提供商分布").size(15.0).strong().color(theme.text_primary));
                ui.add_space(8.0);

                let mut provider_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    *provider_counts.entry(key.provider.clone()).or_insert(0) += 1;
                }

                if provider_counts.is_empty() {
                    ui.label(RichText::new("暂无密钥").size(13.0).color(theme.text_dim));
                } else {
                    let max_count = provider_counts.values().max().copied().unwrap_or(1);
                    let mut providers: Vec<_> = provider_counts.into_iter().collect();
                    providers.sort_by(|a, b| b.1.cmp(&a.1));

                    for (provider, count) in providers.iter().take(8) {
                        ui.horizontal(|ui| {
                            let label_width = 100.0f32;
                            ui.label(RichText::new(provider).size(12.0).color(theme.text_secondary));
                            ui.add_space(8.0);

                            let bar_width = (width - label_width - 60.0).max(50.0);
                            let fraction = *count as f32 / max_count as f32;
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, Rounding::same(3.0), theme.bg_input);
                            let fill_rect = egui::Rect::from_min_size(
                                rect.min,
                                Vec2::new(rect.width() * fraction, rect.height()),
                            );
                            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), Color32::from_rgb(46, 204, 113));

                            ui.label(RichText::new(count.to_string()).size(12.0).color(theme.text_primary));
                        });
                        ui.add_space(4.0);
                    }
                }
            });
    }

    // ==================== 密钥列表视图 ====================

    fn show_key_list_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // 标题行
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔑 密钥管理").size(22.0).strong().color(theme.text_primary));
                ui.add_space(16.0);

                // 搜索框 - 自适应宽度
                let available_w = ui.available_width();
                let search_w = (available_w * 0.35).max(120.0);
                let search_edit = egui::TextEdit::singleline(&mut self.key_search_query)
                    .desired_width(search_w)
                    .hint_text("搜索密钥...");
                ui.add(search_edit);

                // 过滤下拉
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("filter_env")
                    .selected_text(if self.key_filter_env.is_empty() { "所有环境" } else { &self.key_filter_env })
                    .show_ui(ui, |ui| {
                        let mut all_env_clicked = false;
                        if ui.selectable_value(&mut self.key_filter_env, String::new(), "所有环境").clicked() {
                            all_env_clicked = true;
                        }
                        let envs = ["development", "staging", "production", "testing"];
                        for env in &envs {
                            if ui.selectable_value(&mut self.key_filter_env, env.to_string(), *env).clicked() {
                                all_env_clicked = false;
                            }
                        }
                        if all_env_clicked { self.key_filter_env.clear(); }
                    });

                // 右侧按钮
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

            // 过滤密钥列表
            let filtered_keys: Vec<(usize, KeyEntry)> = self.key_list.iter().enumerate().filter(|(_, key)| {
                let matches_search = self.key_search_query.is_empty()
                    || key.name.to_lowercase().contains(&self.key_search_query.to_lowercase())
                    || key.provider.to_lowercase().contains(&self.key_search_query.to_lowercase())
                    || key.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&self.key_search_query.to_lowercase()));

                let matches_env = self.key_filter_env.is_empty()
                    || key.environment.to_string() == self.key_filter_env;

                matches_search && matches_env
            }).map(|(idx, key)| (idx, key.clone())).collect();

            // 密钥表格
            if filtered_keys.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🔑").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    if self.key_list.is_empty() {
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
                        ui.label(RichText::new("没有匹配的密钥").size(14.0).color(theme.text_dim));
                    }
                });
            } else {
                // 表头 - 自适应列宽
                let table_w = ui.available_width();
                let tbl_col_widths = [
                    table_w * 0.20,  // 名称
                    table_w * 0.14,  // 提供商
                    table_w * 0.12,  // 类型
                    table_w * 0.12,  // 环境
                    table_w * 0.24,  // 标签
                    table_w * 0.18,  // 操作
                ];
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
                                let resp = ui.add_sized(
                                    Vec2::new(tbl_col_widths[i], 28.0),
                                    egui::Button::new(
                                        RichText::new(*header).size(12.0).strong().color(theme.text_secondary)
                                    ).fill(Color32::TRANSPARENT).frame(false),
                                );
                                if resp.clicked() {
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

                // 表格内容（可滚动）
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (list_idx, (orig_idx, key)) in filtered_keys.iter().enumerate() {
                        let row_bg = if list_idx % 2 == 0 { theme.bg_card } else { theme.bg_secondary };

                        egui::Frame::none()
                            .fill(row_bg)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(8.0);

                                    // 名称
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

                                    // 提供商
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[1], 32.0),
                                        egui::Label::new(RichText::new(&key.provider).size(13.0).color(theme.text_primary)),
                                    );

                                    // 类型
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[2], 32.0),
                                        egui::Label::new(RichText::new(key.key_type.to_string()).size(13.0).color(theme.text_secondary)),
                                    );

                                    // 环境
                                    let env_color = match key.environment.to_string().as_str() {
                                        "production" => theme.error,
                                        "staging" => theme.warning,
                                        "development" => theme.success,
                                        _ => theme.text_secondary,
                                    };
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[3], 32.0),
                                        egui::Label::new(
                                            RichText::new(key.environment.to_string()).size(12.0).color(env_color).family(FontFamily::Monospace)
                                        ),
                                    );

                                    // 标签
                                    let tags_str = if key.tags.is_empty() {
                                        "-".to_string()
                                    } else {
                                        key.tags.join(", ")
                                    };
                                    ui.add_sized(
                                        Vec2::new(tbl_col_widths[4], 32.0),
                                        egui::Label::new(RichText::new(tags_str).size(12.0).color(theme.text_dim)),
                                    );

                                    // 操作按钮
                                    ui.horizontal(|ui| {
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

                        // 分隔线
                        ui.painter().line_segment(
                            [
                                egui::pos2(ui.cursor().left() + 8.0, ui.cursor().top()),
                                egui::pos2(ui.cursor().right() - 8.0, ui.cursor().top()),
                            ],
                            Stroke::new(0.5, theme.border),
                        );
                    }
                });
            }
        });
    }

    // ==================== 密钥详情视图 ====================

    fn show_key_detail_view(&mut self, ui: &mut egui::Ui, index: usize) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        if index >= self.key_list.len() {
            ui.label(RichText::new("密钥不存在").size(16.0).color(theme.error));
            return;
        }

        let key = self.key_list[index].clone();

        ui.vertical(|ui| {
            // 返回按钮和标题
            ui.horizontal(|ui| {
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

            // 操作按钮
            ui.horizontal(|ui| {
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
                if ui.add(
                    egui::Button::new(RichText::new("🗑 删除").size(13.0).color(theme.error))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, theme.error))
                        .min_size(Vec2::new(80.0, 34.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() {
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

            // 详情卡片
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    let _label_width = 120.0;

                    // 基本信息
                    ui.label(RichText::new("基本信息").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

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
                    ui.separator();
                    ui.add_space(16.0);

                    // 密钥值
                    ui.label(RichText::new("密钥值").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let display_value = if self.show_decrypted_value {
                            if let Some(ref v) = self.decrypted_value {
                                v.clone()
                            } else {
                                "（点击 '显示' 获取密钥值）".to_string()
                            }
                        } else {
                            "••••••••••••••••".to_string()
                        };

                        let value_w = (ui.available_width() - 160.0).max(150.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut display_value.clone())
                                .desired_width(value_w)
                                .font(FontId::new(14.0, FontFamily::Monospace))
                                .interactive(false),
                        );

                        if ui.add(
                            egui::Button::new(RichText::new(if self.show_decrypted_value { "隐藏" } else { "显示" }).size(12.0))
                                .min_size(Vec2::new(60.0, 28.0))
                        ).clicked() {
                            if !self.show_decrypted_value && self.decrypted_value.is_none() {
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
                                self.show_decrypted_value = !self.show_decrypted_value;
                            }
                        }

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

                    // 时间信息
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

    fn show_key_edit_view(&mut self, ui: &mut egui::Ui, index: Option<usize>) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };
        let title = if self.edit_is_new { "➕ 添加密钥" } else { "✏ 编辑密钥" };

        ui.vertical(|ui| {
            // 标题和返回
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

            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(24.0)
                .show(ui, |ui| {
                    let available_w = ui.available_width();
                    let input_width = (available_w * 0.65).max(200.0);

                    egui::Grid::new("key_edit_form")
                        .num_columns(2)
                        .spacing([12.0, 16.0])
                        .show(ui, |ui| {
                            // 名称
                            ui.label(RichText::new("名称 *").size(13.0).color(theme.text_secondary));
                            ui.vertical(|ui| {
                                let name_edit = egui::TextEdit::singleline(&mut self.edit_form.name)
                                    .desired_width(input_width)
                                    .hint_text("例如: openai-api-key");
                                if self.edit_is_new {
                                    ui.add(name_edit);
                                } else {
                                    ui.add(name_edit.interactive(false));
                                }
                                if let Some(ref err) = self.edit_form.name_error {
                                    ui.label(RichText::new(err).size(11.0).color(theme.error));
                                }
                            });
                            ui.end_row();

                            // 提供商
                            ui.label(RichText::new("提供商 *").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.provider)
                                    .desired_width(input_width)
                                    .hint_text("例如: OpenAI, AWS, Google"),
                            );
                            ui.end_row();

                            // 密钥类型
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

                            // 密钥值
                            ui.label(RichText::new("密钥值 *").size(13.0).color(theme.text_secondary));
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let value_edit = egui::TextEdit::singleline(&mut self.edit_form.value)
                                        .password(!self.edit_form.show_value)
                                        .desired_width(input_width - 40.0)
                                        .hint_text(if self.edit_is_new { "输入密钥值" } else { "输入新的密钥值（留空则不更新）" });
                                    ui.add(value_edit);

                                    let eye = if self.edit_form.show_value { "可见" } else { "不可见" };
                                    if ui.add(
                                        egui::Button::new(RichText::new(eye).size(14.0))
                                            .fill(Color32::TRANSPARENT).frame(false)
                                    ).clicked() {
                                        self.edit_form.show_value = !self.edit_form.show_value;
                                    }
                                });
                                if let Some(ref err) = self.edit_form.value_error {
                                    ui.label(RichText::new(err).size(11.0).color(theme.error));
                                }
                            });
                            ui.end_row();

                            // 环境
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

                            // 分组
                            ui.label(RichText::new("分组").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("group_combo")
                                .selected_text(if self.edit_form.group_id_str.is_empty() { "无分组" } else { &self.edit_form.group_id_str })
                                .width(input_width)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.edit_form.group_id_str, String::new(), "无分组");
                                    for group in &self.group_list {
                                        ui.selectable_value(
                                            &mut self.edit_form.group_id_str,
                                            group.id.to_string(),
                                            &group.name,
                                        );
                                    }
                                });
                            ui.end_row();

                            // 标签
                            ui.label(RichText::new("标签").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.tags_str)
                                    .desired_width(input_width)
                                    .hint_text("用逗号分隔，例如: production, api, v2"),
                            );
                            ui.end_row();

                            // 描述
                            ui.label(RichText::new("描述").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::multiline(&mut self.edit_form.description)
                                    .desired_width(input_width)
                                    .desired_rows(3)
                                    .hint_text("可选描述"),
                            );
                            ui.end_row();

                            // 过期时间
                            ui.label(RichText::new("过期日期").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_form.expires_at_str)
                                    .desired_width(input_width)
                                    .hint_text("格式: YYYY-MM-DD（留空表示无过期时间）"),
                            );
                            ui.end_row();
                        });

                    ui.add_space(24.0);

                    // 保存按钮
                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("💾  保存").size(14.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(120.0, 38.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.save_key_form(index);
                        }

                        ui.add_space(16.0);

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

    fn save_key_form(&mut self, index: Option<usize>) {
        if !self.edit_form.validate() {
            return;
        }

        let key_type = KeyType::from_str(&self.edit_form.key_type_str);
        let environment = Environment::from_str(&self.edit_form.environment_str);
        let tags: Vec<String> = self.edit_form.tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let group_id = if self.edit_form.group_id_str.is_empty() {
            None
        } else {
            uuid::Uuid::parse_str(&self.edit_form.group_id_str).ok()
        };

        let description = if self.edit_form.description.is_empty() {
            None
        } else {
            Some(self.edit_form.description.clone())
        };

        let expires_at = if self.edit_form.expires_at_str.is_empty() {
            None
        } else {
            chrono::NaiveDate::parse_from_str(&self.edit_form.expires_at_str, "%Y-%m-%d")
                .ok()
                .map(|d| chrono::DateTime::<Utc>::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), Utc))
        };

        if self.edit_is_new {
            // 新建密钥
            match self.vault.add_key(
                self.edit_form.name.clone(),
                self.edit_form.provider.clone(),
                key_type,
                &self.edit_form.value,
                environment,
                description,
                group_id,
                tags,
            ) {
                Ok(entry) => {
                    // 设置过期时间
                    if let Some(_exp) = expires_at {
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
            // 编辑现有密钥
            let key = &self.key_list[idx];
            let new_value = if self.edit_form.value.is_empty() { None } else { Some(self.edit_form.value.as_str()) };
            let new_desc = if self.edit_form.description.is_empty() { None } else { Some(self.edit_form.description.as_str()) };

            match self.vault.update_key(
                &key.name,
                &key.environment.to_string(),
                new_value,
                new_desc,
                Some(tags),
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

    fn show_group_list_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.label(RichText::new("📁 分组管理").size(22.0).strong().color(theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 新建分组
                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("➕ 新建分组").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(110.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.new_group_name.clear();
                            self.new_group_error = None;
                        }

                        // 新建分组输入框（始终显示）
                        let group_edit = egui::TextEdit::singleline(&mut self.new_group_name)
                            .desired_width(200.0)
                            .hint_text("分组名称");
                        let resp = ui.add(group_edit);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !self.new_group_name.is_empty() {
                                match self.vault.create_group(self.new_group_name.clone(), None) {
                                    Ok(_) => {
                                        self.add_notification(Notification::success(format!("分组 '{}' 已创建", self.new_group_name)));
                                        self.new_group_name.clear();
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

            if let Some(ref err) = self.new_group_error {
                ui.label(RichText::new(err).size(12.0).color(theme.error));
            }

            ui.add_space(16.0);

            if self.group_list.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("📁").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("暂无分组").size(16.0).color(theme.text_dim));
                    ui.add_space(4.0);
                    ui.label(RichText::new("在上方输入分组名称并按回车创建").size(13.0).color(theme.text_dim));
                });
            } else {
                // 统计每个分组的密钥数量
                let mut group_key_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for key in &self.key_list {
                    if let Some(gid) = key.group_id {
                        *group_key_counts.entry(gid.to_string()).or_insert(0) += 1;
                    }
                }

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

                                let count = group_key_counts.get(&group.id.to_string()).copied().unwrap_or(0);
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("{} 个密钥", count))
                                        .size(12.0)
                                        .color(theme.text_dim),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(
                                        egui::Button::new(RichText::new("🗑").size(14.0))
                                            .fill(Color32::TRANSPARENT).frame(false)
                                    ).on_hover_text("删除分组").clicked() {
                                        self.confirm_dialog = Some(ConfirmDialog {
                                            title: "删除分组".to_string(),
                                            message: format!("确定要删除分组 '{}' 吗？分组内的密钥不会被删除。", group.name),
                                            on_confirm_action: ConfirmAction::DeleteGroup(group.id.to_string()),
                                        });
                                    }
                                });
                            });
                            if let Some(ref desc) = group.description {
                                ui.horizontal(|ui| {
                                    ui.add_space(26.0);
                                    ui.label(RichText::new(desc).size(12.0).color(theme.text_secondary));
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.add_space(26.0);
                                ui.label(
                                    RichText::new(format!("创建于 {}", group.created_at.format("%Y-%m-%d %H:%M")))
                                        .size(11.0)
                                        .color(theme.text_dim),
                                );
                            });
                        });

                    ui.add_space(6.0);
                }
            }
        });
    }

    // ==================== 搜索视图 ====================

    fn show_search_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("🔍 搜索密钥").size(22.0).strong().color(theme.text_primary));
            ui.add_space(16.0);

            // 搜索框 - 自适应宽度
            ui.horizontal(|ui| {
                let search_w = (ui.available_width() - 100.0).max(200.0);
                let search_edit = egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(search_w)
                    .hint_text("输入搜索关键词（名称、提供商、描述）...");
                let resp = ui.add(search_edit);

                if ui.add(
                    egui::Button::new(RichText::new("🔍 搜索").size(13.0).color(Color32::WHITE))
                        .fill(theme.accent)
                        .min_size(Vec2::new(80.0, 32.0))
                        .rounding(Rounding::same(6.0))
                ).clicked() || (resp.changed()) {
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

            // 搜索结果
            if self.search_query.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🔍").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("输入关键词开始搜索").size(16.0).color(theme.text_dim));
                });
            } else if self.search_results.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("😕").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("没有找到匹配 '{}' 的密钥", self.search_query)).size(14.0).color(theme.text_dim));
                });
            } else {
                ui.label(
                    RichText::new(format!("找到 {} 个结果", self.search_results.len()))
                        .size(13.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                let search_results_clone: Vec<KeyEntry> = self.search_results.clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (_i, key) in search_results_clone.iter().enumerate() {
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

                                        if let Some(ref desc) = key.description {
                                            ui.label(RichText::new(desc).size(12.0).color(theme.text_dim));
                                        }
                                    });

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

    fn show_audit_log_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📋 审计日志").size(22.0).strong().color(theme.text_primary));
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
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("📋").size(48.0).color(theme.text_dim));
                    ui.add_space(8.0);
                    ui.label(RichText::new("暂无审计日志").size(16.0).color(theme.text_dim));
                });
            } else {
                // 表头 - 自适应列宽
                let audit_w = ui.available_width();
                let audit_cols = [
                    audit_w * 0.22,  // 时间
                    audit_w * 0.22,  // 操作
                    audit_w * 0.18,  // 资源类型
                    audit_w * 0.38,  // 资源ID
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

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, log) in self.audit_logs.iter().enumerate() {
                        let row_bg = if i % 2 == 0 { theme.bg_card } else { theme.bg_secondary };

                        egui::Frame::none()
                            .fill(row_bg)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(12.0);

                                    let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                                    ui.add_sized(Vec2::new(audit_cols[0], 28.0), egui::Label::new(RichText::new(time_str).size(12.0).color(theme.text_secondary).family(FontFamily::Monospace)));

                                    // 操作带颜色
                                    let action_color = match log.action {
                                        AuditAction::KeyCreated | AuditAction::GroupCreated => theme.success,
                                        AuditAction::KeyDeleted | AuditAction::GroupDeleted => theme.error,
                                        AuditAction::KeyUpdated | AuditAction::GroupUpdated | AuditAction::KeyRotated => theme.warning,
                                        AuditAction::KeyViewed | AuditAction::KeyCopied => theme.accent,
                                        AuditAction::VaultLocked => Color32::from_rgb(230, 126, 34),
                                        AuditAction::VaultUnlocked => theme.success,
                                        _ => theme.text_secondary,
                                    };
                                    ui.add_sized(Vec2::new(audit_cols[1], 28.0), egui::Label::new(RichText::new(format!("{}", log.action)).size(12.0).color(action_color)));

                                    ui.add_sized(Vec2::new(audit_cols[2], 28.0), egui::Label::new(RichText::new(&log.resource_type).size(12.0).color(theme.text_secondary)));

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

    fn show_import_export_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("📦 导入导出").size(22.0).strong().color(theme.text_primary));
            ui.add_space(20.0);

            // 导入部分
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) / 2.0;

                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_width(half_width);
                        ui.label(RichText::new("📥 导入密钥").size(16.0).strong().color(theme.text_primary));
                        ui.add_space(12.0);

                        // 格式选择
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

                        // 文件路径
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("文件:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.import_file_path)
                                    .desired_width(half_width - 120.0)
                                    .hint_text("文件路径"),
                            );
                        });

                        ui.add_space(12.0);

                        if ui.add(
                            egui::Button::new(RichText::new("📥 导入").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(100.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.do_import();
                        }
                    });

                ui.add_space(16.0);

                // 导出部分
                egui::Frame::none()
                    .fill(theme.bg_card)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_width(half_width);
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

                        // 文件路径
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("文件:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.export_file_path)
                                    .desired_width(half_width - 120.0)
                                    .hint_text("导出文件路径"),
                            );
                        });

                        ui.add_space(12.0);

                        if ui.add(
                            egui::Button::new(RichText::new("📤 导出").size(13.0).color(Color32::WHITE))
                                .fill(theme.success)
                                .min_size(Vec2::new(100.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.do_export();
                        }
                    });
            });

            ui.add_space(20.0);

            // 备份/恢复
            ui.label(RichText::new("💾 备份与恢复").size(16.0).strong().color(theme.text_primary));
            ui.add_space(12.0);

            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("💾 创建备份").size(13.0).color(Color32::WHITE))
                                .fill(theme.accent)
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            let backup_dir = self.vault.config().vault_path.join("backups");
                            let _ = std::fs::create_dir_all(&backup_dir);
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

    fn do_import(&mut self) {
        if self.import_file_path.is_empty() {
            self.add_notification(Notification::error("请输入文件路径"));
            return;
        }

        let path = std::path::Path::new(&self.import_file_path);
        if !path.exists() {
            self.add_notification(Notification::error("文件不存在"));
            return;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                self.add_notification(Notification::error(format!("读取文件失败: {}", e)));
                return;
            }
        };

        let records = match self.import_format.as_str() {
            "csv" => parse_csv_import(&content),
            "json" => parse_json_import(&content),
            "dotenv" => parse_dotenv_import(&content),
            _ => {
                self.add_notification(Notification::error("不支持的格式"));
                return;
            }
        };

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

    fn do_export(&mut self) {
        if self.export_file_path.is_empty() {
            self.add_notification(Notification::error("请输入导出文件路径"));
            return;
        }

        // 导出所有密钥的元数据（不含值）
        let keys = &self.key_list;
        let output = match self.export_format.as_str() {
            "json" => {
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
                let mut wtr = csv::Writer::from_writer(vec![]);
                let _ = wtr.write_record(&["name", "provider", "key_type", "environment", "tags", "description", "version"]);
                for k in keys {
                    let _ = wtr.write_record(&[
                        &k.name,
                        &k.provider,
                        &k.key_type.to_string(),
                        &k.environment.to_string(),
                        &k.tags.join(";"),
                        k.description.as_deref().unwrap_or(""),
                        &k.version.to_string(),
                    ]);
                }
                String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
            }
            "dotenv" => {
                keys.iter().map(|k| {
                    format!("{}=YOUR_VALUE_HERE  # {} ({})", k.name.to_uppercase().replace('-', "_"), k.name, k.provider)
                }).collect::<Vec<_>>().join("\n")
            }
            _ => String::new(),
        };

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

    fn show_settings_view(&mut self, ui: &mut egui::Ui) {
        let theme = if self.settings_theme == "dark" { dark_theme() } else { light_theme() };

        ui.vertical(|ui| {
            ui.label(RichText::new("⚙ 设置").size(22.0).strong().color(theme.text_primary));
            ui.add_space(20.0);

            // Vault 信息
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

            // 安全设置
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
                            // 自动锁定时间
                            ui.label(RichText::new("自动锁定时间（分钟）:").size(13.0).color(theme.text_secondary));
                            ui.add(egui::Slider::new(&mut self.settings_auto_lock, 1..=60).suffix(" 分钟"));
                            ui.end_row();

                            // 剪贴板清除时间
                            ui.label(RichText::new("剪贴板自动清除（秒）:").size(13.0).color(theme.text_secondary));
                            ui.add(egui::Slider::new(&mut self.settings_clipboard_clear, 5..=120).suffix(" 秒"));
                            ui.end_row();

                            // 主题
                            ui.label(RichText::new("主题:").size(13.0).color(theme.text_secondary));
                            egui::ComboBox::from_id_salt("theme_combo")
                                .selected_text(&self.settings_theme)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.settings_theme, "dark".to_string(), "暗色");
                                    ui.selectable_value(&mut self.settings_theme, "light".to_string(), "亮色");
                                });
                            ui.end_row();

                            // 默认环境
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

                            // 审计日志
                            ui.label(RichText::new("审计日志:").size(13.0).color(theme.text_secondary));
                            ui.checkbox(&mut self.settings_audit_enabled, "启用审计日志");
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    if ui.add(
                        egui::Button::new(RichText::new("💾 保存设置").size(13.0).color(Color32::WHITE))
                            .fill(theme.accent)
                            .min_size(Vec2::new(120.0, 34.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
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

            // 修改密码
            egui::Frame::none()
                .fill(theme.bg_card)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("🔑 修改主密码").size(15.0).strong().color(theme.text_primary));
                    ui.add_space(12.0);

                    let pwd_input_w = (ui.available_width() * 0.5).max(200.0);
                    egui::Grid::new("change_password")
                        .num_columns(2)
                        .spacing([12.0, 12.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("当前密码:").size(13.0).color(theme.text_secondary));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.old_password)
                                    .password(true)
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

                    if let Some(ref err) = self.change_password_error {
                        ui.add_space(4.0);
                        ui.label(RichText::new(err).size(12.0).color(theme.error));
                    }
                    if self.change_password_success {
                        ui.add_space(4.0);
                        ui.label(RichText::new("✅ 密码已修改").size(12.0).color(theme.success));
                    }

                    ui.add_space(12.0);
                    if ui.add(
                        egui::Button::new(RichText::new("🔑 修改密码").size(13.0).color(Color32::WHITE))
                            .fill(theme.warning)
                            .min_size(Vec2::new(120.0, 34.0))
                            .rounding(Rounding::same(6.0))
                    ).clicked() {
                        self.change_password_error = None;
                        self.change_password_success = false;

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

            // 危险操作
            egui::Frame::none()
                .fill(Color32::from_rgb(40, 20, 20))
                .stroke(Stroke::new(1.0, theme.error))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠ 危险操作").size(15.0).strong().color(theme.error));
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("🔒 锁定 Vault").size(13.0).color(theme.warning))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, theme.warning))
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
                            self.vault.lock();
                            self.current_view = View::Login;
                            self.password_input.clear();
                            self.login_error = None;
                        }

                        ui.add_space(16.0);

                        if ui.add(
                            egui::Button::new(RichText::new("🗑 重置 Vault").size(13.0).color(theme.error))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, theme.error))
                                .min_size(Vec2::new(120.0, 34.0))
                                .rounding(Rounding::same(6.0))
                        ).clicked() {
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

fn calculate_password_strength(password: &str) -> u8 {
    let result = zxcvbn::zxcvbn(password, &[]);
    result.score() as u8
}

fn parse_csv_import(content: &str) -> Vec<(String, String, String, String)> {
    let mut records = Vec::new();
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    for result in rdr.records().flatten() {
        let name = result.get(0).unwrap_or("").to_string();
        let provider = result.get(1).unwrap_or("Unknown").to_string();
        let key_type = result.get(2).unwrap_or("api_key").to_string();
        let value = result.get(3).unwrap_or("").to_string();
        if !name.is_empty() && !value.is_empty() {
            records.push((name, provider, key_type, value));
        }
    }
    records
}

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

fn parse_dotenv_import(content: &str) -> Vec<(String, String, String, String)> {
    let mut records = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() && !value.is_empty() {
                records.push((key.clone(), "Imported".to_string(), "api_key".to_string(), value));
            }
        }
    }
    records
}

// ==================== eframe::App 实现 ====================

impl eframe::App for VaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用主题
        let theme = if self.settings_theme == "dark" {
            dark_theme()
        } else {
            light_theme()
        };

        // 设置全局样式
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = theme.bg_primary;
        style.visuals.panel_fill = theme.bg_primary;
        style.visuals.override_text_color = Some(theme.text_primary);

        // 按钮样式
        style.visuals.widgets.inactive.bg_fill = theme.bg_input;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_secondary);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 65);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);
        style.visuals.widgets.active.bg_fill = theme.accent;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // 文本输入框样式
        style.visuals.extreme_bg_color = theme.bg_input;
        style.visuals.faint_bg_color = theme.bg_secondary;

        // 窗口样式
        style.visuals.window_stroke = Stroke::new(1.0, theme.border);
        style.visuals.window_rounding = Rounding::same(8.0);

        ctx.set_style(style);

        // 清理通知
        self.cleanup_notifications();

        // 检查自动锁定
        self.vault.check_auto_lock();
        if *self.vault.state() == VaultState::Locked && self.current_view != View::Login {
            self.current_view = View::Login;
            self.login_error = Some("Vault 已自动锁定".to_string());
            self.password_input.clear();
        }

        // 获取屏幕尺寸用于决定布局
        let screen = ctx.screen_rect();
        let _is_compact = screen.width() < 900.0;

        if self.current_view == View::Login {
            // 登录界面 - 全屏居中
            egui::CentralPanel::default().show(ctx, |ui| {
                self.show_login_view(ui, ctx);
            });
        } else {
            // 主界面 - 侧边栏 + 内容区

            // 侧边栏
            egui::SidePanel::left("sidebar")
                .resizable(false)
                .exact_width(if self.sidebar_collapsed { 56.0 } else { 200.0 })
                .frame(egui::Frame::none().fill(theme.bg_sidebar).inner_margin(0.0))
                .show(ctx, |ui| {
                    self.show_sidebar(ui, &theme);
                });

            // 底部状态栏
            egui::TopBottomPanel::bottom("status_bar")
                .exact_height(28.0)
                .frame(egui::Frame::none().fill(theme.bg_secondary).inner_margin(egui::Margin::symmetric(8.0, 4.0)))
                .show(ctx, |ui| {
                    self.show_status_bar(ui, &theme);
                });

            // 主内容区
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(theme.bg_primary).inner_margin(20.0))
                .show(ctx, |ui| {
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
                            ui.label(RichText::new("未实现的视图").size(16.0));
                        }
                    }
                });
        }

        // 通知层
        self.show_notifications(ctx);

        // 确认对话框
        self.show_confirm_dialog(ctx);

        // 请求持续重绘（用于通知过期等动画）
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}