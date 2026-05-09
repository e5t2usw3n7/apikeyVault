# 问题分析

## 核心发现

GUI 和 CLI 共享了同一个 `core/vault.rs` 的 Vault 核心层，数据库操作（增删改查）是完全一致的。

## 问题所在

### 问题 1：Session 文件不共享（最关键）

**CLI** 在 `src/cli/commands.rs` 中有完整的 session 管理机制：
- `unlock()` / `init()` 后调用 `save_session()` → master key 写入 `{vault_path}/.session`
- 执行命令前调用 `try_restore_session()` → 从 `.session` 恢复解锁
- `lock()` 时调用 `clear_session()` → 删除 `.session`

**GUI** 完全没有 session 管理！GUI 解锁、初始化后从不保存 `.session` 文件，GUI 关闭后 master key 就丢了。

### 问题 2：CLI 的 `cmd_env` 硬编码环境

```rust
// commands.rs:452
let (_entry, value) = vault.get_key(name, "development")?;
//                                        ^^^^^^^^^^^ 写死了
```

### 问题 3：GUI 锁定不清除 session

GUI 有锁定按钮，但只调用了 `vault.lock()`，没调用 `clear_session()`。


