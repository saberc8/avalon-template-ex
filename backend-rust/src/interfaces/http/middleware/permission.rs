use crate::{
    domain::{auth::model::CurrentUser, rbac::model::PermissionContext},
    shared::error::AppError,
};

pub fn require_permission(user: &CurrentUser, permission: &'static str) -> Result<(), AppError> {
    let context = PermissionContext::from(user);
    if context.has(permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::auth::model::{CurrentUser, RoleContext};

    use super::*;

    fn user_with_permissions(permissions: Vec<String>) -> CurrentUser {
        CurrentUser {
            id: 1,
            username: "tester".to_string(),
            dept_id: 1,
            roles: vec![RoleContext {
                id: 2,
                name: "普通用户".to_string(),
                code: "general".to_string(),
                data_scope: 4,
            }],
            permissions,
        }
    }

    #[test]
    fn require_permission_allows_explicit_permission() {
        let user = user_with_permissions(vec!["system:user:list".to_string()]);

        assert!(require_permission(&user, "system:user:list").is_ok());
    }

    #[test]
    fn require_permission_rejects_missing_permission() {
        let user = user_with_permissions(vec!["system:user:list".to_string()]);

        assert!(require_permission(&user, "system:user:delete").is_err());
    }
}
