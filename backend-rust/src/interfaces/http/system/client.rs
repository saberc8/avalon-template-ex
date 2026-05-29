use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::{
    application::system::client_service::{ClientCommand, ClientQuery, ClientResp, ClientService},
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/client/:id", get(get_detail).put(update))
        .route("/system/client", get(page).post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<ClientQuery>,
) -> Result<Json<ApiResponse<PageResult<ClientResp>>>, AppError> {
    require_permission(&current_user, "system:client:list")?;
    let service = ClientService::new(state.db);

    Ok(Json(ApiResponse::ok(service.page(query).await?)))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ClientResp>>, AppError> {
    require_permission(&current_user, "system:client:get")?;
    let service = ClientService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<ClientCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:client:create")?;
    let service = ClientService::new(state.db);
    let id = service.create(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<ClientCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:client:update")?;
    let service = ClientService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:client:delete")?;
    let service = ClientService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}
