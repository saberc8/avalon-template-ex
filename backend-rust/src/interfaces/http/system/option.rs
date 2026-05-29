use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

use crate::{
    application::system::option_service::{
        OptionQuery, OptionResetCommand, OptionResp, OptionService, OptionUpdateItem,
    },
    domain::{auth::model::CurrentUser, rbac::model::PermissionContext},
    shared::{error::AppError, response::ApiResponse},
};

use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/system/option", get(list).put(update))
        .route("/system/option/value", axum::routing::patch(reset_value))
}

async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<OptionQuery>,
) -> Result<Json<ApiResponse<Vec<OptionResp>>>, AppError> {
    require_option_permissions(&current_user, query.category.as_deref(), "get")?;
    let service = OptionService::new(state.db);

    Ok(Json(ApiResponse::ok(service.list(query).await?)))
}

async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(items): Json<Vec<OptionUpdateItem>>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_option_code_permissions(&current_user, &items)?;
    let service = OptionService::new(state.db);
    service.update(current_user.id, items).await?;

    Ok(Json(ApiResponse::ok(true)))
}

async fn reset_value(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(command): Json<OptionResetCommand>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    require_option_permissions(&current_user, command.category.as_deref(), "update")?;
    let service = OptionService::new(state.db);
    service.reset(current_user.id, command).await?;

    Ok(Json(ApiResponse::ok(true)))
}

fn require_option_permissions(
    user: &CurrentUser,
    category: Option<&str>,
    action: &'static str,
) -> Result<(), AppError> {
    let permission = match category.map(|value| value.trim().to_ascii_uppercase()) {
        Some(category) if category == "SITE" => option_permission("SITE", action),
        Some(category) if category == "PASSWORD" => option_permission("PASSWORD", action),
        Some(category) if category == "LOGIN" => option_permission("LOGIN", action),
        Some(_) => None,
        None => None,
    };
    if let Some(permission) = permission {
        return require_all_permissions(user, &[permission]);
    }

    require_all_permissions(
        user,
        &[
            option_permission("SITE", action).expect("known option category"),
            option_permission("PASSWORD", action).expect("known option category"),
            option_permission("LOGIN", action).expect("known option category"),
        ],
    )
}

fn require_option_code_permissions(
    user: &CurrentUser,
    items: &[OptionUpdateItem],
) -> Result<(), AppError> {
    let mut permissions = Vec::new();
    for item in items {
        let code = item.code.trim().to_ascii_uppercase();
        let permission = if code.starts_with("SITE_") {
            option_permission("SITE", "update")
        } else if code.starts_with("PASSWORD_") {
            option_permission("PASSWORD", "update")
        } else if code.starts_with("LOGIN_") {
            option_permission("LOGIN", "update")
        } else {
            None
        };
        if let Some(permission) = permission {
            if !permissions.contains(&permission) {
                permissions.push(permission);
            }
        } else {
            return require_option_permissions(user, None, "update");
        }
    }

    if permissions.is_empty() {
        return require_option_permissions(user, None, "update");
    }
    require_all_permissions(user, &permissions)
}

fn option_permission(category: &str, action: &str) -> Option<&'static str> {
    match (category, action) {
        ("SITE", "get") => Some("system:siteConfig:get"),
        ("SITE", "update") => Some("system:siteConfig:update"),
        ("PASSWORD", "get") => Some("system:securityConfig:get"),
        ("PASSWORD", "update") => Some("system:securityConfig:update"),
        ("LOGIN", "get") => Some("system:loginConfig:get"),
        ("LOGIN", "update") => Some("system:loginConfig:update"),
        _ => None,
    }
}

fn require_all_permissions(
    user: &CurrentUser,
    permissions: &[&'static str],
) -> Result<(), AppError> {
    let context = PermissionContext::from(user);
    if permissions.iter().all(|permission| context.has(permission)) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::domain::auth::model::RoleContext;

    use super::*;

    #[test]
    fn option_update_permissions_are_scoped_by_config_category() {
        let site_user = user_with_permissions(vec!["system:siteConfig:update"]);

        assert!(require_option_code_permissions(&site_user, &[update_item("SITE_TITLE")]).is_ok());
        assert!(matches!(
            require_option_code_permissions(&site_user, &[update_item("PASSWORD_MIN_LENGTH")]),
            Err(AppError::Forbidden)
        ));
    }

    #[test]
    fn mixed_option_updates_require_all_relevant_category_permissions() {
        let site_user = user_with_permissions(vec!["system:siteConfig:update"]);
        let full_user = user_with_permissions(vec![
            "system:siteConfig:update",
            "system:loginConfig:update",
        ]);
        let items = [
            update_item("SITE_TITLE"),
            update_item("LOGIN_CAPTCHA_ENABLED"),
        ];

        assert!(matches!(
            require_option_code_permissions(&site_user, &items),
            Err(AppError::Forbidden)
        ));
        assert!(require_option_code_permissions(&full_user, &items).is_ok());
    }

    fn update_item(code: &str) -> OptionUpdateItem {
        OptionUpdateItem {
            id: 1,
            code: code.to_owned(),
            value: Value::String("value".to_owned()),
        }
    }

    fn user_with_permissions(permissions: Vec<&str>) -> CurrentUser {
        CurrentUser {
            id: 1,
            username: "tester".to_owned(),
            dept_id: 1,
            roles: vec![RoleContext {
                id: 2,
                name: "普通用户".to_owned(),
                code: "general".to_owned(),
                data_scope: 4,
            }],
            permissions: permissions.into_iter().map(ToOwned::to_owned).collect(),
        }
    }
}
