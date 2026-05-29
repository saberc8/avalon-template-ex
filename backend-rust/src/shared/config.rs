use std::env;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
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

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_owned())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a positive integer")?;

        Ok(Self {
            http_port,
            database_url,
            database_max_connections,
        })
    }
}
