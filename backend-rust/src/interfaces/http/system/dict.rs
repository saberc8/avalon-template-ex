use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::system::dict_service::{
        DictCommand, DictItemCommand, DictItemQuery, DictItemResp, DictQuery, DictResp, DictService,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/dict/list", get(list))
        .route("/system/dict/cache/:code", delete(clear_cache))
        .route("/system/dict/item/:id", get(get_item).put(update_item))
        .route(
            "/system/dict/item",
            get(item_page).post(create_item).delete(delete_items),
        )
        .route("/system/dict/:id", get(get_detail).put(update))
        .route("/system/dict", post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<DictQuery>,
) -> Result<Json<ApiResponse<Vec<DictResp>>>, AppError> {
    require_permission(&current_user, "system:dict:list")?;
    let service = DictService::new(state.db);

    Ok(Json(ApiResponse::ok(service.list(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<DictResp>>, AppError> {
    require_permission(&current_user, "system:dict:get")?;
    let service = DictService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<DictCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:dict:create")?;
    let service = DictService::new(state.db);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<DictCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dict:update")?;
    let service = DictService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dict:delete")?;
    let service = DictService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn clear_cache(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dict:item:clearCache")?;
    let service = DictService::new(state.db);
    service.clear_cache(code).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn item_page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<DictItemQuery>,
) -> Result<Json<ApiResponse<PageResult<DictItemResp>>>, AppError> {
    require_permission(&current_user, "system:dict:item:list")?;
    let service = DictService::new(state.db);

    Ok(Json(ApiResponse::ok(service.item_page(query).await?)))
}

async fn get_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<DictItemResp>>, AppError> {
    require_permission(&current_user, "system:dict:item:get")?;
    let service = DictService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get_item(id).await?)))
}

async fn create_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<DictItemCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:dict:item:create")?;
    let service = DictService::new(state.db);
    let id = service.create_item(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update_item(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<DictItemCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dict:item:update")?;
    let service = DictService::new(state.db);
    service.update_item(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_items(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dict:item:delete")?;
    let service = DictService::new(state.db);
    service.delete_items(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}
