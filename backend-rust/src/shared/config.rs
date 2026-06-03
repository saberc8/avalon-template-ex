use std::env;

use anyhow::{bail, Context, Result};

const DEFAULT_CORS_ALLOWED_ORIGINS: &str =
    "http://localhost:4399,http://127.0.0.1:4399,http://localhost:5173,http://127.0.0.1:5173";
const JWT_SECRET_PLACEHOLDER: &str = "dev-only-change-me";
const JWT_SECRET_MIN_LEN: usize = 32;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub db_auto_migrate: bool,
    pub cors_allowed_origins: Vec<String>,
    pub auth_jwt_secret: String,
    pub auth_jwt_ttl_hours: u64,
    pub scheduler_embedded: bool,
    pub scheduler_worker_enabled: bool,
    pub scheduler_tick_seconds: u64,
    pub scheduler_batch_size: i64,
    pub scheduler_worker_id: String,
    pub scheduler_http_allowlist_mode: String,
    pub scheduler_http_allowlist: Vec<String>,
    pub rabbitmq_url: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_execute_queue: String,
    pub rabbitmq_retry_queue: String,
    pub rabbitmq_dead_queue: String,
    pub rabbitmq_execute_routing_key: String,
    pub rabbitmq_retry_routing_key: String,
    pub rabbitmq_dead_routing_key: String,
    pub rabbitmq_retry_ttl_ms: u32,
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
        let auth_jwt_secret_raw = env::var("AUTH_JWT_SECRET").ok();
        let auth_jwt_secret = parse_auth_jwt_secret(auth_jwt_secret_raw.as_deref())?;
        let auth_jwt_ttl_hours = parse_positive_u64_env(
            "AUTH_JWT_TTL_HOURS",
            &env::var("AUTH_JWT_TTL_HOURS").unwrap_or_else(|_| "24".to_owned()),
        )?;
        let scheduler_embedded =
            parse_bool_env(env::var("SCHEDULER_EMBEDDED").ok().as_deref(), false)?;
        let scheduler_worker_enabled =
            parse_bool_env(env::var("SCHEDULER_WORKER_ENABLED").ok().as_deref(), true)?;
        let scheduler_tick_seconds = parse_positive_u64_env(
            "SCHEDULER_TICK_SECONDS",
            &env::var("SCHEDULER_TICK_SECONDS").unwrap_or_else(|_| "5".to_owned()),
        )?;
        let scheduler_batch_size = parse_positive_i64_env(
            "SCHEDULER_BATCH_SIZE",
            &env::var("SCHEDULER_BATCH_SIZE").unwrap_or_else(|_| "50".to_owned()),
        )?;
        let scheduler_worker_id = env::var("SCHEDULER_WORKER_ID")
            .unwrap_or_else(|_| format!("worker-{}", std::process::id()));
        let scheduler_http_allowlist_mode =
            env::var("SCHEDULER_HTTP_ALLOWLIST_MODE").unwrap_or_else(|_| "default".to_owned());
        let scheduler_http_allowlist =
            parse_cors_allowed_origins(&env::var("SCHEDULER_HTTP_ALLOWLIST").unwrap_or_default());
        let rabbitmq_url = env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".to_owned());
        let rabbitmq_exchange =
            env::var("RABBITMQ_EXCHANGE").unwrap_or_else(|_| "avalon.scheduler".to_owned());
        let rabbitmq_execute_queue = env::var("RABBITMQ_SCHEDULER_EXECUTE_QUEUE")
            .unwrap_or_else(|_| "avalon.scheduler.execute".to_owned());
        let rabbitmq_retry_queue = env::var("RABBITMQ_SCHEDULER_RETRY_QUEUE")
            .unwrap_or_else(|_| "avalon.scheduler.retry".to_owned());
        let rabbitmq_dead_queue = env::var("RABBITMQ_SCHEDULER_DEAD_QUEUE")
            .unwrap_or_else(|_| "avalon.scheduler.dead".to_owned());
        let rabbitmq_execute_routing_key = env::var("RABBITMQ_SCHEDULER_EXECUTE_ROUTING_KEY")
            .unwrap_or_else(|_| "scheduler.execute".to_owned());
        let rabbitmq_retry_routing_key = env::var("RABBITMQ_SCHEDULER_RETRY_ROUTING_KEY")
            .unwrap_or_else(|_| "scheduler.retry".to_owned());
        let rabbitmq_dead_routing_key = env::var("RABBITMQ_SCHEDULER_DEAD_ROUTING_KEY")
            .unwrap_or_else(|_| "scheduler.dead".to_owned());
        let rabbitmq_retry_ttl_ms = parse_positive_u32_env(
            "RABBITMQ_SCHEDULER_RETRY_TTL_MS",
            &env::var("RABBITMQ_SCHEDULER_RETRY_TTL_MS").unwrap_or_else(|_| "30000".to_owned()),
        )?;

        if cors_allowed_origins.is_empty() {
            bail!("CORS_ALLOWED_ORIGINS must include at least one origin");
        }

        Ok(Self {
            http_port,
            database_url,
            database_max_connections,
            db_auto_migrate,
            cors_allowed_origins,
            auth_jwt_secret,
            auth_jwt_ttl_hours,
            scheduler_embedded,
            scheduler_worker_enabled,
            scheduler_tick_seconds,
            scheduler_batch_size,
            scheduler_worker_id,
            scheduler_http_allowlist_mode,
            scheduler_http_allowlist,
            rabbitmq_url,
            rabbitmq_exchange,
            rabbitmq_execute_queue,
            rabbitmq_retry_queue,
            rabbitmq_dead_queue,
            rabbitmq_execute_routing_key,
            rabbitmq_retry_routing_key,
            rabbitmq_dead_routing_key,
            rabbitmq_retry_ttl_ms,
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

fn parse_positive_u64_env(name: &str, raw: &str) -> Result<u64> {
    let value = raw
        .parse::<u64>()
        .with_context(|| format!("{name} must be a positive integer"))?;

    if value == 0 {
        bail!("{name} must be a positive integer");
    }

    Ok(value)
}

fn parse_positive_i64_env(name: &str, raw: &str) -> Result<i64> {
    let value = raw
        .parse::<i64>()
        .with_context(|| format!("{name} must be a positive integer"))?;

    if value <= 0 {
        bail!("{name} must be a positive integer");
    }

    Ok(value)
}

fn parse_positive_u32_env(name: &str, raw: &str) -> Result<u32> {
    let value = raw
        .parse::<u32>()
        .with_context(|| format!("{name} must be a positive integer"))?;

    if value == 0 {
        bail!("{name} must be a positive integer");
    }

    Ok(value)
}

fn parse_auth_jwt_secret(raw: Option<&str>) -> Result<String> {
    let Some(raw) = raw else {
        bail!("AUTH_JWT_SECRET must be set");
    };
    let secret = raw.trim();

    if secret.is_empty() {
        bail!("AUTH_JWT_SECRET must not be empty");
    }
    if secret == JWT_SECRET_PLACEHOLDER {
        bail!("AUTH_JWT_SECRET must not use the placeholder value");
    }
    if secret.len() < JWT_SECRET_MIN_LEN {
        bail!("AUTH_JWT_SECRET must be at least {JWT_SECRET_MIN_LEN} characters");
    }

    Ok(secret.to_owned())
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

    #[test]
    fn default_cors_allowed_origins_include_nextjs_dev_port() {
        let origins = parse_cors_allowed_origins(DEFAULT_CORS_ALLOWED_ORIGINS);

        assert!(origins.contains(&"http://localhost:4399".to_owned()));
        assert!(origins.contains(&"http://127.0.0.1:4399".to_owned()));
    }

    #[test]
    fn jwt_ttl_rejects_zero() {
        let err = parse_positive_u64_env("AUTH_JWT_TTL_HOURS", "0").unwrap_err();

        assert!(err.to_string().contains("positive"));
    }

    #[test]
    fn jwt_secret_rejects_missing_value() {
        let err = parse_auth_jwt_secret(None).unwrap_err();

        assert!(err.to_string().contains("AUTH_JWT_SECRET"));
    }

    #[test]
    fn jwt_secret_rejects_empty_value() {
        let err = parse_auth_jwt_secret(Some("   ")).unwrap_err();

        assert!(err.to_string().contains("AUTH_JWT_SECRET"));
    }

    #[test]
    fn jwt_secret_rejects_placeholder_value() {
        let err = parse_auth_jwt_secret(Some("dev-only-change-me")).unwrap_err();

        assert!(err.to_string().contains("placeholder"));
    }

    #[test]
    fn jwt_secret_rejects_short_value() {
        let err = parse_auth_jwt_secret(Some("short-secret")).unwrap_err();

        assert!(err.to_string().contains("at least 32"));
    }

    #[test]
    fn jwt_secret_accepts_valid_value() {
        let secret =
            parse_auth_jwt_secret(Some("local-dev-only-change-this-secret-32chars-min")).unwrap();

        assert_eq!(secret, "local-dev-only-change-this-secret-32chars-min");
    }

    #[test]
    fn scheduler_batch_size_rejects_zero() {
        let err = parse_positive_i64_env("SCHEDULER_BATCH_SIZE", "0").unwrap_err();

        assert!(err.to_string().contains("positive"));
    }
}
