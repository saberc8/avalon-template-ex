use std::env;

use anyhow::{bail, Context, Result};

const DEFAULT_CORS_ALLOWED_ORIGINS: &str =
    "http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub db_auto_migrate: bool,
    pub cors_allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let http_port = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "4398".to_owned())
            .parse::<u16>()
            .context("HTTP_PORT must be a valid TCP port")?;

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/avalon_admin".to_owned()
        });

        let database_max_connections = parse_database_max_connections(
            &env::var("DATABASE_MAX_CONNECTIONS").unwrap_or_else(|_| "5".to_owned()),
        )?;
        let db_auto_migrate_raw = env::var("DB_AUTO_MIGRATE").ok();
        let db_auto_migrate = parse_bool_env(db_auto_migrate_raw.as_deref(), false)?;
        let cors_allowed_origins = parse_cors_allowed_origins(
            &env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| DEFAULT_CORS_ALLOWED_ORIGINS.to_owned()),
        );

        if cors_allowed_origins.is_empty() {
            bail!("CORS_ALLOWED_ORIGINS must include at least one origin");
        }

        Ok(Self {
            http_port,
            database_url,
            database_max_connections,
            db_auto_migrate,
            cors_allowed_origins,
        })
    }
}

fn parse_database_max_connections(raw: &str) -> Result<u32> {
    let value = raw
        .parse::<u32>()
        .context("DATABASE_MAX_CONNECTIONS must be a positive integer")?;

    if value == 0 {
        bail!("DATABASE_MAX_CONNECTIONS must be a positive integer");
    }

    Ok(value)
}

fn parse_cors_allowed_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_bool_env(raw: Option<&str>, default: bool) -> Result<bool> {
    let Some(raw) = raw else {
        return Ok(default);
    };

    match raw.trim().to_ascii_lowercase().as_str() {
        "" => Ok(default),
        "true" | "1" | "yes" | "y" | "on" => Ok(true),
        "false" | "0" | "no" | "n" | "off" => Ok(false),
        _ => bail!("DB_AUTO_MIGRATE must be a boolean"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_max_connections_rejects_zero() {
        let err = parse_database_max_connections("0").unwrap_err();

        assert!(err.to_string().contains("positive"));
    }

    #[test]
    fn db_auto_migrate_defaults_to_false() {
        assert!(!parse_bool_env(None, false).unwrap());
    }

    #[test]
    fn db_auto_migrate_accepts_true() {
        assert!(parse_bool_env(Some("true"), false).unwrap());
        assert!(parse_bool_env(Some("1"), false).unwrap());
    }
}
