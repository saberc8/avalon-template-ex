use sqlx::{FromRow, PgPool};

use crate::{
    domain::rbac::model::{Menu, MenuType},
    shared::error::AppError,
};

#[derive(Debug, Clone)]
pub struct RbacRepository {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct MenuRow {
    id: i64,
    parent_id: i64,
    title: String,
    menu_type: i16,
    path: String,
    name: String,
    component: String,
    redirect: String,
    icon: String,
    is_external: bool,
    is_cache: bool,
    is_hidden: bool,
    permission: String,
    sort: i32,
    status: i16,
}

impl RbacRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn all_enabled_route_menus(&self) -> Result<Vec<Menu>, AppError> {
        let menus = sqlx::query_as::<_, MenuRow>(
            r#"
SELECT
    m.id,
    m.parent_id,
    m.title,
    m.type AS menu_type,
    COALESCE(m.path, '') AS path,
    COALESCE(m.name, '') AS name,
    COALESCE(m.component, '') AS component,
    COALESCE(m.redirect, '') AS redirect,
    COALESCE(m.icon, '') AS icon,
    COALESCE(m.is_external, FALSE) AS is_external,
    COALESCE(m.is_cache, FALSE) AS is_cache,
    COALESCE(m.is_hidden, FALSE) AS is_hidden,
    COALESCE(m.permission, '') AS permission,
    m.sort,
    m.status
FROM sys_menu AS m
WHERE m.status = 1
  AND m.type <> 3
ORDER BY m.sort ASC, m.id ASC;
"#,
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(Menu::from)
        .collect();

        Ok(menus)
    }

    pub async fn enabled_route_menus_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Vec<Menu>, AppError> {
        let menus = sqlx::query_as::<_, MenuRow>(
            r#"
SELECT DISTINCT
    m.id,
    m.parent_id,
    m.title,
    m.type AS menu_type,
    COALESCE(m.path, '') AS path,
    COALESCE(m.name, '') AS name,
    COALESCE(m.component, '') AS component,
    COALESCE(m.redirect, '') AS redirect,
    COALESCE(m.icon, '') AS icon,
    COALESCE(m.is_external, FALSE) AS is_external,
    COALESCE(m.is_cache, FALSE) AS is_cache,
    COALESCE(m.is_hidden, FALSE) AS is_hidden,
    COALESCE(m.permission, '') AS permission,
    m.sort,
    m.status
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
JOIN sys_user_role AS ur ON ur.role_id = rm.role_id
WHERE ur.user_id = $1
  AND m.status = 1
  AND m.type <> 3
ORDER BY m.sort ASC, m.id ASC;
"#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(Menu::from)
        .collect();

        Ok(menus)
    }
}

impl From<MenuRow> for Menu {
    fn from(row: MenuRow) -> Self {
        Self {
            id: row.id,
            parent_id: row.parent_id,
            title: row.title,
            menu_type: MenuType::from(row.menu_type),
            path: row.path,
            name: row.name,
            component: row.component,
            redirect: row.redirect,
            icon: row.icon,
            is_external: row.is_external,
            is_cache: row.is_cache,
            is_hidden: row.is_hidden,
            permission: row.permission,
            sort: row.sort,
            status: row.status,
        }
    }
}
