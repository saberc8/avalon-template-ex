use sqlx::postgres::PgPoolOptions;

use crate::config::AppConfig;

/// PostgreSQL 连接池类型别名。
pub type DbPool = sqlx::Pool<sqlx::Postgres>;

/// 基于 AppConfig 创建 PostgreSQL 连接池。
pub async fn create_pool(cfg: &AppConfig) -> Result<DbPool, sqlx::Error> {
    // 这里的连接池配置保持简单，后续如需更高并发可再调整参数。
    PgPoolOptions::new()
        .max_connections(20)
        .connect_timeout(std::time::Duration::from_secs(10))
        .connect(&cfg.database_url())
        .await
}

