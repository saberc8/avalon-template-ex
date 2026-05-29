use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};

use crate::{
    application::monitor::online_service::{OnlineService, OnlineUserQuery, OnlineUserResp},
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/monitor/online/:token", axum::routing::delete(kickout))
        .route("/monitor/online", get(page))
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<OnlineUserQuery>,
) -> Result<Json<ApiResponse<PageResult<OnlineUserResp>>>, AppError> {
    require_permission(&current_user, "monitor:online:list")?;
    let service = OnlineService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.page(&current_user, query).await?,
    )))
}

async fn kickout(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(token): Path<String>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "monitor:online:kickout")?;
    let service = OnlineService::new(state.db);
    service.kickout(token).await?;

    Ok(Json(ApiResponse::ok(true)))
}
