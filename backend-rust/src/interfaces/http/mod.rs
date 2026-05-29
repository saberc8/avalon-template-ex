use anyhow::{bail, Context, Result};
use axum::{
    extract::State,
    http::{header, HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::shared::{error::AppError, response::ApiResponse};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn build_router(db: PgPool, cors_allowed_origins: &[String]) -> Result<Router> {
    let cors = cors_layer(cors_allowed_origins)?;

    Ok(Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(AppState { db })
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}

fn cors_layer(cors_allowed_origins: &[String]) -> Result<CorsLayer> {
    if cors_allowed_origins.is_empty() {
        bail!("CORS_ALLOWED_ORIGINS must include at least one origin");
    }

    let origins = cors_allowed_origins
        .iter()
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .with_context(|| format!("invalid CORS allowed origin: {origin}"))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("ok"))
}

async fn ready(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(ApiResponse::ok("ready")))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    fn test_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost:5432/avalon_admin")
            .unwrap()
    }

    #[tokio::test]
    async fn health_route_returns_success_envelope() {
        let app = build_router(test_pool(), &["http://localhost:3000".to_owned()]).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body, json!({"code": "200", "msg": "成功", "data": "ok"}));
    }

    #[tokio::test]
    async fn ready_route_returns_generic_error_when_database_is_unreachable() {
        let db = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/avalon_admin")
            .unwrap();
        let app = build_router(db, &["http://localhost:3000".to_owned()]).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(
            body,
            json!({"code": "500", "msg": "系统异常，请稍后重试", "data": null})
        );
    }

    #[tokio::test]
    async fn cors_allows_configured_origins_only() {
        let app = build_router(test_pool(), &["http://localhost:3000".to_owned()]).unwrap();

        let allowed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header(header::ORIGIN, "http://localhost:3000")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            allowed.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"http://localhost:3000".parse().unwrap())
        );

        let disallowed = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(disallowed
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
    }
}
