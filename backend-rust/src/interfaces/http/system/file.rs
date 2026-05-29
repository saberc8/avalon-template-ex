use axum::{
    extract::{Multipart, Path, Query, State},
    routing::{get, post},
    Json, Router,
};

use crate::{
    application::system::file_service::{
        CreateDirCommand, FileCheckQuery, FileDirCalcSizeResp, FileQuery, FileResp, FileService,
        FileStatisticsResp, FileUpdateCommand, FileUploadCommand,
    },
    domain::auth::model::CurrentUser,
    interfaces::http::middleware::permission::require_permission,
    shared::{error::AppError, pagination::PageResult, response::ApiResponse},
};

use super::{super::AppState, IdsReq};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/file/upload", post(upload))
        .route("/system/file/statistics", get(statistics))
        .route("/system/file/check", get(check))
        .route("/system/file/dir/:id/size", get(dir_size))
        .route("/system/file/dir", post(create_dir))
        .route("/system/file/:id", axum::routing::put(update))
        .route("/system/file", get(page).delete(delete_many))
        .route("/common/file", post(common_upload))
}

async fn page(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<FileQuery>,
) -> Result<Json<ApiResponse<PageResult<FileResp>>>, AppError> {
    require_permission(&current_user, "system:file:list")?;
    let service = FileService::new(state.db);

    Ok(Json(ApiResponse::ok(service.page(query).await?)))
}

async fn upload(
    State(state): State<AppState>,
    current_user: CurrentUser,
    multipart: Multipart,
) -> Result<Json<ApiResponse<FileResp>>, AppError> {
    require_permission(&current_user, "system:file:upload")?;
    let service = FileService::new(state.db);
    let command = multipart_upload_command(multipart).await?;

    Ok(Json(ApiResponse::ok(
        service.upload(current_user.id, command).await?,
    )))
}

async fn common_upload(
    State(state): State<AppState>,
    current_user: CurrentUser,
    multipart: Multipart,
) -> Result<Json<ApiResponse<FileResp>>, AppError> {
    let service = FileService::new(state.db);
    let command = multipart_upload_command(multipart).await?;

    Ok(Json(ApiResponse::ok(
        service.upload(current_user.id, command).await?,
    )))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(command): Json<FileUpdateCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:file:update")?;
    let service = FileService::new(state.db);
    service.update(current_user.id, id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn delete_many(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<IdsReq>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_permission(&current_user, "system:file:delete")?;
    let service = FileService::new(state.db);
    service.delete(req.ids).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn statistics(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<FileStatisticsResp>>, AppError> {
    require_permission(&current_user, "system:file:list")?;
    let service = FileService::new(state.db);

    Ok(Json(ApiResponse::ok(service.statistics().await?)))
}

async fn check(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<FileCheckQuery>,
) -> Result<Json<ApiResponse<Option<FileResp>>>, AppError> {
    require_permission(&current_user, "system:file:check")?;
    let service = FileService::new(state.db);

    Ok(Json(ApiResponse::ok(service.check(query.sha256).await?)))
}

async fn create_dir(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<CreateDirCommand>,
) -> Result<Json<ApiResponse<FileResp>>, AppError> {
    require_permission(&current_user, "system:file:createDir")?;
    let service = FileService::new(state.db);

    Ok(Json(ApiResponse::ok(
        service.create_dir(current_user.id, command).await?,
    )))
}

async fn dir_size(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FileDirCalcSizeResp>>, AppError> {
    require_permission(&current_user, "system:file:calcDirSize")?;
    let service = FileService::new(state.db);

    Ok(Json(ApiResponse::ok(service.dir_size(id).await?)))
}

async fn multipart_upload_command(mut multipart: Multipart) -> Result<FileUploadCommand, AppError> {
    let mut parent_path = String::new();
    let mut original_name = None;
    let mut content_type = String::new();
    let mut bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("上传文件解析失败"))?
    {
        let field_name = field.name().unwrap_or_default().to_owned();
        if field_name == "file" {
            original_name = field.file_name().map(ToOwned::to_owned).or(original_name);
            content_type = field
                .content_type()
                .map(ToOwned::to_owned)
                .unwrap_or_default();
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("上传文件读取失败"))?
                    .to_vec(),
            );
            continue;
        }

        let text = String::from_utf8(
            field
                .bytes()
                .await
                .map_err(|_| AppError::bad_request("上传参数读取失败"))?
                .to_vec(),
        )
        .map_err(|_| AppError::bad_request("上传参数编码不正确"))?;
        match field_name.as_str() {
            "parentPath" | "parent_path" => parent_path = text,
            "originalName" | "original_name" | "name" => original_name = Some(text),
            _ => {}
        }
    }

    Ok(FileUploadCommand {
        original_name: original_name.ok_or_else(|| AppError::bad_request("文件名称不能为空"))?,
        content_type,
        parent_path,
        bytes: bytes.ok_or_else(|| AppError::bad_request("上传文件不能为空"))?,
    })
}
