use axum::{routing::get, Json, Router};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::shared::response::ApiResponse;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn build_router(db: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(AppState { db })
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("ok"))
}
