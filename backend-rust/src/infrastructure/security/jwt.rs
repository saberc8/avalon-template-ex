use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JwtService {
    secret: String,
    ttl_hours: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub expire: DateTime<Utc>,
}

impl JwtService {
    pub fn new(secret: String, ttl_hours: u64) -> Self {
        Self { secret, ttl_hours }
    }

    pub fn issue(&self, user_id: i64, username: &str) -> Result<String> {
        Ok(self.issue_with_expire(user_id, username)?.token)
    }

    pub fn issue_with_expire(&self, user_id: i64, username: &str) -> Result<IssuedToken> {
        let now = Utc::now();
        let ttl_hours = i64::try_from(self.ttl_hours).context("JWT TTL is too large")?;
        let expire = now + Duration::hours(ttl_hours);
        let claims = TokenClaims {
            user_id,
            username: username.to_owned(),
            iat: now.timestamp() as usize,
            exp: expire.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .context("issue JWT")?;

        Ok(IssuedToken { token, expire })
    }

    pub fn parse(&self, authorization: &str) -> Result<TokenClaims> {
        let token = bearer_token(authorization)?;
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .context("parse JWT")?;

        Ok(token_data.claims)
    }
}

fn bearer_token(authorization: &str) -> Result<&str> {
    let value = authorization.trim();
    if value.is_empty() {
        bail!("missing authorization token");
    }

    let token = value
        .split_once(' ')
        .and_then(|(scheme, token)| {
            if scheme.eq_ignore_ascii_case("bearer") {
                Some(token.trim())
            } else {
                None
            }
        })
        .unwrap_or(value);

    if token.is_empty() {
        bail!("missing bearer token");
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn token_round_trips_user_id() {
        let service = JwtService::new("secret".to_string(), 24);
        let token = service.issue(1, "admin").unwrap();
        let claims = service.parse(&format!("Bearer {token}")).unwrap();
        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "admin");
    }

    #[tokio::test]
    async fn expired_token_is_rejected() {
        let secret = "local-dev-only-change-this-secret-32chars-min";
        let claims = TokenClaims {
            user_id: 1,
            username: "admin".to_owned(),
            iat: (Utc::now() - Duration::hours(2)).timestamp() as usize,
            exp: (Utc::now() - Duration::hours(1)).timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        let service = JwtService::new(secret.to_owned(), 24);

        let err = service.parse(&format!("Bearer {token}")).unwrap_err();

        assert!(err.to_string().contains("parse JWT"));
    }

    #[tokio::test]
    async fn token_signed_with_wrong_secret_is_rejected() {
        let issuer = JwtService::new(
            "local-dev-only-change-this-secret-32chars-min".to_owned(),
            24,
        );
        let parser = JwtService::new("another-local-dev-only-secret-32chars-min".to_owned(), 24);
        let token = issuer.issue(1, "admin").unwrap();

        let err = parser.parse(&format!("Bearer {token}")).unwrap_err();

        assert!(err.to_string().contains("parse JWT"));
    }
}
