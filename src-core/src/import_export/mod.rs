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

/// 从 CSV 字符串解析密钥
pub fn parse_csv(content: &str) -> Result<Vec<(String, String, String, String)>, AppError> {
    csv::parse_csv_content(content)
}

/// 从 JSON 字符串解析密钥
pub fn parse_json(content: &str) -> Result<Vec<(String, String, String, String)>, AppError> {
    json::parse_json_content(content)
}

/// 从 .env 字符串解析密钥
pub fn parse_dotenv(content: &str) -> Result<Vec<(String, String, String, String)>, AppError> {
    dotenv::parse_dotenv_content(content)
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

/// 导出为 CSV 字符串
pub fn export_csv(keys: &[(String, String, String, String)]) -> Result<String, AppError> {
    csv::export_csv_to_string(keys)
}

/// 导出为 JSON 字符串
pub fn export_json_str(keys: &[(String, String, String, String)]) -> Result<String, AppError> {
    json::export_json_to_string(keys)
}

/// 导出为 .env 字符串
pub fn export_dotenv_str(keys: &[(String, String, String, String)]) -> Result<String, AppError> {
    dotenv::export_dotenv_to_string(keys)
}
