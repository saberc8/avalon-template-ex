use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    application::monitor::{
        log_service::{build_log_record, LogRecordInput, LOGIN_LOG_TYPE},
        online_service::{OnlineLoginCommand, OnlineService},
    },
    domain::auth::model::{RoleContext, UserAccount},
    infrastructure::{
        persistence::log_repository::LogRepository,
        persistence::user_repository::UserRepository,
        security::{jwt::JwtService, password::verify_password},
    },
    shared::error::AppError,
};

const ACCOUNT_AUTH_TYPE: &str = "ACCOUNT";
const ACTIVE_USER_STATUS: i16 = 1;
const INVALID_CREDENTIALS_MESSAGE: &str = "用户名或密码不正确";

#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    pub auth_type: Option<String>,
    pub client_id: Option<String>,
    pub captcha: Option<String>,
    pub captcha_key: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub token: String,
    pub expire: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct LoginMeta {
    pub ip: String,
    pub browser: String,
    pub os: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUserDetails {
    pub user: UserAccount,
    pub roles: Vec<RoleContext>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthService {
    db: PgPool,
    users: UserRepository,
    jwt: JwtService,
}

impl AuthService {
    pub fn new(db: PgPool, jwt: JwtService) -> Self {
        Self {
            users: UserRepository::new(db.clone()),
            db,
            jwt,
        }
    }

    pub async fn login(
        &self,
        command: LoginCommand,
        meta: LoginMeta,
    ) -> Result<LoginResult, AppError> {
        ensure_account_auth_type(command.auth_type.as_deref())?;

        let (username, password) = login_credentials(&command)?;

        let Some(user) = self.users.find_by_username(username).await? else {
            return Err(AppError::bad_request(INVALID_CREDENTIALS_MESSAGE));
        };
        let Some(password_hash) = user.password_hash.as_deref() else {
            return Err(AppError::bad_request(INVALID_CREDENTIALS_MESSAGE));
        };
        if !verify_password(password, password_hash)? {
            return Err(AppError::bad_request(INVALID_CREDENTIALS_MESSAGE));
        }
        if user.status != ACTIVE_USER_STATUS {
            return Err(AppError::bad_request(
                "此账号已被禁用，如有疑问，请联系管理员",
            ));
        }

        let issued = self.jwt.issue_with_expire(user.id, &user.username)?;
        self.record_successful_login(&user, &command, &issued.token, &meta)
            .await?;
        Ok(LoginResult {
            token: issued.token,
            expire: issued.expire,
        })
    }

    pub async fn current_user_details(&self, user_id: i64) -> Result<CurrentUserDetails, AppError> {
        let Some(user) = self.users.find_by_id(user_id).await? else {
            return Err(AppError::Unauthorized);
        };
        if user.status != ACTIVE_USER_STATUS {
            return Err(AppError::Unauthorized);
        }

        let roles = self.users.roles_by_user_id(user_id).await?;
        let permissions = self.users.permissions_for_roles(user_id, &roles).await?;

        Ok(CurrentUserDetails {
            user,
            roles,
            permissions,
        })
    }

    async fn record_successful_login(
        &self,
        user: &UserAccount,
        command: &LoginCommand,
        token: &str,
        meta: &LoginMeta,
    ) -> Result<(), AppError> {
        OnlineService::new(self.db.clone())
            .save_login(
                user,
                OnlineLoginCommand {
                    token: token.to_owned(),
                    client_type: "PC".to_owned(),
                    client_id: command
                        .client_id
                        .clone()
                        .unwrap_or_else(|| "default".to_owned()),
                    ip: meta.ip.clone(),
                    browser: meta.browser.clone(),
                    os: meta.os.clone(),
                },
            )
            .await?;

        let record = build_log_record(LogRecordInput {
            description: "账号登录",
            module: "登录",
            log_type: LOGIN_LOG_TYPE,
            request_url: "/auth/login",
            request_method: "POST",
            request_headers: "{}",
            request_body: "[redacted]",
            status_code: 200,
            response_headers: "{}",
            response_body: "",
            time_taken: 0,
            ip: &meta.ip,
            browser: &meta.browser,
            os: &meta.os,
            status: 1,
            error_msg: "",
            create_user: Some(user.id),
        });
        LogRepository::new(self.db.clone()).insert(&record).await
    }
}

fn ensure_account_auth_type(auth_type: Option<&str>) -> Result<(), AppError> {
    let auth_type = auth_type.unwrap_or(ACCOUNT_AUTH_TYPE).trim();
    if auth_type.is_empty() || auth_type.eq_ignore_ascii_case(ACCOUNT_AUTH_TYPE) {
        return Ok(());
    }

    Err(AppError::bad_request("暂不支持该认证方式"))
}

fn login_credentials(command: &LoginCommand) -> Result<(&str, &str), AppError> {
    let username = command.username.trim();
    let password = command.password.as_str();
    if username.is_empty() {
        return Err(AppError::bad_request("用户名不能为空"));
    }
    if password.is_empty() {
        return Err(AppError::bad_request("密码不能为空"));
    }

    Ok((username, password))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_auth_type_accepts_missing_or_case_insensitive_value() {
        ensure_account_auth_type(None).unwrap();
        ensure_account_auth_type(Some("account")).unwrap();
        ensure_account_auth_type(Some("ACCOUNT")).unwrap();
    }

    #[test]
    fn unsupported_auth_type_returns_bad_request() {
        let err = ensure_account_auth_type(Some("EMAIL")).unwrap_err();

        assert!(matches!(err, AppError::BadRequest(_)));
        assert_eq!(err.to_string(), "暂不支持该认证方式");
    }

    #[test]
    fn login_input_keeps_password_exactly_as_supplied() {
        let command = LoginCommand {
            username: " admin ".to_owned(),
            password: " admin123 ".to_owned(),
            auth_type: None,
            client_id: None,
            captcha: None,
            captcha_key: None,
            uuid: None,
        };

        let (username, password) = login_credentials(&command).unwrap();

        assert_eq!(username, "admin");
        assert_eq!(password, " admin123 ");
    }
}
