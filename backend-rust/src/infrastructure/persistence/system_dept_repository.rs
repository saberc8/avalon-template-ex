use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::{domain::data_scope::model::DataScopeFilter, shared::error::AppError};

#[derive(Debug, Clone)]
pub struct SystemDeptRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct DeptRecord {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub sort: i32,
    pub status: i16,
    pub is_system: bool,
    pub description: String,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DeptBasicRecord {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub ancestors: String,
    pub status: i16,
    pub is_system: bool,
}

#[derive(Debug, Clone)]
pub struct DeptListFilter<'a> {
    pub description: Option<&'a str>,
    pub status: Option<i16>,
    pub data_scope: &'a DataScopeFilter,
}

#[derive(Debug, Clone)]
pub struct DeptCreateRecord {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub ancestors: String,
    pub sort: i32,
    pub status: i16,
    pub description: Option<String>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct DeptUpdateRecord {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub ancestors: String,
    pub sort: i32,
    pub status: i16,
    pub description: Option<String>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

impl SystemDeptRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self, filter: &DeptListFilter<'_>) -> Result<Vec<DeptRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dept_select_sql());
        query.push(" WHERE 1 = 1");
        push_dept_filters(&mut query, filter.description, filter.status);
        filter.data_scope.append_and_clause(&mut query);
        query.push(" ORDER BY d.sort ASC, d.id ASC");

        let records = query
            .build_query_as::<DeptRecord>()
            .fetch_all(&self.db)
            .await?;
        Ok(records)
    }

    pub async fn list_all_for_common_tree(&self) -> Result<Vec<DeptRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dept_select_sql());
        query.push(" ORDER BY d.sort ASC, d.id ASC");

        Ok(query
            .build_query_as::<DeptRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get(&self, id: i64) -> Result<Option<DeptRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dept_select_sql());
        query.push(" WHERE d.id = ").push_bind(id).push(" LIMIT 1");

        Ok(query
            .build_query_as::<DeptRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn basic(&self, id: i64) -> Result<Option<DeptBasicRecord>, AppError> {
        let record = sqlx::query_as::<_, DeptBasicRecord>(
            r#"
SELECT id, name, parent_id, ancestors, status, is_system
FROM sys_dept
WHERE id = $1
LIMIT 1;
"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(record)
    }

    pub async fn name_exists(
        &self,
        name: &str,
        parent_id: i64,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT EXISTS(SELECT 1 FROM sys_dept WHERE name = ");
        query
            .push_bind(name)
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

    pub async fn create(&self, record: &DeptCreateRecord) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_dept (
    id, name, parent_id, ancestors, description, sort, status, is_system,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, $8, $9);
"#,
        )
        .bind(record.id)
        .bind(&record.name)
        .bind(record.parent_id)
        .bind(&record.ancestors)
        .bind(&record.description)
        .bind(record.sort)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update(&self, record: &DeptUpdateRecord) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_dept
SET name = $1,
    parent_id = $2,
    ancestors = $3,
    sort = $4,
    status = $5,
    description = $6,
    update_user = $7,
    update_time = $8
WHERE id = $9;
"#,
        )
        .bind(&record.name)
        .bind(record.parent_id)
        .bind(&record.ancestors)
        .bind(record.sort)
        .bind(record.status)
        .bind(&record.description)
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

    pub async fn first_system_name(&self, ids: &[i64]) -> Result<Option<String>, AppError> {
        if ids.is_empty() {
            return Ok(None);
        }

        let name = sqlx::query_scalar::<_, String>(
            r#"
SELECT name
FROM sys_dept
WHERE id = ANY($1) AND is_system = TRUE
ORDER BY id ASC
LIMIT 1;
"#,
        )
        .bind(ids.to_vec())
        .fetch_optional(&self.db)
        .await?;

        Ok(name)
    }

    pub async fn has_children(&self, ids: &[i64]) -> Result<bool, AppError> {
        if ids.is_empty() {
            return Ok(false);
        }

        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM sys_dept WHERE parent_id = ANY($1));",
        )
        .bind(ids.to_vec())
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn has_users(&self, ids: &[i64]) -> Result<bool, AppError> {
        if ids.is_empty() {
            return Ok(false);
        }

        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM sys_user WHERE dept_id = ANY($1));",
        )
        .bind(ids.to_vec())
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn delete_many(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM sys_role_dept WHERE dept_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_dept WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(())
    }
}

fn dept_select_sql() -> &'static str {
    r#"
SELECT
    d.id,
    d.name,
    d.parent_id,
    d.sort,
    d.status,
    d.is_system,
    COALESCE(d.description, '') AS description,
    d.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    d.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
"#
}

fn push_dept_filters(
    query: &mut QueryBuilder<'_, Postgres>,
    description: Option<&str>,
    status: Option<i16>,
) {
    if let Some(description) = non_empty(description) {
        let pattern = format!("%{description}%");
        query
            .push(" AND (d.name ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR COALESCE(d.description, '') ILIKE ")
            .push_bind(pattern)
            .push(")");
    }

    if let Some(status) = status.filter(|value| *value > 0) {
        query.push(" AND d.status = ").push_bind(status);
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
