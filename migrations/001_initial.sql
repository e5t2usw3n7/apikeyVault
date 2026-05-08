-- apikey_etorer 数据库初始化脚本

-- 密钥分组表
CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (parent_id) REFERENCES groups(id) ON DELETE SET NULL
);

-- 密钥存储表
CREATE TABLE IF NOT EXISTS keys (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    key_type TEXT NOT NULL,
    encrypted_value BLOB NOT NULL,
    description TEXT,
    tags TEXT DEFAULT '[]',
    environment TEXT NOT NULL DEFAULT 'development',
    group_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT,
    last_used_at TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    metadata TEXT DEFAULT '{}',
    version INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE SET NULL,
    UNIQUE(name, environment)
);

-- 审计日志表
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    details TEXT
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_keys_provider ON keys(provider);
CREATE INDEX IF NOT EXISTS idx_keys_environment ON keys(environment);
CREATE INDEX IF NOT EXISTS idx_keys_group_id ON keys(group_id);
CREATE INDEX IF NOT EXISTS idx_keys_expires_at ON keys(expires_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);