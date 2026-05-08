use regex::Regex;
use serde::{Deserialize, Serialize};

/// 密钥模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyTemplate {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub provider: String,
    pub docs_url: Option<String>,
    pub example: Option<String>,
}

impl KeyTemplate {
    /// 验证密钥值是否匹配模板的正则表达式
    pub fn validate(&self, value: &str) -> bool {
        match Regex::new(&self.pattern) {
            Ok(re) => re.is_match(value),
            Err(_) => false,
        }
    }
}

/// 预定义密钥模板
pub fn builtin_templates() -> Vec<KeyTemplate> {
    vec![
        KeyTemplate {
            name: "OpenAI API Key".to_string(),
            pattern: r"^sk-[a-zA-Z0-9]{20,}$".to_string(),
            description: "OpenAI GPT API Key".to_string(),
            provider: "OpenAI".to_string(),
            docs_url: Some("https://platform.openai.com/api-keys".to_string()),
            example: Some("sk-...".to_string()),
        },
        KeyTemplate {
            name: "AWS Access Key".to_string(),
            pattern: r"^AKIA[0-9A-Z]{16}$".to_string(),
            description: "AWS Access Key ID".to_string(),
            provider: "AWS".to_string(),
            docs_url: Some("https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_access-keys.html".to_string()),
            example: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
        },
        KeyTemplate {
            name: "GitHub Token".to_string(),
            pattern: r"^(ghp|gho|ghu|ghs|ghr)_[a-zA-Z0-9]{36}$".to_string(),
            description: "GitHub Personal Access Token".to_string(),
            provider: "GitHub".to_string(),
            docs_url: Some("https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens".to_string()),
            example: Some("ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string()),
        },
        KeyTemplate {
            name: "Google API Key".to_string(),
            pattern: r"^AIza[0-9A-Za-z_-]{35}$".to_string(),
            description: "Google Cloud API Key".to_string(),
            provider: "Google".to_string(),
            docs_url: Some("https://cloud.google.com/docs/authentication/api-keys".to_string()),
            example: Some("AIzaSyD...".to_string()),
        },
        KeyTemplate {
            name: "Slack Token".to_string(),
            pattern: r"^xox[bpoas]-[0-9]{10,}-[0-9a-zA-Z-]+$".to_string(),
            description: "Slack Bot/User Token".to_string(),
            provider: "Slack".to_string(),
            docs_url: Some("https://api.slack.com/authentication/token-types".to_string()),
            example: Some("xoxb-...".to_string()),
        },
        KeyTemplate {
            name: "Stripe API Key".to_string(),
            pattern: r"^(sk|pk)_(test|live)_[a-zA-Z0-9]{20,}$".to_string(),
            description: "Stripe Secret/Publishable Key".to_string(),
            provider: "Stripe".to_string(),
            docs_url: Some("https://stripe.com/docs/keys".to_string()),
            example: Some("sk_test_...".to_string()),
        },
    ]
}

#[cfg(test)]
pub fn find_template(name: &str) -> Option<KeyTemplate> {
    builtin_templates().into_iter().find(|t| {
        t.name.to_lowercase() == name.to_lowercase()
            || t.provider.to_lowercase() == name.to_lowercase()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openai_key() {
        let template = find_template("OpenAI").unwrap();
        assert!(template.validate("sk-abcdefghijklmnopqrstuvwxyz1234567890"));
        assert!(!template.validate("invalid-key"));
    }

    #[test]
    fn test_validate_aws_key() {
        let template = find_template("AWS").unwrap();
        assert!(template.validate("AKIAIOSFODNN7EXAMPLE"));
        assert!(!template.validate("invalid-key"));
    }

    #[test]
    fn test_validate_github_token() {
        let template = find_template("GitHub").unwrap();
        assert!(template.validate("ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"));
        assert!(!template.validate("invalid-token"));
    }

    #[test]
    fn test_find_template_case_insensitive() {
        assert!(find_template("openai").is_some());
        assert!(find_template("OPENAI").is_some());
        assert!(find_template("nonexistent").is_none());
    }
}