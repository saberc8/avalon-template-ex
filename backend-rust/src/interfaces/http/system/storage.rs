use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::system::storage_service::{
        StorageCommand, StorageQuery, StorageResp, StorageService, StorageStatusCommand,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/storage/list", get(list))
        .route(
            "/system/storage/:id/status",
            axum::routing::put(update_status),
        )
        .route(
            "/system/storage/:id/default",
            axum::routing::put(set_default),
        )
        .route("/system/storage/:id", get(get_detail).put(update))
        .route("/system/storage", post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<StorageQuery>,
) -> Result<Json<ApiResponse<Vec<StorageResp>>>, AppError> {
    require_permission(&current_user, "system:storage:list")?;
    let service = StorageService::new(state.db);

    Ok(Json(ApiResponse::ok(service.list(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<StorageResp>>, AppError> {
    require_permission(&current_user, "system:storage:get")?;
    let service = StorageService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<StorageCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:storage:create")?;
    let service = StorageService::new(state.db);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<StorageCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:storage:update")?;
    let service = StorageService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:storage:delete")?;
    let service = StorageService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<StorageStatusCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:storage:updateStatus")?;
    let service = StorageService::new(state.db);
    service.update_status(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn set_default(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:storage:setDefault")?;
    let service = StorageService::new(state.db);
    service.set_default(id).await?;

    Ok(Json(ApiResponse::ok(true)))
}
