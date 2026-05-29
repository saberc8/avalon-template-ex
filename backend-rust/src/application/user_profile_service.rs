use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::system::ensure_max_chars,
    infrastructure::{
        persistence::system_user_repository::SystemUserRepository,
        security::password::{hash_password, verify_password},
    },
    shared::error::AppError,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicInfoCommand {
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub gender: i16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePasswordCommand {
    #[serde(default)]
    pub old_password: String,
    #[serde(default)]
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePhoneCommand {
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub captcha: String,
    #[serde(default)]
    pub old_password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileEmailCommand {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub captcha: String,
    #[serde(default)]
    pub old_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarResp {
    pub avatar: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialAccountResp {
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct UserProfileService {
    users: SystemUserRepository,
}

impl UserProfileService {
    pub fn new(db: PgPool) -> Self {
        Self {
            users: SystemUserRepository::new(db),
        }
    }

    pub async fn update_avatar(
        &self,
        user_id: i64,
        original_filename: Option<&str>,
        bytes: &[u8],
    ) -> Result<AvatarResp, AppError> {
        if bytes.is_empty() {
            return Err(AppError::bad_request("头像文件不能为空"));
        }

        let extension = avatar_extension(original_filename);
        let avatar = format!("/file/avatar/{}.{}", uuid::Uuid::new_v4(), extension);
        self.users
            .update_avatar(user_id, &avatar, Utc::now().naive_utc())
            .await?;

        Ok(AvatarResp { avatar })
    }

    pub async fn update_basic_info(
        &self,
        user_id: i64,
        mut command: BasicInfoCommand,
    ) -> Result<(), AppError> {
        command.nickname = command.nickname.trim().to_owned();
        if command.nickname.is_empty() {
            return Err(AppError::bad_request("昵称不能为空"));
        }
        if !(0..=2).contains(&command.gender) {
            return Err(AppError::bad_request("性别不正确"));
        }
        ensure_max_chars("昵称", &command.nickname, 30)?;

        self.users
            .update_basic_info(
                user_id,
                &command.nickname,
                command.gender,
                Utc::now().naive_utc(),
            )
            .await
    }

    pub async fn update_password(
        &self,
        user_id: i64,
        command: ProfilePasswordCommand,
    ) -> Result<(), AppError> {
        let old_password = command.old_password;
        let new_password = command.new_password.trim().to_owned();
        if new_password.is_empty() {
            return Err(AppError::bad_request("新密码不能为空"));
        }
        ensure_max_chars("新密码", &new_password, 128)?;
        self.verify_old_password(user_id, &old_password).await?;
        let hash = hash_password(&new_password)?;

        self.users
            .update_password(user_id, &hash, user_id, Utc::now().naive_utc())
            .await
    }

    pub async fn update_phone(
        &self,
        user_id: i64,
        mut command: ProfilePhoneCommand,
    ) -> Result<(), AppError> {
        command.phone = command.phone.trim().to_owned();
        if command.phone.is_empty() {
            return Err(AppError::bad_request("手机号不能为空"));
        }
        ensure_max_chars("手机号", &command.phone, 255)?;
        self.verify_old_password(user_id, &command.old_password)
            .await?;
        if self
            .users
            .phone_exists(&command.phone, Some(user_id))
            .await?
        {
            return Err(AppError::bad_request(format!(
                "保存失败，[{}] 已存在",
                command.phone
            )));
        }

        self.users
            .update_phone(user_id, &command.phone, Utc::now().naive_utc())
            .await
    }

    pub async fn update_email(
        &self,
        user_id: i64,
        mut command: ProfileEmailCommand,
    ) -> Result<(), AppError> {
        command.email = command.email.trim().to_owned();
        if command.email.is_empty() {
            return Err(AppError::bad_request("邮箱不能为空"));
        }
        ensure_max_chars("邮箱", &command.email, 255)?;
        self.verify_old_password(user_id, &command.old_password)
            .await?;
        if self
            .users
            .email_exists(&command.email, Some(user_id))
            .await?
        {
            return Err(AppError::bad_request(format!(
                "保存失败，[{}] 已存在",
                command.email
            )));
        }

        self.users
            .update_email(user_id, &command.email, Utc::now().naive_utc())
            .await
    }

    pub fn list_social_accounts(&self) -> Vec<SocialAccountResp> {
        vec![]
    }

    pub fn bind_social_account(&self, _source: &str) {}

    pub fn unbind_social_account(&self, _source: &str) {}

    async fn verify_old_password(&self, user_id: i64, old_password: &str) -> Result<(), AppError> {
        let Some(password_hash) = self.users.password_hash(user_id).await? else {
            return Err(AppError::bad_request("旧密码不正确"));
        };
        if !verify_password(old_password, &password_hash)? {
            return Err(AppError::bad_request("旧密码不正确"));
        }
        Ok(())
    }
}

fn avatar_extension(original_filename: Option<&str>) -> &'static str {
    let Some(filename) = original_filename else {
        return "bin";
    };
    match filename
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_extension_is_whitelisted() {
        assert_eq!(avatar_extension(Some("me.png")), "png");
        assert_eq!(avatar_extension(Some("me.svg")), "bin");
    }
}
