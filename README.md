# 🔐 API Key Vault

一个安全、轻量级的本地 API Key / 密钥管理工具，使用 Rust + Tauri 构建。同时提供 **Tauri 桌面应用**、**CLI 命令行** 两种交互方式，共享同一套核心业务逻辑。

## ✨ 核心特性

- **🔒 零知识安全** — 主密码通过 Argon2id 派生密钥，AES-256-GCM 认证加密，完全离线
- **🖥️ Tauri + React** — 基于 Tauri 2.x 的跨平台桌面应用 + React/TypeScript 前端
- **💻 CLI 命令行** — clap 驱动的命令行工具，适合脚本集成
- **📦 轻量级** — 使用系统 WebView，SQLite 内嵌数据库，无需外部依赖
- **🏷️ 智能分类** — 分组管理、标签系统、环境标识（dev/staging/prod）
- **🔍 格式验证** — 内置密钥模板，自动识别常见 API Key 格式
- **⏰ 过期提醒** — 密钥过期检测与自动轮换版本管理
- **📋 导入导出** — 支持 CSV / JSON / .env 格式
- **📝 审计日志** — 记录所有敏感操作，支持查询和导出
- **🌙 深色/浅色主题** — 支持主题切换

## 📸 界面预览

| 功能 | 描述 |
|------|------|
| 登录/初始化 | 主密码输入、密码强度指示器 |
| Dashboard | 密钥统计、环境分布、提供商分布 |
| 密钥管理 | 表格展示、排序、过滤、搜索 |
| 密钥详情 | 隐藏/显示值、复制、连通性测试 |
| 分组管理 | 分组列表、创建/重命名/删除 |
| 审计日志 | 时间线视图、按类型过滤 |
| 导入导出 | CSV/JSON/.env、预览功能 |
| 设置 | 自动锁定、主题切换、密码修改 |

## 🚀 快速开始

### 前置要求

- **Rust** >= 1.70（推荐通过 [rustup](https://rustup.rs/) 安装）
- **Node.js** >= 18（推荐通过 [nodejs.org](https://nodejs.org/) 安装）
- **pnpm** >= 8（通过 `npm install -g pnpm` 安装）
- **Windows / macOS / Linux** 均支持

### 安装依赖

**Windows：**
```cmd
install.bat
```

**macOS / Linux：**
```bash
chmod +x install.sh
./install.sh
```

**手动安装：**
```bash
# 克隆仓库
git clone git@github.com:e5t2usw3n7/apikeyVault.git
cd apikeyVault

# 安装前端依赖
pnpm install

# 检查 Rust 编译
cargo check
```

### 运行应用

```bash
# 启动 Tauri 桌面应用（开发模式）
pnpm tauri dev

# 运行 CLI 命令
cargo run -p apikey-vault-cli -- --help

# 运行 CLI 初始化
cargo run -p apikey-vault-cli -- init
```

### 构建发布版

```bash
# 构建 Tauri 桌面应用
pnpm tauri build

# 仅构建 CLI
cargo build -p apikey-vault-cli --release
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 串行运行测试（涉及文件系统操作时推荐）
cargo test --workspace -- --test-threads=1

# 运行特定模块测试
cargo test -p apikey-vault-core core::vault
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
apikey-vault search "openai"                            # 搜索密钥
apikey-vault search "openai" --group <group-id>         # 按分组过滤搜索
apikey-vault search "openai" --tag "ai"                 # 按标签过滤搜索

# === 密钥轮换 ===
apikey-vault rotate openai-api --value "sk-rotated-xxxxx"

# === 导入导出 ===
apikey-vault import csv keys.csv                        # 导入 CSV
apikey-vault import csv keys.csv --skip-existing        # 跳过已存在的密钥
apikey-vault import json keys.json                      # 导入 JSON
apikey-vault import env .env                            # 导入 .env
apikey-vault export csv keys.csv                        # 导出为 CSV
apikey-vault export json exported_keys.json             # 导出为 JSON
apikey-vault export env .env                            # 导出为 .env 格式
apikey-vault export json keys.json --include-values     # 导出时包含密钥值

# === 审计日志 ===
apikey-vault audit                             # 查看审计日志
apikey-vault audit --limit 50                  # 最近 50 条

# === 备份恢复 ===
apikey-vault backup backup.db
apikey-vault restore backup.db

# === 设置 ===
apikey-vault config show                       # 查看当前配置
apikey-vault config set auto_lock_minutes 10
```

### GUI 桌面应用

启动方式：

```bash
# 开发模式（支持热重载）
pnpm tauri dev

# 或使用编译后的二进制
./target/release/apikey-vault.exe
```

**GUI 功能**：

1. **登录界面** — 输入主密码，首次使用自动引导初始化
2. **Dashboard** — 密钥统计概览、环境分布、提供商分布
3. **密钥管理** — 表格展示、多维度筛选、实时搜索
4. **密钥详情** — 一键复制、隐藏/显示值、连通性测试
5. **分组管理** — 创建/重命名/删除分组
6. **导入导出** — 支持 CSV/JSON/.env，导入预览
7. **审计日志** — 操作历史查看
8. **设置** — 自动锁定时间、主题切换、密码修改

## 🏗️ 项目架构

```
apikeyVault/
├── Cargo.toml                    # Workspace 根配置
├── package.json                  # 前端依赖
├── src-core/                     # 共享 Rust 核心库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # 库入口
│       ├── main.rs               # CLI 入口
│       ├── core/                 # 核心业务逻辑
│       │   ├── vault.rs          # Vault 状态机
│       │   ├── key.rs            # 密钥数据模型
│       │   ├── group.rs          # 分组定义
│       │   ├── audit.rs          # 审计日志
│       │   └── template.rs       # 密钥模板
│       ├── crypto/               # 加密层
│       │   ├── encryption.rs     # AES-256-GCM
│       │   └── kdf.rs            # Argon2id
│       ├── storage/              # 存储层
│       │   ├── database.rs       # SQLite 连接
│       │   └── repository.rs     # 数据访问层
│       ├── config/               # 配置管理
│       ├── validation/           # 格式验证
│       └── import_export/        # 导入导出
├── src-tauri/                    # Tauri 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs               # Tauri 入口
│       ├── lib.rs                # 命令注册
│       ├── state.rs              # 应用状态
│       ├── error.rs              # 错误处理
│       └── commands/             # Tauri 命令
│           ├── auth.rs           # 认证命令
│           ├── keys.rs           # 密钥命令
│           ├── groups.rs         # 分组命令
│           ├── audit.rs          # 审计命令
│           ├── import_export.rs  # 导入导出命令
│           └── config.rs         # 配置命令
├── src/                          # React 前端
│   ├── main.tsx                  # React 入口
│   ├── App.tsx                   # 根组件
│   ├── api/                      # Tauri invoke 封装
│   ├── components/               # UI 组件
│   │   ├── layout/               # 布局组件
│   │   └── ui/                   # 通用组件
│   ├── pages/                    # 页面组件
│   ├── store/                    # Zustand 状态管理
│   ├── types/                    # TypeScript 类型
│   └── styles/                   # CSS 和主题
└── migrations/                   # 数据库迁移
```

### 分层架构

```
┌─────────────────────────────────────────────────┐
│              表示层 (Presentation)               │
│   ┌──────────────────┐  ┌──────────────────┐   │
│   │   Tauri + React  │  │       CLI        │   │
│   │   (Web 前端)     │  │   (命令行)       │   │
│   └────────┬─────────┘  └────────┬─────────┘   │
│            │                      │              │
│   ┌────────▼──────────────────────▼─────────┐   │
│   │          Tauri Command Layer            │   │
│   │         (IPC 命令桥接层)                │   │
│   └────────┬──────────────────────┬─────────┘   │
│            │                      │              │
│   ┌────────▼──────────────────────▼─────────┐   │
│   │           核心层 (src-core/)            │   │
│   │      Vault 状态机 + 业务逻辑            │   │
│   └────────┬──────────────────────┬─────────┘   │
│            │                      │              │
│   ┌────────▼──────┐      ┌───────▼───────┐     │
│   │    加密层     │      │    存储层     │     │
│   │  (crypto/)   │      │  (storage/)   │     │
│   └──────────────┘      └───────────────┘     │
└─────────────────────────────────────────────────┘
```

所有业务逻辑在 `src-core` 层实现，Tauri 前端和 CLI 均为薄壳，调用核心层方法，确保功能一致性。

## 🔐 安全设计

| 安全措施 | 实现方式 |
|----------|----------|
| 密钥派生 | Argon2id（64MB 内存、3 次迭代、4 并行度） |
| 对称加密 | AES-256-GCM（认证加密） |
| 内存安全 | `zeroize` crate 清零敏感数据 |
| 敏感数据包装 | `secrecy` crate |
| 剪贴板安全 | Tauri 剪贴板插件，N 秒后自动清除（默认 30s） |
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
```

## 🛠️ 技术栈

### 后端 (Rust)

| 类别 | 依赖 | 用途 |
|------|------|------|
| 桌面框架 | `tauri` 2.x | 跨平台桌面应用 |
| CLI | `clap` 4 | 命令行参数解析 |
| 加密 | `aes-gcm` 0.10 | AES-256-GCM 对称加密 |
| 密钥派生 | `argon2` 0.5 | Argon2id 密码哈希 |
| 数据库 | `rusqlite` 0.31 (bundled) | SQLite |
| 序列化 | `serde` / `serde_json` / `toml` | 数据序列化 |
| UUID | `uuid` 1 (v7) | 时间有序 UUID |
| 错误处理 | `thiserror` | 结构化错误 |
| 日志 | `tracing` 0.1 | 结构化日志 |

### 前端 (TypeScript)

| 类别 | 依赖 | 用途 |
|------|------|------|
| UI 框架 | `react` 18 | 组件化 UI |
| 类型系统 | `typescript` 5 | 类型安全 |
| 构建工具 | `vite` 5 | 快速开发和构建 |
| 样式 | `tailwindcss` 3 | 原子化 CSS |
| 状态管理 | `zustand` 4 | 轻量级状态管理 |
| 数据获取 | `@tanstack/react-query` 5 | 服务器状态管理 |
| 路由 | `react-router-dom` 6 | 客户端路由 |
| Tauri API | `@tauri-apps/api` 2 | 前后端通信 |

## 📄 许可证

本项目采用 **MIT OR Apache-2.0** 双许可证。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request
