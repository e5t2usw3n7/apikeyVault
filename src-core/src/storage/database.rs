use std::path::Path;
use rusqlite::Connection;
use crate::error::ApiKeyError;

const SCHEMA_VERSION: u32 = 1;

/// 数据库连接管理
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开或创建数据库文件
    pub fn open(path: &Path) -> Result<Self, ApiKeyError> {
        let conn = Connection::open(path).map_err(|e| {
            ApiKeyError::Database(format!("Failed to open database: {}", e))
        })?;

        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// 使用内存数据库（测试用）
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self, ApiKeyError> {
        let conn = Connection::open_in_memory().map_err(|e| {
            ApiKeyError::Database(format!("Failed to create in-memory database: {}", e))
        })?;

        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// 初始化数据库 schema
    pub fn initialize(&self) -> Result<(), ApiKeyError> {
        self.conn
            .execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| ApiKeyError::Database(format!("Failed to set WAL mode: {}", e)))?;

        self.conn
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| ApiKeyError::Database(format!("Failed to enable foreign keys: {}", e)))?;

        // 检查版本
        let current_version = self.get_schema_version()?;
        if current_version < SCHEMA_VERSION {
            self.create_tables()?;
            self.set_schema_version(SCHEMA_VERSION)?;
        }

        Ok(())
    }

    fn create_tables(&self) -> Result<(), ApiKeyError> {
        self.conn
            .execute_batch(include_str!("../../../migrations/001_initial.sql"))
            .map_err(|e| ApiKeyError::Database(format!("Failed to create tables: {}", e)))?;
        Ok(())
    }

    fn get_schema_version(&self) -> Result<u32, ApiKeyError> {
        let version: u32 = self.conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|e| ApiKeyError::Database(format!("Failed to get schema version: {}", e)))?;
        Ok(version)
    }

    fn set_schema_version(&self, version: u32) -> Result<(), ApiKeyError> {
        self.conn
            .pragma_update(None, "user_version", version)
            .map_err(|e| ApiKeyError::Database(format!("Failed to set schema version: {}", e)))?;
        Ok(())
    }

    /// 获取底层连接引用
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// 获取可变连接引用
    #[allow(dead_code)]
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_schema_version() {
        let db = Database::open_in_memory().unwrap();
        let version = db.get_schema_version();
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), SCHEMA_VERSION);
    }
}