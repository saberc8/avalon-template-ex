use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::shared::config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionMode {
    Lazy,
    EagerWithMigrations,
}

pub async fn init_pool(config: &AppConfig) -> Result<PgPool> {
    let pool_options = PgPoolOptions::new().max_connections(config.database_max_connections);
    let pool = match connection_mode(config.db_auto_migrate) {
        ConnectionMode::Lazy => pool_options
            .connect_lazy(&config.database_url)
            .context("configure PostgreSQL pool")?,
        ConnectionMode::EagerWithMigrations => pool_options
            .connect(&config.database_url)
            .await
            .context("connect PostgreSQL")?,
    };

    if config.db_auto_migrate {
        run_migrations(&pool).await?;
    }

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("run PostgreSQL migrations")?;
    Ok(())
}

fn connection_mode(db_auto_migrate: bool) -> ConnectionMode {
    if db_auto_migrate {
        ConnectionMode::EagerWithMigrations
    } else {
        ConnectionMode::Lazy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_auto_migrate_uses_eager_connection_mode() {
        assert_eq!(connection_mode(true), ConnectionMode::EagerWithMigrations);
    }

    #[test]
    fn disabled_auto_migrate_uses_lazy_connection_mode() {
        assert_eq!(connection_mode(false), ConnectionMode::Lazy);
    }
}
