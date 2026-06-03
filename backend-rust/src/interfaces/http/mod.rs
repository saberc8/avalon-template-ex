use anyhow::{bail, Context, Result};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, HeaderValue, Method, Request, StatusCode},
    middleware::{from_fn, from_fn_with_state, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use crate::{
    application::scheduler::http_safety::HttpSafetyConfig,
    infrastructure::security::jwt::JwtService,
    shared::{error::AppError, response::ApiResponse},
};

pub mod auth;
pub mod captcha;
pub mod common;
pub mod extractor;
pub mod middleware {
    pub mod access_log;
    pub mod permission;
}
pub mod monitor;
pub mod scheduler;
pub mod system;
pub mod user_profile;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt: JwtService,
    pub captcha: captcha::CaptchaStore,
    pub scheduler_http_safety: HttpSafetyConfig,
}

pub fn build_router(
    db: PgPool,
    cors_allowed_origins: &[String],
    jwt: JwtService,
) -> Result<Router> {
    build_router_with_scheduler_http_safety(
        db,
        cors_allowed_origins,
        jwt,
        HttpSafetyConfig::default(),
    )
}

pub fn build_router_with_scheduler_http_safety(
    db: PgPool,
    cors_allowed_origins: &[String],
    jwt: JwtService,
    scheduler_http_safety: HttpSafetyConfig,
) -> Result<Router> {
    let cors = cors_layer(cors_allowed_origins)?;
    let state = AppState {
        db,
        jwt,
        captcha: captcha::CaptchaStore::default(),
        scheduler_http_safety,
    };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .merge(auth::routes())
        .merge(captcha::routes())
        .merge(common::routes())
        .merge(monitor::log::routes())
        .merge(monitor::online::routes())
        .merge(scheduler::routes())
        .merge(system::dept::routes())
        .merge(system::dict::routes())
        .merge(system::file::routes())
        .merge(system::client::routes())
        .merge(system::menu::routes())
        .merge(system::option::routes())
        .merge(system::role::routes())
        .merge(system::storage::routes())
        .merge(system::user::routes())
        .merge(user_profile::routes())
        .nest_service("/file", ServeDir::new("data/file"))
        .fallback(not_found)
        .with_state(state.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(from_fn_with_state(
            state,
            middleware::access_log::record_access_log,
        ))
        .layer(from_fn(vue_failure_envelope)))
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

async fn not_found() -> Json<ApiResponse<()>> {
    Json(ApiResponse::fail("404", "请求的资源不存在"))
}

async fn ready(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(ApiResponse::ok("ready")))
}

async fn vue_failure_envelope(request: Request<Body>, next: Next) -> Response {
    let is_preflight = request.method() == Method::OPTIONS;
    let response = next.run(request).await;

    if is_preflight || response.status() == StatusCode::OK {
        return response;
    }

    let status = response.status();
    let headers = response.headers().clone();
    let mut wrapped = Json(ApiResponse::fail(
        status.as_u16().to_string(),
        fallback_message(status),
    ))
    .into_response();
    for (name, value) in headers.iter() {
        if should_preserve_failure_header(name) {
            wrapped.headers_mut().insert(name.clone(), value.clone());
        }
    }

    wrapped
}

fn should_preserve_failure_header(name: &HeaderName) -> bool {
    name != header::CONTENT_TYPE && name != header::CONTENT_LENGTH
}

fn fallback_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "请求参数错误",
        StatusCode::UNAUTHORIZED => "未授权，请重新登录",
        StatusCode::FORBIDDEN => "没有访问权限，请联系管理员授权",
        StatusCode::NOT_FOUND => "请求的资源不存在",
        StatusCode::METHOD_NOT_ALLOWED => "请求方法不支持",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "请求内容类型不支持",
        StatusCode::PAYLOAD_TOO_LARGE => "请求体过大",
        StatusCode::INTERNAL_SERVER_ERROR => "系统异常，请稍后重试",
        _ => status.canonical_reason().unwrap_or("请求失败"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    fn test_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost:5432/avalon_admin")
            .unwrap()
    }

    fn test_jwt() -> JwtService {
        JwtService::new("test-secret".to_owned(), 24)
    }

    #[tokio::test]
    async fn health_route_returns_success_envelope() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

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
        assert_eq!(body["code"], "200");
        assert_eq!(body["msg"], "成功");
        assert_eq!(body["data"], "ok");
        assert_eq!(body["success"], true);
        assert!(body["timestamp"].as_str().unwrap().parse::<i64>().is_ok());
    }

    #[tokio::test]
    async fn ready_route_returns_generic_error_when_database_is_unreachable() {
        let db = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/avalon_admin")
            .unwrap();
        let app = build_router(db, &["http://localhost:4399".to_owned()], test_jwt()).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "500");
        assert_eq!(body["msg"], "系统异常，请稍后重试");
        assert_eq!(body["data"], Value::Null);
        assert_eq!(body["success"], false);
        assert!(body["timestamp"].as_str().unwrap().parse::<i64>().is_ok());
    }

    #[tokio::test]
    async fn unmatched_route_returns_vue_failure_envelope() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/route/that/does/not/exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "404");
        assert_eq!(body["data"], Value::Null);
        assert_eq!(body["success"], false);
        assert!(body["timestamp"].as_str().unwrap().parse::<i64>().is_ok());
    }

    #[tokio::test]
    async fn axum_rejections_return_vue_failure_envelope() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "400");
        assert_eq!(body["data"], Value::Null);
        assert_eq!(body["success"], false);
        assert!(body["timestamp"].as_str().unwrap().parse::<i64>().is_ok());
    }

    #[tokio::test]
    async fn wrapped_axum_rejection_preserves_cors_headers() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/auth/login")
                    .header(header::ORIGIN, "http://localhost:4399")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"http://localhost:4399".parse().unwrap())
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "400");
        assert_eq!(body["success"], false);
    }

    #[tokio::test]
    async fn method_not_allowed_returns_vue_failure_envelope() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "405");
        assert_eq!(body["data"], Value::Null);
        assert_eq!(body["success"], false);
        assert!(body["timestamp"].as_str().unwrap().parse::<i64>().is_ok());
    }

    #[tokio::test]
    async fn cors_allows_configured_origins_only() {
        let app = build_router(
            test_pool(),
            &["http://localhost:4399".to_owned()],
            test_jwt(),
        )
        .unwrap();

        let allowed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header(header::ORIGIN, "http://localhost:4399")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            allowed.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&"http://localhost:4399".parse().unwrap())
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
