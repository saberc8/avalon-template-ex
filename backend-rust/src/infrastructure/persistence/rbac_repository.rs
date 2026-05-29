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
WITH RECURSIVE assigned_menus AS (
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
    JOIN sys_role AS r ON r.id = ur.role_id
    WHERE ur.user_id = $1
      AND r.status = 1
      AND m.status = 1
      AND m.type <> 3
),
menu_tree AS (
    SELECT * FROM assigned_menus
    UNION
    SELECT
        parent.id,
        parent.parent_id,
        parent.title,
        parent.type AS menu_type,
        COALESCE(parent.path, '') AS path,
        COALESCE(parent.name, '') AS name,
        COALESCE(parent.component, '') AS component,
        COALESCE(parent.redirect, '') AS redirect,
        COALESCE(parent.icon, '') AS icon,
        COALESCE(parent.is_external, FALSE) AS is_external,
        COALESCE(parent.is_cache, FALSE) AS is_cache,
        COALESCE(parent.is_hidden, FALSE) AS is_hidden,
        COALESCE(parent.permission, '') AS permission,
        parent.sort,
        parent.status
    FROM sys_menu AS parent
    JOIN menu_tree AS child ON child.parent_id = parent.id
    WHERE parent.status = 1
      AND parent.type <> 3
)
SELECT DISTINCT
    m.id,
    m.parent_id,
    m.title,
    m.menu_type,
    m.path,
    m.name,
    m.component,
    m.redirect,
    m.icon,
    m.is_external,
    m.is_cache,
    m.is_hidden,
    m.permission,
    m.sort,
    m.status
FROM menu_tree AS m
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
