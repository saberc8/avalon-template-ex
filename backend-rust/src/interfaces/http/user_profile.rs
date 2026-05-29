use axum::{
    extract::{Multipart, Path, State},
    routing::{get, patch, post},
    Json, Router,
};

use crate::{
    application::user_profile_service::{
        AvatarResp, BasicInfoCommand, ProfileEmailCommand, ProfilePasswordCommand,
        ProfilePhoneCommand, SocialAccountResp, UserProfileService,
    },
    domain::auth::model::CurrentUser,
    shared::{error::AppError, response::ApiResponse},
};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/profile/avatar", patch(upload_avatar))
        .route("/user/profile/basic/info", patch(update_basic_info))
        .route("/user/profile/password", patch(update_password))
        .route("/user/profile/phone", patch(update_phone))
        .route("/user/profile/email", patch(update_email))
        .route("/user/profile/social", get(list_social))
        .route(
            "/user/profile/social/:source",
            post(bind_social).delete(unbind_social),
        )
}

async fn upload_avatar(
    State(state): State<AppState>,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<AvatarResp>>, AppError> {
    let service = UserProfileService::new(state.db);
    let (filename, bytes) = avatar_file(&mut multipart).await?;

    Ok(Json(ApiResponse::ok(
        service
            .update_avatar(current_user.id, filename.as_deref(), &bytes)
            .await?,
    )))
}

async fn update_basic_info(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<BasicInfoCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let service = UserProfileService::new(state.db);
    service.update_basic_info(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_password(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<ProfilePasswordCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let service = UserProfileService::new(state.db);
    service.update_password(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_phone(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<ProfilePhoneCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let service = UserProfileService::new(state.db);
    service.update_phone(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn update_email(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<ProfileEmailCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let service = UserProfileService::new(state.db);
    service.update_email(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn list_social(
    State(state): State<AppState>,
    _current_user: CurrentUser,
) -> Json<ApiResponse<Vec<SocialAccountResp>>> {
    let service = UserProfileService::new(state.db);

    Json(ApiResponse::ok(service.list_social_accounts()))
}

async fn bind_social(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Path(source): Path<String>,
) -> Json<ApiResponse<bool>> {
    let service = UserProfileService::new(state.db);
    service.bind_social_account(&source);

    Json(ApiResponse::ok(true))
}

async fn unbind_social(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Path(source): Path<String>,
) -> Json<ApiResponse<bool>> {
    let service = UserProfileService::new(state.db);
    service.unbind_social_account(&source);

    Json(ApiResponse::ok(true))
}

async fn avatar_file(multipart: &mut Multipart) -> Result<(Option<String>, Vec<u8>), AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("头像文件解析失败"))?
    {
        let name = field.name().map(ToOwned::to_owned);
        let filename = field.file_name().map(ToOwned::to_owned);
        let bytes = field
            .bytes()
            .await
            .map_err(|_| AppError::bad_request("头像文件读取失败"))?;
        if name.as_deref() == Some("avatarFile") {
            return Ok((filename, bytes.to_vec()));
        }
    }

    Err(AppError::bad_request("头像文件不能为空"))
}
