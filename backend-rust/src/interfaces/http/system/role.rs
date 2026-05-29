use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::system::role_service::{
        RoleCommand, RoleDetailResp, RolePermissionCommand, RoleQuery, RoleResp, RoleService,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/role/list", get(list))
        .route("/system/role/:id/permission", put(update_permission))
        .route("/system/role/:id/user/id", get(list_user_ids))
        .route("/system/role/:id", get(get_detail).put(update))
        .route("/system/role", post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<RoleQuery>,
) -> Result<Json<ApiResponse<Vec<RoleResp>>>, AppError> {
    require_permission(&current_user, "system:role:list")?;
    let service = RoleService::new(state.db);

    Ok(Json(ApiResponse::ok(service.list(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<RoleDetailResp>>, AppError> {
    require_permission(&current_user, "system:role:get")?;
    let service = RoleService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<RoleCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:role:create")?;
    let service = RoleService::new(state.db);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<RoleCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:role:update")?;
    let service = RoleService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:role:delete")?;
    let service = RoleService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_permission(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<RolePermissionCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:role:updatePermission")?;
    let service = RoleService::new(state.db);
    service
        .update_permission(current_user.id, id, command)
        .await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn list_user_ids(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<i64>>>, AppError> {
    require_permission(&current_user, "system:role:assign")?;
    let service = RoleService::new(state.db);

    Ok(Json(ApiResponse::ok(service.user_ids(id).await?)))
}
