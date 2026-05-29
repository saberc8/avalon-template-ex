use std::net::SocketAddr;

use backend_rust::{interfaces::http::build_router, shared::config::AppConfig};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect_lazy(&config.database_url)?;

    tracing::info!("postgres pool configured");
    // Database migrations are added in the next backend task.

    let app = build_router(db, &config.cors_allowed_origins)?;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.http_port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "rust admin backend listening");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("backend_rust=debug,tower_http=info,axum::rejection=trace")
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
