use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::shared::error::AppError;

#[derive(Debug, Clone)]
pub struct SystemMenuRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct MenuRecord {
    pub id: i64,
    pub title: String,
    pub parent_id: i64,
    pub menu_type: i16,
    pub path: String,
    pub name: String,
    pub component: String,
    pub redirect: String,
    pub icon: String,
    pub is_external: bool,
    pub is_cache: bool,
    pub is_hidden: bool,
    pub permission: String,
    pub sort: i32,
    pub status: i16,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
struct MenuEdgeRecord {
    id: i64,
    parent_id: i64,
}

#[derive(Debug, Clone)]
pub struct MenuListFilter<'a> {
    pub title: Option<&'a str>,
    pub status: Option<i16>,
    pub only_catalog_and_menu: bool,
}

#[derive(Debug, Clone)]
pub struct MenuSaveRecord {
    pub id: i64,
    pub title: String,
    pub parent_id: i64,
    pub menu_type: i16,
    pub path: Option<String>,
    pub name: Option<String>,
    pub component: Option<String>,
    pub redirect: Option<String>,
    pub icon: Option<String>,
    pub is_external: bool,
    pub is_cache: bool,
    pub is_hidden: bool,
    pub permission: Option<String>,
    pub sort: i32,
    pub status: i16,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

impl SystemMenuRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self, filter: &MenuListFilter<'_>) -> Result<Vec<MenuRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(menu_select_sql());
        query.push(" WHERE 1 = 1");
        if filter.only_catalog_and_menu {
            query.push(" AND m.type IN (1, 2)");
        }
        push_menu_filters(&mut query, filter.title, filter.status);
        query.push(" ORDER BY m.sort ASC, m.id ASC");

        Ok(query
            .build_query_as::<MenuRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get(&self, id: i64) -> Result<Option<MenuRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(menu_select_sql());
        query.push(" WHERE m.id = ").push_bind(id).push(" LIMIT 1");

        Ok(query
            .build_query_as::<MenuRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn title_exists(
        &self,
        title: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT EXISTS(SELECT 1 FROM sys_menu WHERE title = ");
        query
            .push_bind(title)
            .push(" AND parent_id = ")
            .push_bind(parent_id);
        if let Some(exclude_id) = exclude_id {
            query.push(" AND id <> ").push_bind(exclude_id);
        }
        query.push(")");

        Ok(query
            .build_query_scalar::<bool>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn create(&self, record: &MenuSaveRecord) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect, icon,
    is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
)
VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9,
    $10, $11, $12, $13, $14, $15,
    $16, $17
);
"#,
        )
        .bind(record.id)
        .bind(&record.title)
        .bind(record.parent_id)
        .bind(record.menu_type)
        .bind(&record.path)
        .bind(&record.name)
        .bind(&record.component)
        .bind(&record.redirect)
        .bind(&record.icon)
        .bind(record.is_external)
        .bind(record.is_cache)
        .bind(record.is_hidden)
        .bind(&record.permission)
        .bind(record.sort)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update(&self, record: &MenuSaveRecord) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_menu
SET title = $1,
    parent_id = $2,
    type = $3,
    path = $4,
    name = $5,
    component = $6,
    redirect = $7,
    icon = $8,
    is_external = $9,
    is_cache = $10,
    is_hidden = $11,
    permission = $12,
    sort = $13,
    status = $14,
    update_user = $15,
    update_time = $16
WHERE id = $17;
"#,
        )
        .bind(&record.title)
        .bind(record.parent_id)
        .bind(record.menu_type)
        .bind(&record.path)
        .bind(&record.name)
        .bind(&record.component)
        .bind(&record.redirect)
        .bind(&record.icon)
        .bind(record.is_external)
        .bind(record.is_cache)
        .bind(record.is_hidden)
        .bind(&record.permission)
        .bind(record.sort)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    pub async fn all_edges(&self) -> Result<Vec<(i64, i64)>, AppError> {
        let rows = sqlx::query_as::<_, MenuEdgeRecord>(
            "SELECT id, parent_id FROM sys_menu ORDER BY parent_id ASC, id ASC;",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.id, row.parent_id))
            .collect())
    }

    pub async fn delete_many(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM sys_role_menu WHERE menu_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_menu WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(())
    }
}

fn menu_select_sql() -> &'static str {
    r#"
SELECT
    m.id,
    m.title,
    m.parent_id,
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
    m.status,
    m.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    m.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
"#
}

fn push_menu_filters(
    query: &mut QueryBuilder<'_, Postgres>,
    title: Option<&str>,
    status: Option<i16>,
) {
    if let Some(title) = title.map(str::trim).filter(|value| !value.is_empty()) {
        query
            .push(" AND m.title ILIKE ")
            .push_bind(format!("%{title}%"));
    }
    if let Some(status) = status.filter(|value| *value > 0) {
        query.push(" AND m.status = ").push_bind(status);
    }
}
