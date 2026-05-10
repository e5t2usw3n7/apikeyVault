# 🔐 API Key Vault

一个安全、轻量级的本地 API Key / 密钥管理工具，使用 Rust 编写。同时提供 **GUI 桌面应用**、**CLI 命令行** 两种交互方式，共享同一套核心业务逻辑。

## ✨ 核心特性

- **🔒 零知识安全** — 主密码通过 Argon2id 派生密钥，AES-256-GCM 认证加密，完全离线
- **🖥️ GUI + CLI** — 基于 egui/eframe 的桌面图形界面 + clap 驱动的命令行工具
- **📦 轻量级** — 单个二进制文件，SQLite 内嵌数据库，无需外部依赖
- **🏷️ 智能分类** — 分组管理、标签系统、环境标识（dev/staging/prod）
- **🔍 格式验证** — 内置密钥模板，自动识别常见 API Key 格式
- **⏰ 过期提醒** — 密钥过期检测与自动轮换版本管理
- **📋 导入导出** — 支持 CSV / JSON / .env 格式
- **🐚 Shell 集成** — 支持 Bash / Zsh / Fish / PowerShell 环境变量注入
- **📝 审计日志** — 记录所有敏感操作，支持查询和导出

## 📸 界面预览

| 功能 | 描述 |
|------|------|
| 登录/初始化 | 主密码输入、密码强度指示器 |
| Dashboard | 密钥统计、过期提醒、最近操作 |
| 密钥管理 | 表格展示、排序、过滤、搜索 |
| 密钥详情 | 隐藏/显示值、复制、版本历史 |
| 分组管理 | 分组列表、创建/重命名/删除 |
| 审计日志 | 时间线视图、按类型/时间过滤 |
| 导入导出 | CSV/JSON/.env、冲突处理策略 |
| 设置 | 自动锁定、主题切换、密码修改 |

## 🚀 快速开始

### 前置要求

- **Rust** >= 1.70（推荐通过 [rustup](https://rustup.rs/) 安装）
- **Windows / macOS / Linux** 均支持

### 安装

```bash
# 克隆仓库
git clone git@github.com:e5t2usw3n7/apikeyVault.git
cd apikeyVault

# 开发模式检查
cargo check

# 运行 CLI 命令
cargo run -- --help

# 运行 GUI 桌面应用
cargo run -- gui
```

### 构建发布版

```bash
# Release 构建（启用 LTO 优化 + 去除调试符号）
cargo build --release

# 构建产物路径：
# Windows: target/release/apikey-vault.exe
# macOS:   target/release/apikey-vault
# Linux:   target/release/apikey-vault
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 串行运行测试（涉及文件系统操作时推荐）
cargo test -- --test-threads=1

# 运行特定模块测试
cargo test core::vault
```

## 📖 使用指南

### CLI 命令行

```bash
# === 初始化与认证 ===
apikey-vault init                              # 首次初始化 Vault（设置主密码）
apikey-vault unlock                            # 解锁 Vault
apikey-vault lock                              # 锁定 Vault
apikey-vault status                            # 查看 Vault 状态

# === 密钥管理 ===
apikey-vault key add "openai-api" \
  --provider "OpenAI" \
  --key-type api-key \
  --value "sk-xxxxxxxxxxxx" \
  -e production \
  --tags "ai,gpt"

apikey-vault key list                          # 列出所有密钥
apikey-vault key list --environment production # 按环境过滤
apikey-vault key get openai-api --copy         # 获取并复制到剪贴板
apikey-vault key get openai-api --full         # 显示完整密钥值
apikey-vault key update openai-api --value "sk-new-xxxxx"
apikey-vault key delete openai-api --force

# === 分组管理 ===
apikey-vault group create "AI Services"                     # 创建分组
apikey-vault group list                                     # 列出所有分组
apikey-vault group rename <group-id> "New Name"             # 重命名分组
apikey-vault group delete <group-id> --force                # 删除分组

# === 搜索 ===
apikey-vault search "openai"

# === 密钥轮换 ===
apikey-vault rotate openai-api --value "sk-rotated-xxxxx"

# === 导入导出 ===
apikey-vault import csv keys.csv
apikey-vault import json keys.json
apikey-vault import env .env
apikey-vault export json exported_keys.json

# === Shell 集成 ===
apikey-vault env openai-api --shell bash       # 生成 export 命令
apikey-vault shell init --shell zsh            # 生成 shell 初始化脚本

# === 审计日志 ===
apikey-vault audit                             # 查看审计日志
apikey-vault audit --limit 50                  # 最近 50 条

# === 备份恢复 ===
apikey-vault backup backup.db
apikey-vault restore backup.db

# === 设置 ===
apikey-vault config show                       # 查看当前配置
apikey-vault config set auto_lock_minutes 10

# === 安全检查 ===
apikey-vault security-check                    # 运行安全检查报告

# === 启动 GUI ===
apikey-vault gui
```

### GUI 桌面应用

启动方式：

```bash
# 通过 CLI 启动
cargo run -- gui

# 或使用编译后的二进制
./target/release/apikey-vault gui
```

**GUI 功能**：

1. **登录界面** — 输入主密码，首次使用自动引导初始化
2. **Dashboard** — 密钥统计概览、即将过期提醒、最近操作记录
3. **密钥管理** — 表格展示、多维度筛选、实时搜索、批量操作
4. **密钥详情** — 一键复制、隐藏/显示值、版本历史
5. **分组管理** — 创建/重命名/删除分组
6. **导入导出** — 支持 CSV/JSON/.env，导入预览与冲突处理
7. **审计日志** — 操作历史查看与搜索
8. **设置** — 自动锁定时间、主题切换、密码修改、安全检查

**快捷键**：
| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` | 添加新密钥 |
| `Ctrl+F` | 搜索 |
| `Ctrl+L` | 锁定 Vault |
| `Ctrl+I` | 导入 |
| `Ctrl+E` | 导出 |

## 🏗️ 项目架构

```
apikey-vault/
├── Cargo.toml                 # 项目配置和依赖
├── CLAUDE.md                  # 开发指导文档
├── README.md                  # 本文件
├── migrations/
│   └── 001_initial.sql        # 数据库初始化 SQL
└── src/
    ├── main.rs                # 程序入口，CLI/GUI 分发
    ├── error.rs               # 错误类型定义（thiserror）
    ├── config/                # 配置管理（TOML）
    ├── core/                  # 核心业务逻辑
    │   ├── vault.rs           # Vault 状态机（Uninit/Locked/Unlocked）
    │   ├── key.rs             # 密钥数据模型
    │   ├── group.rs           # 分组定义
    │   ├── audit.rs           # 审计日志
    │   └── template.rs        # 密钥模板（正则匹配）
    ├── crypto/                # 加密层
    │   ├── encryption.rs      # AES-256-GCM / ChaCha20-Poly1305
    │   └── kdf.rs             # Argon2id 密钥派生
    ├── storage/               # 存储层（SQLite + WAL）
    │   ├── database.rs        # 数据库连接管理
    │   └── repository.rs      # 数据访问层（CRUD）
    ├── import_export/         # 导入导出（CSV/JSON/.env）
    ├── shell/                 # Shell 集成
    ├── validation/            # 密钥格式验证
    ├── cli/                   # CLI 命令层
    └── gui/                   # GUI 桌面应用（egui/eframe）
```

### 分层架构

```
┌──────────────────────────────────────────┐
│        表示层 (Presentation)             │
│   ┌──────────┐  ┌──────────┐            │
│   │   CLI    │  │   GUI    │            │
│   └────┬─────┘  └────┬─────┘            │
│        │              │                  │
│   ┌────▼──────────────▼──────────┐      │
│   │     核心层 (core/)           │      │
│   │  Vault 状态机 + 业务逻辑     │      │
│   └────┬──────────────┬──────────┘      │
│        │              │                  │
│   ┌────▼─────┐  ┌─────▼────┐            │
│   │ 加密层   │  │ 存储层   │            │
│   │(crypto/) │  │(storage/)│            │
│   └──────────┘  └──────────┘            │
└──────────────────────────────────────────┘
```

所有业务逻辑在 `core` 层实现，GUI 和 CLI 均为薄壳，调用 core 层方法，确保功能一致性。

## 🔐 安全设计

| 安全措施 | 实现方式 |
|----------|----------|
| 密钥派生 | Argon2id（64MB 内存、3 次迭代、4 并行度） |
| 对称加密 | AES-256-GCM（认证加密） |
| 内存安全 | `zeroize` crate 清零敏感数据 |
| 敏感数据包装 | `secrecy` crate |
| 剪贴板安全 | `arboard` crate，N 秒后自动清除（默认 30s） |
| SQL 注入防护 | 参数化语句（rusqlite） |
| 审计日志 | 所有敏感操作均记录 |
| 自动锁定 | 可配置超时时间（默认 15 分钟） |

## ⚙️ 配置

配置文件路径：`{用户配置目录}/apikey_etorer/config.toml`

```toml
vault_path = "~/.apikey-vault"       # Vault 数据目录
auto_lock_minutes = 15               # 自动锁定时间（分钟）
clipboard_clear_seconds = 30         # 剪贴板清除时间（秒）
theme = "dark"                       # 主题：dark / light
default_environment = "development"  # 默认环境
audit_log_enabled = true             # 审计日志开关
max_history = 100                    # 最大历史记录数
```

## 🛠️ 技术栈

| 类别 | 依赖 | 用途 |
|------|------|------|
| CLI | `clap` 4 | 命令行参数解析 |
| GUI | `eframe` 0.29 / `egui` | 桌面图形界面 |
| 加密 | `aes-gcm` 0.10 / `chacha20poly1305` 0.10 | 对称加密 |
| 密钥派生 | `argon2` 0.5 | 密码哈希 |
| 数据库 | `rusqlite` 0.31 (bundled) | SQLite |
| 序列化 | `serde` / `serde_json` / `toml` | 数据序列化 |
| 剪贴板 | `arboard` 3 | 剪贴板操作 |
| UUID | `uuid` 1 (v7) | 时间有序 UUID |
| 错误处理 | `thiserror` 1 | 结构化错误 |
| 日志 | `tracing` 0.1 | 结构化日志 |
| 密码评估 | `zxcvbn` 3 | 密码强度检查 |

## 📄 许可证

本项目采用 **MIT OR Apache-2.0** 双许可证。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request