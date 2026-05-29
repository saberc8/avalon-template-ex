use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    domain::auth::model::{RoleContext, UserAccount},
    infrastructure::{
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

#[derive(Debug, Clone)]
pub struct CurrentUserDetails {
    pub user: UserAccount,
    pub roles: Vec<RoleContext>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthService {
    users: UserRepository,
    jwt: JwtService,
}

impl AuthService {
    pub fn new(db: PgPool, jwt: JwtService) -> Self {
        Self {
            users: UserRepository::new(db),
            jwt,
        }
    }

    pub async fn login(&self, command: LoginCommand) -> Result<LoginResult, AppError> {
        ensure_account_auth_type(command.auth_type.as_deref())?;

        let username = command.username.trim();
        let password = command.password.trim();
        if username.is_empty() {
            return Err(AppError::bad_request("用户名不能为空"));
        }
        if password.is_empty() {
            return Err(AppError::bad_request("密码不能为空"));
        }

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
}

fn ensure_account_auth_type(auth_type: Option<&str>) -> Result<(), AppError> {
    let auth_type = auth_type.unwrap_or(ACCOUNT_AUTH_TYPE).trim();
    if auth_type.is_empty() || auth_type.eq_ignore_ascii_case(ACCOUNT_AUTH_TYPE) {
        return Ok(());
    }

    Err(AppError::bad_request("暂不支持该认证方式"))
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
}
