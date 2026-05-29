use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    application::system::dept_service::{DeptCommand, DeptQuery, DeptResp, DeptService},
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/dept/tree", get(list_tree))
        .route("/system/dept/export", get(export))
        .route("/system/dept/:id", get(get_detail).put(update))
        .route("/system/dept", post(create).delete(delete_many))
}

async fn list_tree(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<DeptQuery>,
) -> Result<Json<ApiResponse<Vec<DeptResp>>>, AppError> {
    require_permission(&current_user, "system:dept:list")?;
    let service = DeptService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.tree(&current_user, query).await?,
    )))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<DeptResp>>, AppError> {
    require_permission(&current_user, "system:dept:get")?;
    let service = DeptService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<DeptCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dept:create")?;
    let service = DeptService::new(state.db);
    service.create(&current_user, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<DeptCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dept:update")?;
    let service = DeptService::new(state.db);
    service.update(&current_user, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:dept:delete")?;
    let service = DeptService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn export(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<DeptQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&current_user, "system:dept:export")?;
    let service = DeptService::new(state.db);
    let list = service.list_for_export(&current_user, query).await?;
    let csv = dept_csv(&list);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"dept_export.csv\""),
    );

    Ok((headers, csv))
}

fn dept_csv(list: &[DeptResp]) -> String {
    let mut csv = String::from(
        "ID,名称,上级部门ID,状态,排序,系统内置,描述,创建时间,创建人,修改时间,修改人\n",
    );
    for dept in list {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            dept.id,
            csv_cell(&dept.name),
            dept.parent_id,
            dept.status,
            dept.sort,
            dept.is_system,
            csv_cell(&dept.description),
            csv_cell(&dept.create_time),
            csv_cell(&dept.create_user_string),
            csv_cell(&dept.update_time),
            csv_cell(&dept.update_user_string)
        ));
    }
    csv
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
