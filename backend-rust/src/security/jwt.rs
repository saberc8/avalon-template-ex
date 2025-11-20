use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT 载荷结构，与 Go 版 Claims 保持一致。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "userId")]
    pub user_id: i64,
    /// 签发时间（秒级时间戳）
    pub iat: u64,
    /// 过期时间（秒级时间戳）
    pub exp: u64,
}

/// TokenService 提供 JWT 生成与解析能力。
pub struct TokenService {
    secret: Vec<u8>,
    ttl: Duration,
}

impl TokenService {
    /// 创建 TokenService，ttl_hours <= 0 时默认 24 小时。
    pub fn new(secret: String, ttl_hours: i64) -> Self {
        let ttl = if ttl_hours <= 0 {
            Duration::from_secs(24 * 3600)
        } else {
            Duration::from_secs(ttl_hours as u64 * 3600)
        };
        Self {
            secret: secret.into_bytes(),
            ttl,
        }
    }

    /// 生成包含 userId 的 HS256 Token。
    pub fn generate(&self, user_id: i64) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let claims = Claims {
            user_id,
            iat: now,
            exp: now + self.ttl.as_secs(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
    }

    /// 解析并校验 Token，支持携带 "Bearer xxx" 前缀。
    pub fn parse(&self, token_str: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut token = token_str.trim();
        if token.to_lowercase().starts_with("bearer ") {
            token = token[7..].trim();
        }
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}

