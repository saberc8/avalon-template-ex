use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

use crate::shared::response::ApiResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("没有访问权限，请联系管理员授权")]
    Forbidden,
    #[error("请求的资源不存在")]
    NotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::Forbidden => "403",
            Self::NotFound => "404",
            Self::Sqlx(_) | Self::Anyhow(_) => "500",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status();
        let body = Json(ApiResponse::fail(self.code(), self.to_string()));
        (status, body).into_response()
    }
}
