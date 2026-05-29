use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::{
    domain::auth::model::CurrentUser, infrastructure::persistence::user_repository::UserRepository,
    shared::error::AppError,
};

use super::AppState;

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let claims = state
            .jwt
            .parse(authorization)
            .map_err(|_| AppError::Unauthorized)?;

        let users = UserRepository::new(state.db.clone());
        users
            .current_user_context(claims.user_id)
            .await?
            .ok_or(AppError::Unauthorized)
    }
}
