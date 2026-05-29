use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    application::auth::service::{AuthService, CurrentUserDetails, LoginCommand, LoginResult},
    domain::auth::model::{CurrentUser, RoleContext},
    shared::{error::AppError, response::ApiResponse},
};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/user/info", get(user_info))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginReq {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    auth_type: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    captcha: Option<String>,
    #[serde(default)]
    captcha_key: Option<String>,
    #[serde(default)]
    uuid: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResp {
    token: String,
    expire: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserInfoResp {
    id: i64,
    username: String,
    nickname: String,
    gender: i16,
    email: String,
    phone: String,
    avatar: String,
    description: String,
    pwd_reset_time: String,
    pwd_expired: bool,
    registration_date: String,
    dept_name: String,
    roles: Vec<String>,
    permissions: Vec<String>,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<ApiResponse<LoginResp>>, AppError> {
    let service = AuthService::new(state.db, state.jwt);
    let result = service.login(req.into()).await?;

    Ok(Json(ApiResponse::ok(LoginResp::from(result))))
}

async fn logout(_current_user: CurrentUser) -> Json<ApiResponse<()>> {
    Json(ApiResponse::ok(()))
}

async fn user_info(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<UserInfoResp>>, AppError> {
    let service = AuthService::new(state.db, state.jwt);
    let details = service.current_user_details(current_user.id).await?;

    Ok(Json(ApiResponse::ok(UserInfoResp::from(details))))
}

impl From<LoginReq> for LoginCommand {
    fn from(req: LoginReq) -> Self {
        Self {
            username: req.username,
            password: req.password,
            auth_type: req.auth_type,
            client_id: req.client_id,
            captcha: req.captcha,
            captcha_key: req.captcha_key,
            uuid: req.uuid,
        }
    }
}

impl From<LoginResult> for LoginResp {
    fn from(result: LoginResult) -> Self {
        Self {
            token: result.token,
            expire: result.expire,
        }
    }
}

impl From<CurrentUserDetails> for UserInfoResp {
    fn from(details: CurrentUserDetails) -> Self {
        let user = details.user;
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            gender: user.gender,
            email: user.email.unwrap_or_default(),
            phone: user.phone.unwrap_or_default(),
            avatar: user.avatar.unwrap_or_default(),
            description: user.description.unwrap_or_default(),
            pwd_reset_time: format_optional_datetime(user.pwd_reset_time),
            pwd_expired: false,
            registration_date: user.create_time.format("%Y-%m-%d").to_string(),
            dept_name: user.dept_name,
            roles: role_codes(details.roles),
            permissions: details.permissions,
        }
    }
}

fn role_codes(roles: Vec<RoleContext>) -> Vec<String> {
    roles.into_iter().map(|role| role.code).collect()
}

fn format_optional_datetime(value: Option<NaiveDateTime>) -> String {
    value
        .map(|time| time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}
