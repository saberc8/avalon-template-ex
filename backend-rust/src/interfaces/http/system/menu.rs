use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::system::menu_service::{MenuCommand, MenuQuery, MenuResp, MenuService},
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/menu/tree", get(list_tree))
        .route("/system/menu/cache", delete(clear_cache))
        .route("/system/menu/:id", get(get_detail).put(update))
        .route("/system/menu", post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn list_tree(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<MenuQuery>,
) -> Result<Json<ApiResponse<Vec<MenuResp>>>, AppError> {
    require_permission(&current_user, "system:menu:list")?;
    let service = MenuService::new(state.db);

    Ok(Json(ApiResponse::ok(service.tree(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<MenuResp>>, AppError> {
    require_permission(&current_user, "system:menu:get")?;
    let service = MenuService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<MenuCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:menu:create")?;
    let service = MenuService::new(state.db);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<MenuCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:menu:update")?;
    let service = MenuService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:menu:delete")?;
    let service = MenuService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn clear_cache(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:menu:clearCache")?;
    let service = MenuService::new(state.db);
    service.clear_cache().await?;

    Ok(Json(ApiResponse::ok(true)))
}
