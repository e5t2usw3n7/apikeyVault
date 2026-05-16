use std::time::Instant;

/// 模型商 API 测试端点
struct ProviderEndpoint {
    name: &'static str,
    test_url: &'static str,
    auth_header: &'static str,
    auth_prefix: &'static str,
}

/// 内置支持的模型商列表
fn builtin_providers() -> Vec<ProviderEndpoint> {
    vec![
        ProviderEndpoint {
            name: "deepseek",
            test_url: "https://api.deepseek.com/v1/models",
            auth_header: "Authorization",
            auth_prefix: "Bearer ",
        },
        ProviderEndpoint {
            name: "openai",
            test_url: "https://api.openai.com/v1/models",
            auth_header: "Authorization",
            auth_prefix: "Bearer ",
        },
        ProviderEndpoint {
            name: "anthropic",
            test_url: "https://api.anthropic.com/v1/messages",
            auth_header: "x-api-key",
            auth_prefix: "",
        },
        ProviderEndpoint {
            name: "google",
            test_url: "https://generativelanguage.googleapis.com/v1/models",
            auth_header: "x-goog-api-key",
            auth_prefix: "",
        },
        ProviderEndpoint {
            name: "gemini",
            test_url: "https://generativelanguage.googleapis.com/v1/models",
            auth_header: "x-goog-api-key",
            auth_prefix: "",
        },
    ]
}

/// 连通性测试结果
#[derive(Debug, Clone)]
pub struct ConnectivityResult {
    pub provider: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: Option<u64>,
}

/// 构建认证头值，智能检测 Bearer 前缀避免重复
fn build_auth_value(api_key: &str, prefix: &str) -> String {
    if !prefix.is_empty() && api_key.to_lowercase().starts_with(&prefix.to_lowercase()) {
        api_key.to_string()
    } else {
        format!("{}{}", prefix, api_key)
    }
}

/// 测试 API Key 连通性
pub fn test_connectivity(api_key: &str, provider: &str, base_url: Option<&str>) -> ConnectivityResult {
    let provider_lower = provider.to_lowercase();

    // 匹配内置模型商
    let endpoint = builtin_providers().into_iter().find(|p| p.name == provider_lower);

    let (test_url, auth_header, auth_value) = if let Some(ep) = endpoint {
        let auth_val = build_auth_value(api_key, ep.auth_prefix);
        (ep.test_url.to_string(), ep.auth_header.to_string(), auth_val)
    } else if let Some(url) = base_url {
        // 回退到通用 OpenAI-compatible 端点
        let base = url.trim_end_matches('/');
        let auth_val = build_auth_value(api_key, "Bearer ");
        (format!("{}/v1/models", base), "Authorization".to_string(), auth_val)
    } else {
        return ConnectivityResult {
            provider: provider.to_string(),
            success: false,
            status_code: None,
            message: format!("不支持的提供商 '{}'，请在 metadata 中配置 base_url", provider),
            latency_ms: None,
        };
    };

    // 发送 HTTP 请求
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return ConnectivityResult {
                provider: provider.to_string(),
                success: false,
                status_code: None,
                message: format!("创建 HTTP 客户端失败: {}", e),
                latency_ms: None,
            };
        }
    };

    let start = Instant::now();

    // Google/Gemini 使用 query 参数传递 key，不走 header
    let response = if provider_lower == "google" || provider_lower == "gemini" {
        client
            .get(&test_url)
            .query(&[("key", api_key)])
            .send()
    } else {
        client
            .get(&test_url)
            .header(&auth_header, &auth_value)
            .send()
    };

    let latency = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let success = resp.status().is_success();
            let message = if success {
                "连通成功".to_string()
            } else {
                format!("HTTP {}: {}", status, resp.status().canonical_reason().unwrap_or("未知错误"))
            };
            ConnectivityResult {
                provider: provider.to_string(),
                success,
                status_code: Some(status),
                message,
                latency_ms: Some(latency),
            }
        }
        Err(e) => {
            let message = if e.is_timeout() {
                "请求超时（10秒）".to_string()
            } else if e.is_connect() {
                format!("连接失败: {}", e)
            } else {
                format!("请求失败: {}", e)
            };
            ConnectivityResult {
                provider: provider.to_string(),
                success: false,
                status_code: None,
                message,
                latency_ms: Some(latency),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_value_no_prefix() {
        assert_eq!(build_auth_value("sk-abc123", ""), "sk-abc123");
    }

    #[test]
    fn test_build_auth_value_with_prefix() {
        assert_eq!(build_auth_value("sk-abc123", "Bearer "), "Bearer sk-abc123");
    }

    #[test]
    fn test_build_auth_value_already_has_prefix() {
        assert_eq!(build_auth_value("Bearer sk-abc123", "Bearer "), "Bearer sk-abc123");
    }

    #[test]
    fn test_build_auth_value_case_insensitive() {
        assert_eq!(build_auth_value("bearer sk-abc123", "Bearer "), "bearer sk-abc123");
    }

    #[test]
    fn test_unsupported_provider() {
        let result = test_connectivity("test-key", "unknown_provider", None);
        assert!(!result.success);
        assert!(result.message.contains("不支持的提供商"));
    }
}
