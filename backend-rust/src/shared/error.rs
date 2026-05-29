use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

use crate::shared::response::ApiResponse;

const INTERNAL_ERROR_MESSAGE: &str = "系统异常，请稍后重试";

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("未授权，请重新登录")]
    Unauthorized,
    #[error("没有访问权限，请联系管理员授权")]
    Forbidden,
    #[error("请求的资源不存在")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "400",
            Self::Unauthorized => "401",
            Self::Forbidden => "403",
            Self::NotFound | Self::Sqlx(sqlx::Error::RowNotFound) => "404",
            Self::Conflict(_) => "409",
            Self::Sqlx(_) | Self::Io(_) | Self::Anyhow(_) => "500",
        }
    }

    fn client_message(&self) -> &str {
        match self {
            Self::BadRequest(message) => message,
            Self::Unauthorized => "未授权，请重新登录",
            Self::Forbidden => "没有访问权限，请联系管理员授权",
            Self::NotFound | Self::Sqlx(sqlx::Error::RowNotFound) => "请求的资源不存在",
            Self::Conflict(message) => message,
            Self::Sqlx(_) | Self::Io(_) | Self::Anyhow(_) => INTERNAL_ERROR_MESSAGE,
        }
    }

    fn should_log_internal(&self) -> bool {
        matches!(self, Self::Sqlx(_) | Self::Io(_) | Self::Anyhow(_))
            && !matches!(self, Self::Sqlx(sqlx::Error::RowNotFound))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code();
        let message = self.client_message();

        if self.should_log_internal() {
            tracing::error!(error = ?self, "internal server error");
        }

        let body = Json(ApiResponse::fail(code, message));
        (StatusCode::OK, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use http_body_util::BodyExt;

    use super::*;

    async fn response_body(error: AppError) -> (StatusCode, ApiResponse<()>) {
        let response = error.into_response();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<ApiResponse<()>>(&body).unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn sql_internal_error_message_is_not_returned_to_client() {
        let (status, body) = response_body(AppError::Sqlx(sqlx::Error::Protocol(
            "secret sql detail".into(),
        )))
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.code, "500");
        assert!(!body.success);
        assert_ne!(body.msg, "secret sql detail");
        assert!(!body.msg.contains("secret sql detail"));
    }

    #[tokio::test]
    async fn anyhow_internal_error_message_is_not_returned_to_client() {
        let (status, body) =
            response_body(AppError::Anyhow(anyhow::anyhow!("secret anyhow detail"))).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.code, "500");
        assert!(!body.success);
        assert_ne!(body.msg, "secret anyhow detail");
        assert!(!body.msg.contains("secret anyhow detail"));
    }

    #[tokio::test]
    async fn row_not_found_maps_to_not_found_response() {
        let (status, body) = response_body(AppError::Sqlx(sqlx::Error::RowNotFound)).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.code, "404");
        assert!(!body.success);
        assert_eq!(body.msg, "请求的资源不存在");
    }
}
