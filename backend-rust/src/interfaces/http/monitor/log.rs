use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    application::monitor::log_service::{
        log_csv, LogDetailResp, LogQuery, LogResp, LogService, LOGIN_LOG_TYPE, OPERATION_LOG_TYPE,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/log/export/login", get(export_login))
        .route("/system/log/export/operation", get(export_operation))
        .route("/system/log/:id", get(get_detail))
        .route("/system/log", get(page))
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LogQuery>,
) -> Result<Json<ApiResponse<PageResult<LogResp>>>, AppError> {
    require_permission(&current_user, "monitor:log:list")?;
    let service = LogService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.page(&current_user, query).await?,
    )))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<LogDetailResp>>, AppError> {
    require_permission(&current_user, "monitor:log:get")?;
    let service = LogService::new(state.db);

    Ok(Json(ApiResponse::ok(service.detail(id).await?)))
}

async fn export_login(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LogQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&current_user, "monitor:log:export")?;
    let service = LogService::new(state.db);
    let list = service.export(&current_user, query, LOGIN_LOG_TYPE).await?;

    Ok(csv_response("login_logs.csv", &log_csv(&list)))
}

async fn export_operation(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LogQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&current_user, "monitor:log:export")?;
    let service = LogService::new(state.db);
    let list = service
        .export(&current_user, query, OPERATION_LOG_TYPE)
        .await?;

    Ok(csv_response("operation_logs.csv", &log_csv(&list)))
}

fn csv_response(filename: &'static str, csv: &str) -> (HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            .expect("static csv filename is a valid header value"),
    );
    (headers, csv.to_owned())
}
