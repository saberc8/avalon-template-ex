use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::Value;

use crate::{
    application::system::user_service::{
        PasswordResetCommand, UserCommand, UserDetailResp, UserImportResp, UserImportResultResp,
        UserQuery, UserResp, UserRoleCommand, UserService,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/user/list", get(list_all))
        .route("/system/user/export", get(export))
        .route("/system/user/import/template", get(import_template))
        .route("/system/user/import/parse", post(parse_import))
        .route("/system/user/import", post(import_users))
        .route("/system/user/:id/password", patch(reset_password))
        .route("/system/user/:id/role", patch(update_role))
        .route("/system/user/:id", get(get_detail).put(update))
        .route("/system/user", get(page).post(create).delete(delete_many))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdResp {
    id: i64,
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<UserQuery>,
) -> Result<Json<ApiResponse<PageResult<UserResp>>>, AppError> {
    require_permission(&current_user, "system:user:list")?;
    let service = UserService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.page(&current_user, query).await?,
    )))
}

async fn list_all(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<UserQuery>,
) -> Result<Json<ApiResponse<Vec<UserResp>>>, AppError> {
    require_permission(&current_user, "system:user:list")?;
    let service = UserService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.list(&current_user, query).await?,
    )))
}

async fn get_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<UserDetailResp>>, AppError> {
    require_permission(&current_user, "system:user:get")?;
    let service = UserService::new(state.db);

    Ok(Json(ApiResponse::ok(service.get(id).await?)))
}

async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<UserCommand>,
) -> Result<Json<ApiResponse<IdResp>>, AppError> {
    require_permission(&current_user, "system:user:create")?;
    let service = UserService::new(state.db);
    let id = service.create(&current_user, command).await?;

    Ok(Json(ApiResponse::ok(IdResp { id })))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<UserCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:user:update")?;
    let service = UserService::new(state.db);
    service.update(&current_user, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:user:delete")?;
    let service = UserService::new(state.db);
    service.delete(&current_user, req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn reset_password(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<PasswordResetCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:user:resetPwd")?;
    let service = UserService::new(state.db);
    service.reset_password(&current_user, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_role(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<UserRoleCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:user:updateRole")?;
    let service = UserService::new(state.db);
    service.update_role(id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn export(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<UserQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&current_user, "system:user:export")?;
    let service = UserService::new(state.db);
    let list = service.list_for_export(&current_user, query).await?;
    let csv = user_csv(&list);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"users.csv\""),
    );

    Ok((headers, csv))
}

async fn import_template(current_user: CurrentUser) -> Result<impl IntoResponse, AppError> {
    require_permission(&current_user, "system:user:import")?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"user_import_template.csv\""),
    );

    Ok((headers, "username,nickname,gender,email,phone\n"))
}

async fn parse_import(
    State(state): State<AppState>,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UserImportResp>>, AppError> {
    require_permission(&current_user, "system:user:import")?;
    let service = UserService::new(state.db);
    let content = multipart_file_bytes(&mut multipart).await?;

    Ok(Json(ApiResponse::ok(service.parse_import(&content))))
}

async fn import_users(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(_body): Json<Value>,
) -> Result<Json<ApiResponse<UserImportResultResp>>, AppError> {
    require_permission(&current_user, "system:user:import")?;
    let service = UserService::new(state.db);

    Ok(Json(ApiResponse::ok(service.import_users())))
}

async fn multipart_file_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    let mut first_file = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("上传文件解析失败"))?
    {
        let is_file = field.name() == Some("file");
        let bytes = field
            .bytes()
            .await
            .map_err(|_| AppError::bad_request("上传文件读取失败"))?;
        if is_file {
            return Ok(bytes.to_vec());
        }
        if first_file.is_none() {
            first_file = Some(bytes.to_vec());
        }
    }

    first_file.ok_or_else(|| AppError::bad_request("文件不能为空"))
}

fn user_csv(list: &[UserResp]) -> String {
    let mut csv =
        String::from("ID,用户名,昵称,性别,邮箱,手机号,状态,部门ID,部门名称,描述,创建时间,创建人\n");
    for user in list {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            user.id,
            csv_cell(&user.username),
            csv_cell(&user.nickname),
            user.gender,
            csv_cell(&user.email),
            csv_cell(&user.phone),
            user.status,
            user.dept_id,
            csv_cell(&user.dept_name),
            csv_cell(&user.description),
            csv_cell(&user.create_time),
            csv_cell(&user.create_user_string)
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
