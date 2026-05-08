pub mod csv;
pub mod json;
pub mod dotenv;

use std::path::Path;
use crate::error::AppError;

/// 导入结果
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImportResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// 从 CSV 导入密钥
pub fn import_from_csv(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    csv::import_csv(path)
}

/// 从 JSON 导入密钥
pub fn import_from_json(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    json::import_json(path)
}

/// 从 .env 导入密钥
pub fn import_from_dotenv(path: &Path) -> Result<Vec<(String, String, String, String)>, AppError> {
    dotenv::import_dotenv(path)
}

/// 导出为 CSV
pub fn export_to_csv(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    csv::export_csv(path, keys)
}

/// 导出为 JSON
pub fn export_to_json(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    json::export_json(path, keys)
}

/// 导出为 .env
pub fn export_to_dotenv(path: &Path, keys: &[(String, String, String, String)]) -> Result<(), AppError> {
    dotenv::export_dotenv(path, keys)
}