use argon2::{Argon2, Algorithm, Params, Version};
use crate::error::CryptoError;

/// Argon2id 密钥派生函数配置
pub struct KdfConfig {
    /// 内存成本 (KB)
    pub memory_cost: u32,
    /// 时间成本 (迭代次数)
    pub time_cost: u32,
    /// 并行度
    pub parallelism: u32,
    /// 输出长度 (字节)
    pub output_length: usize,
}

impl Default for KdfConfig {
    fn default() -> Self {
        Self {
            memory_cost: 65536,
            time_cost: 3,
            parallelism: 4,
            output_length: 32,
        }
    }
}

/// 使用 Argon2id 从密码派生密钥
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    config: &KdfConfig,
) -> Result<[u8; 32], CryptoError> {
    let params = Params::new(
        config.memory_cost,
        config.time_cost,
        config.parallelism,
        Some(config.output_length),
    )
    .map_err(|_| CryptoError::KdfError)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password, salt, &mut output)
        .map_err(|_| CryptoError::KdfError)?;

    Ok(output)
}

/// 密钥派生器封装
pub struct KeyDeriver {
    config: KdfConfig,
}

impl KeyDeriver {
    pub fn new() -> Self {
        Self {
            config: KdfConfig::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: KdfConfig) -> Self {
        Self { config }
    }

    /// 从密码派生密钥
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let key = derive_key(password.as_bytes(), salt, &self.config)?;
        Ok(key.to_vec())
    }
}

/// 生成随机盐值
pub fn generate_salt() -> [u8; 32] {
    use rand::RngCore;
    let mut salt = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let password = b"test_password";
        let salt = generate_salt();
        let config = KdfConfig::default();
        let key = derive_key(password, &salt, &config).unwrap();
        assert_ne!(key, [0u8; 32]);
    }

    #[test]
    fn test_same_password_same_salt_same_key() {
        let password = b"test_password";
        let salt = [1u8; 32];
        let config = KdfConfig::default();
        let key1 = derive_key(password, &salt, &config).unwrap();
        let key2 = derive_key(password, &salt, &config).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_password_different_key() {
        let salt = [1u8; 32];
        let config = KdfConfig::default();
        let key1 = derive_key(b"password1", &salt, &config).unwrap();
        let key2 = derive_key(b"password2", &salt, &config).unwrap();
        assert_ne!(key1, key2);
    }
}