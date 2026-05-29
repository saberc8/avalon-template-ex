use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool};

use crate::{
    domain::auth::model::{CurrentUser, RoleContext, UserAccount},
    domain::rbac::model::ALL_PERMISSION,
    shared::error::AppError,
};

#[derive(Debug, Clone)]
pub struct UserRepository {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UserAccountRow {
    id: i64,
    username: String,
    nickname: String,
    password_hash: Option<String>,
    gender: i16,
    email: Option<String>,
    phone: Option<String>,
    avatar: Option<String>,
    description: Option<String>,
    status: i16,
    pwd_reset_time: Option<NaiveDateTime>,
    dept_id: i64,
    dept_name: String,
    create_time: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct RoleContextRow {
    id: i64,
    name: String,
    code: String,
    data_scope: i16,
}

#[derive(Debug, FromRow)]
struct PermissionRow {
    permission: String,
}

impl UserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserAccount>, AppError> {
        let user = sqlx::query_as::<_, UserAccountRow>(
            r#"
SELECT
    u.id,
    u.username,
    u.nickname,
    u.password AS password_hash,
    u.gender,
    u.email,
    u.phone,
    u.avatar,
    u.description,
    u.status,
    u.pwd_reset_time,
    u.dept_id,
    COALESCE(d.name, '') AS dept_name,
    u.create_time
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE u.username = $1
LIMIT 1;
"#,
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?
        .map(UserAccount::from);

        Ok(user)
    }

    pub async fn find_by_id(&self, user_id: i64) -> Result<Option<UserAccount>, AppError> {
        let user = sqlx::query_as::<_, UserAccountRow>(
            r#"
SELECT
    u.id,
    u.username,
    u.nickname,
    u.password AS password_hash,
    u.gender,
    u.email,
    u.phone,
    u.avatar,
    u.description,
    u.status,
    u.pwd_reset_time,
    u.dept_id,
    COALESCE(d.name, '') AS dept_name,
    u.create_time
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE u.id = $1
LIMIT 1;
"#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .map(UserAccount::from);

        Ok(user)
    }

    pub async fn roles_by_user_id(&self, user_id: i64) -> Result<Vec<RoleContext>, AppError> {
        let roles = sqlx::query_as::<_, RoleContextRow>(
            r#"
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = $1
  AND r.status = 1
ORDER BY r.sort ASC, r.id ASC;
"#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(RoleContext::from)
        .collect();

        Ok(roles)
    }

    pub async fn permissions_by_user_id(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        let permissions = sqlx::query_as::<_, PermissionRow>(
            r#"
SELECT DISTINCT m.permission
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
JOIN sys_user_role AS ur ON ur.role_id = rm.role_id
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = $1
  AND r.status = 1
  AND m.status = 1
  AND m.permission IS NOT NULL
  AND m.permission <> ''
ORDER BY m.permission ASC;
"#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| row.permission)
        .collect();

        Ok(permissions)
    }

    pub async fn current_user_context(
        &self,
        user_id: i64,
    ) -> Result<Option<CurrentUser>, AppError> {
        let Some(user) = self.find_by_id(user_id).await? else {
            return Ok(None);
        };
        if user.status != 1 {
            return Ok(None);
        }

        let roles = self.roles_by_user_id(user_id).await?;
        let permissions = self.permissions_for_roles(user_id, &roles).await?;

        Ok(Some(CurrentUser {
            id: user.id,
            username: user.username,
            dept_id: user.dept_id,
            roles,
            permissions,
        }))
    }

    pub async fn permissions_for_roles(
        &self,
        user_id: i64,
        roles: &[RoleContext],
    ) -> Result<Vec<String>, AppError> {
        if roles.iter().any(RoleContext::is_admin) {
            return Ok(vec![ALL_PERMISSION.to_owned()]);
        }

        self.permissions_by_user_id(user_id).await
    }
}

impl From<UserAccountRow> for UserAccount {
    fn from(row: UserAccountRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            nickname: row.nickname,
            password_hash: row.password_hash,
            gender: row.gender,
            email: row.email,
            phone: row.phone,
            avatar: row.avatar,
            description: row.description,
            status: row.status,
            pwd_reset_time: row.pwd_reset_time,
            dept_id: row.dept_id,
            dept_name: row.dept_name,
            create_time: row.create_time,
        }
    }
}

impl From<RoleContextRow> for RoleContext {
    fn from(row: RoleContextRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            code: row.code,
            data_scope: row.data_scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

    use super::*;

    fn test_repository() -> UserRepository {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost:5432/avalon_admin")
            .unwrap();
        UserRepository::new(db)
    }

    #[tokio::test]
    async fn admin_permission_uses_java_wildcard() {
        let permissions = test_repository()
            .permissions_for_roles(
                1,
                &[RoleContext {
                    id: 1,
                    name: "系统管理员".to_string(),
                    code: "admin".to_string(),
                    data_scope: 1,
                }],
            )
            .await
            .unwrap();

        assert_eq!(permissions, vec!["*:*:*"]);
    }
}
