# API Key Vault - Tauri 架构说明

## 项目结构

本项目采用 **Tauri 2.x + React + TypeScript** 架构，将应用分为三层：

### 1. 核心层 (src-core/)

共享的 Rust 业务逻辑库，包含：
- **Vault 状态机** - 管理 Uninitialized/Locked/Unlocked 三种状态
- **加密引擎** - AES-256-GCM + Argon2id
- **存储层** - SQLite + WAL 模式
- **导入导出** - CSV/JSON/.env 格式支持
- **CLI** - 命令行接口

### 2. Tauri 后端 (src-tauri/)

Tauri 应用的 Rust 部分，负责：
- 注册 Tauri 命令（IPC 桥接）
- 管理应用状态（Mutex<Vault>）
- 错误处理和序列化

### 3. React 前端 (src/)

使用现代 Web 技术构建的 UI：
- **React 18** - 组件化 UI
- **TypeScript** - 类型安全
- **Tailwind CSS** - 原子化样式
- **Zustand** - 状态管理
- **React Query** - 数据获取

## Tauri 命令接口

前端通过 `invoke()` 调用 Rust 后端命令：

```typescript
// 示例：获取密钥列表
import { invoke } from "@tauri-apps/api/core";

const keys = await invoke<KeyEntry[]>("list_keys");
```

### 可用命令

| 命令 | 说明 |
|------|------|
| `vault_status` | 获取 Vault 状态 |
| `vault_init` | 初始化 Vault |
| `vault_unlock` | 解锁 Vault |
| `vault_lock` | 锁定 Vault |
| `list_keys` | 获取密钥列表 |
| `search_keys` | 搜索密钥 |
| `get_key_value` | 获取解密后的密钥值 |
| `add_key` | 添加密钥 |
| `update_key` | 更新密钥 |
| `delete_key` | 删除密钥 |
| `test_connectivity` | 测试 API 连通性 |
| `list_groups` | 获取分组列表 |
| `create_group` | 创建分组 |
| `get_audit_logs` | 获取审计日志 |
| `import_keys` | 导入密钥 |
| `export_keys` | 导出密钥 |
| `get_config` | 获取配置 |
| `update_config` | 更新配置 |

## 前端页面

| 页面 | 路由 | 功能 |
|------|------|------|
| LoginPage | / | 密码输入、Vault 初始化 |
| DashboardPage | / | 统计概览 |
| KeyListPage | /keys | 密钥列表、搜索、筛选 |
| KeyDetailPage | /keys/:name/:env | 密钥详情、连通性测试 |
| KeyEditPage | /keys/new, /keys/:name/:env/edit | 添加/编辑密钥 |
| GroupListPage | /groups | 分组列表 |
| GroupEditPage | /groups/new, /groups/:id/edit | 添加/编辑分组 |
| AuditLogPage | /audit | 审计日志 |
| ImportExportPage | /import-export | 导入导出 |
| SettingsPage | /settings | 应用设置 |

## 开发指南

### 启动开发服务器

```bash
# 安装依赖
pnpm install

# 启动 Tauri 开发服务器
pnpm tauri dev
```

### 构建生产版本

```bash
pnpm tauri build
```

### 仅运行 CLI

```bash
cargo run -p apikey-vault-cli -- --help
```

## 安全注意事项

1. **密钥值传输** - 解密后的密钥值通过 IPC 传递，会经过 JSON 序列化
2. **剪贴板安全** - 使用 Tauri 剪贴板插件，支持自动清除
3. **内存安全** - Rust 端使用 zeroize 清零敏感数据
4. **DevTools** - 生产构建禁用开发者工具
