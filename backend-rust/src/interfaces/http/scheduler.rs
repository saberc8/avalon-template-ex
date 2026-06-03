use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::scheduler::service::{
        JobCommand, JobLogQuery, JobLogResp, JobQuery, JobResp, JobStatusCommand, JobTriggerResp,
        SchedulerService,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::{middleware::permission::require_permission, system::IdsReq, AppState},
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/schedule/job/page", get(page))
        .route("/schedule/job/:id/log", get(log_page))
        .route("/schedule/job/:id/status", patch(update_status))
        .route("/schedule/job/:id/run", post(run_once))
        .route("/schedule/job/:id", get(get_detail).put(update))
        .route("/schedule/job", post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<JobQuery>,
) -> Result<Json<ApiResponse<PageResult<JobResp>>>, AppError> {
    require_permission(&current_user, "schedule:job:list")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);

    Ok(Json(ApiResponse::ok(service.page(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<JobResp>>, AppError> {
    require_permission(&current_user, "schedule:job:get")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<JobCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "schedule:job:create")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<JobCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "schedule:job:update")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "schedule:job:delete")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<JobStatusCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "schedule:job:updateStatus")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);
    service.update_status(id, command.status).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn run_once(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<JobTriggerResp>>, AppError> {
    require_permission(&current_user, "schedule:job:run")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);

    Ok(Json(ApiResponse::ok(service.run_once(id).await?)))
}

async fn log_page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Query(query): Query<JobLogQuery>,
) -> Result<Json<ApiResponse<PageResult<JobLogResp>>>, AppError> {
    require_permission(&current_user, "schedule:job:log:list")?;
    let service = SchedulerService::new(state.db, state.scheduler_http_safety);

    Ok(Json(ApiResponse::ok(service.log_page(id, query).await?)))
}
