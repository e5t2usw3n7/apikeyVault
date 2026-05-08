use crate::core::key::KeyType;
use crate::core::template::builtin_templates;
use crate::error::ApiKeyError;

/// 密钥格式验证结果
#[derive(Debug)]
#[allow(dead_code)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// 验证密钥格式
pub fn validate_key_format(value: &str, key_type: &KeyType) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut is_valid = true;

    // 基本长度检查
    if value.len() < 8 {
        warnings.push("Key value is very short (< 8 characters)".to_string());
    }

    // 检查是否包含空格
    if value.contains(' ') && matches!(key_type, KeyType::ApiKey | KeyType::OAuthToken) {
        warnings.push("Key value contains spaces, which may be invalid".to_string());
    }

    // 尝试匹配内置模板
    let templates = builtin_templates();
    let mut matched = false;
    for template in &templates {
        if template.validate(value) {
            matched = true;
            suggestions.push(format!("This looks like a {} key", template.name));
            break;
        }
    }

    if !matched && value.len() > 20 {
        suggestions.push("No known pattern matched. Consider adding a custom template.".to_string());
    }

    // 检查是否可能是环境变量格式
    if value.starts_with('$') && value.contains('{') {
        warnings.push("Value looks like an environment variable reference, not an actual key".to_string());
    }

    // 检查是否可能是占位符
    let placeholder_indicators = ["xxx", "your_key_here", "REPLACE", "TODO", "FIXME", "PLACEHOLDER"];
    for indicator in &placeholder_indicators {
        if value.to_lowercase().contains(&indicator.to_lowercase()) {
            warnings.push(format!("Value may be a placeholder (contains '{}')", indicator));
            is_valid = false;
        }
    }

    ValidationResult {
        is_valid,
        warnings,
        suggestions,
    }
}

/// 验证密钥名称
pub fn validate_key_name(name: &str) -> Result<(), ApiKeyError> {
    if name.is_empty() {
        return Err(ApiKeyError::InvalidInput("Key name cannot be empty".to_string()));
    }

    if name.len() > 128 {
        return Err(ApiKeyError::InvalidInput("Key name too long (max 128 characters)".to_string()));
    }

    // 检查特殊字符
    if name.contains('\n') || name.contains('\r') || name.contains('\t') {
        return Err(ApiKeyError::InvalidInput("Key name contains invalid characters".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openai_key() {
        let result = validate_key_format("sk-abcdefghijklmnopqrstuvwxyz1234567890", &KeyType::ApiKey);
        assert!(result.is_valid);
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_validate_placeholder() {
        let result = validate_key_format("xxx_your_key_here_xxx", &KeyType::ApiKey);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_short_key() {
        let result = validate_key_format("short", &KeyType::ApiKey);
        assert!(result.warnings.iter().any(|w| w.contains("short")));
    }

    #[test]
    fn test_validate_name() {
        assert!(validate_key_name("my-key").is_ok());
        assert!(validate_key_name("").is_err());
        assert!(validate_key_name("key\nwith\nnewlines").is_err());
    }
}