use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::shared::error::AppError;

#[derive(Debug, Clone)]
pub struct SystemRoleRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct RoleRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub sort: i32,
    pub description: String,
    pub data_scope: i16,
    pub is_system: bool,
    pub menu_check_strictly: bool,
    pub dept_check_strictly: bool,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct RoleUserRecord {
    pub id: i64,
    pub role_id: i64,
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub gender: i16,
    pub status: i16,
    pub is_system: bool,
    pub description: String,
    pub dept_id: i64,
    pub dept_name: String,
    pub role_ids: Vec<i64>,
    pub role_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RoleListFilter<'a> {
    pub description: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct RoleUserListFilter<'a> {
    pub role_id: i64,
    pub description: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct RoleCreateRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub data_scope: i16,
    pub description: Option<String>,
    pub sort: i32,
    pub dept_check_strictly: bool,
    pub dept_ids: Vec<i64>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct RoleUpdateRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub data_scope: i16,
    pub description: Option<String>,
    pub sort: i32,
    pub dept_check_strictly: bool,
    pub dept_ids: Vec<i64>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct RolePermissionRecord {
    pub id: i64,
    pub menu_ids: Vec<i64>,
    pub menu_check_strictly: bool,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

impl SystemRoleRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list(&self, filter: &RoleListFilter<'_>) -> Result<Vec<RoleRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(role_select_sql());
        query.push(" WHERE 1 = 1");
        if let Some(description) = filter
            .description
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let pattern = format!("%{description}%");
            query
                .push(" AND (r.name ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR COALESCE(r.description, '') ILIKE ")
                .push_bind(pattern)
                .push(")");
        }
        query.push(" ORDER BY r.sort ASC, r.id ASC");

        Ok(query
            .build_query_as::<RoleRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get(&self, id: i64) -> Result<Option<RoleRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(role_select_sql());
        query.push(" WHERE r.id = ").push_bind(id).push(" LIMIT 1");

        Ok(query
            .build_query_as::<RoleRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> Result<bool, AppError> {
        self.exists_by_unique_column("name", name, exclude_id).await
    }

    pub async fn code_exists(&self, code: &str, exclude_id: Option<i64>) -> Result<bool, AppError> {
        self.exists_by_unique_column("code", code, exclude_id).await
    }

    pub async fn create(&self, record: &RoleCreateRecord) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        sqlx::query(
            r#"
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time, status
)
VALUES ($1, $2, $3, $4, $5, $6, FALSE, TRUE, $7, $8, $9, 1);
"#,
        )
        .bind(record.id)
        .bind(&record.name)
        .bind(&record.code)
        .bind(record.data_scope)
        .bind(&record.description)
        .bind(record.sort)
        .bind(record.dept_check_strictly)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&mut *tx)
        .await?;

        insert_role_depts(&mut tx, record.id, &record.dept_ids).await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn update(&self, record: &RoleUpdateRecord) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        let result = sqlx::query(
            r#"
UPDATE sys_role
SET name = $1,
    code = $2,
    data_scope = $3,
    description = $4,
    sort = $5,
    dept_check_strictly = $6,
    update_user = $7,
    update_time = $8
WHERE id = $9;
"#,
        )
        .bind(&record.name)
        .bind(&record.code)
        .bind(record.data_scope)
        .bind(&record.description)
        .bind(record.sort)
        .bind(record.dept_check_strictly)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        if record.code != "admin" {
            sqlx::query("DELETE FROM sys_role_dept WHERE role_id = $1;")
                .bind(record.id)
                .execute(&mut *tx)
                .await?;
            insert_role_depts(&mut tx, record.id, &record.dept_ids).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn menu_ids(&self, role_id: i64) -> Result<Vec<i64>, AppError> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT menu_id FROM sys_role_menu WHERE role_id = $1 ORDER BY menu_id ASC;",
        )
        .bind(role_id)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn dept_ids(&self, role_id: i64) -> Result<Vec<i64>, AppError> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT dept_id FROM sys_role_dept WHERE role_id = $1 ORDER BY dept_id ASC;",
        )
        .bind(role_id)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn update_permission(&self, record: &RolePermissionRecord) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_role WHERE id = $1);")
                .bind(record.id)
                .fetch_one(&mut *tx)
                .await?;
        if !exists {
            return Err(AppError::NotFound);
        }

        sqlx::query("DELETE FROM sys_role_menu WHERE role_id = $1;")
            .bind(record.id)
            .execute(&mut *tx)
            .await?;
        insert_role_menus(&mut tx, record.id, &record.menu_ids).await?;
        sqlx::query(
            r#"
UPDATE sys_role
SET menu_check_strictly = $1,
    update_user = $2,
    update_time = $3
WHERE id = $4;
"#,
        )
        .bind(record.menu_check_strictly)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn first_system_name(&self, ids: &[i64]) -> Result<Option<String>, AppError> {
        if ids.is_empty() {
            return Ok(None);
        }

        Ok(sqlx::query_scalar::<_, String>(
            r#"
SELECT name
FROM sys_role
WHERE id = ANY($1) AND is_system = TRUE
ORDER BY id ASC
LIMIT 1;
"#,
        )
        .bind(ids.to_vec())
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn has_users(&self, ids: &[i64]) -> Result<bool, AppError> {
        if ids.is_empty() {
            return Ok(false);
        }

        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM sys_user_role WHERE role_id = ANY($1));",
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
        sqlx::query("DELETE FROM sys_role_menu WHERE role_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_role_dept WHERE role_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_role WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn user_ids(&self, role_id: i64) -> Result<Vec<i64>, AppError> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT user_id FROM sys_user_role WHERE role_id = $1 ORDER BY user_id ASC;",
        )
        .bind(role_id)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn count_role_users(&self, filter: &RoleUserListFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
SELECT COUNT(*)
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
WHERE ur.role_id = "#,
        );
        query.push_bind(filter.role_id);
        push_role_user_description_filter(&mut query, filter.description);

        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_role_users(
        &self,
        filter: &RoleUserListFilter<'_>,
    ) -> Result<Vec<RoleUserRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(role_user_select_sql());
        query.push(" WHERE ur.role_id = ").push_bind(filter.role_id);
        push_role_user_description_filter(&mut query, filter.description);
        query.push(
            r#"
GROUP BY
    ur.id,
    ur.role_id,
    u.id,
    u.username,
    u.nickname,
    u.gender,
    u.status,
    u.is_system,
    u.description,
    u.dept_id,
    d.name
ORDER BY ur.id DESC
LIMIT "#,
        );
        query
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);

        Ok(query
            .build_query_as::<RoleUserRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn assign_users(&self, role_id: i64, user_ids: &[i64]) -> Result<(), AppError> {
        if user_ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.db.begin().await?;
        for user_id in normalized_ids(user_ids) {
            sqlx::query(
                r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
            )
            .bind(crate::shared::id::next_id())
            .bind(user_id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn has_protected_admin_user_role(
        &self,
        user_role_ids: &[i64],
    ) -> Result<bool, AppError> {
        if user_role_ids.is_empty() {
            return Ok(false);
        }

        Ok(sqlx::query_scalar::<_, bool>(
            r#"
SELECT EXISTS(
    SELECT 1
    FROM sys_user_role AS ur
    JOIN sys_user AS u ON u.id = ur.user_id
    JOIN sys_role AS r ON r.id = ur.role_id
    WHERE ur.id = ANY($1)
      AND u.is_system = TRUE
      AND (r.id = 1 OR r.code = 'admin')
);
"#,
        )
        .bind(normalized_ids(user_role_ids))
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn unassign_user_roles(&self, user_role_ids: &[i64]) -> Result<(), AppError> {
        if user_role_ids.is_empty() {
            return Ok(());
        }

        sqlx::query("DELETE FROM sys_user_role WHERE id = ANY($1);")
            .bind(normalized_ids(user_role_ids))
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn exists_by_unique_column(
        &self,
        column: &'static str,
        value: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT EXISTS(SELECT 1 FROM sys_role WHERE ");
        query.push(column).push(" = ").push_bind(value);
        if let Some(exclude_id) = exclude_id {
            query.push(" AND id <> ").push_bind(exclude_id);
        }
        query.push(")");

        Ok(query
            .build_query_scalar::<bool>()
            .fetch_one(&self.db)
            .await?)
    }
}

fn role_user_select_sql() -> &'static str {
    r#"
SELECT
    ur.id,
    ur.role_id,
    u.id AS user_id,
    u.username,
    u.nickname,
    u.gender,
    u.status,
    u.is_system,
    COALESCE(u.description, '') AS description,
    u.dept_id,
    COALESCE(d.name, '') AS dept_name,
    COALESCE(
        ARRAY_AGG(all_ur.role_id ORDER BY all_ur.role_id)
            FILTER (WHERE all_ur.role_id IS NOT NULL),
        ARRAY[]::BIGINT[]
    ) AS role_ids,
    COALESCE(
        ARRAY_AGG(r.name ORDER BY all_ur.role_id)
            FILTER (WHERE r.name IS NOT NULL),
        ARRAY[]::TEXT[]
    ) AS role_names
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user_role AS all_ur ON all_ur.user_id = u.id
LEFT JOIN sys_role AS r ON r.id = all_ur.role_id
"#
}

fn role_select_sql() -> &'static str {
    r#"
SELECT
    r.id,
    r.name,
    r.code,
    r.sort,
    COALESCE(r.description, '') AS description,
    r.data_scope,
    r.is_system,
    COALESCE(r.menu_check_strictly, TRUE) AS menu_check_strictly,
    COALESCE(r.dept_check_strictly, TRUE) AS dept_check_strictly,
    r.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    r.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
"#
}

fn push_role_user_description_filter(
    query: &mut QueryBuilder<'_, Postgres>,
    description: Option<&str>,
) {
    if let Some(description) = description.map(str::trim).filter(|value| !value.is_empty()) {
        let pattern = format!("%{description}%");
        query
            .push(" AND (u.username ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR u.nickname ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR COALESCE(u.description, '') ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

async fn insert_role_depts(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    role_id: i64,
    dept_ids: &[i64],
) -> Result<(), AppError> {
    for dept_id in normalized_ids(dept_ids) {
        sqlx::query(
            "INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
        )
        .bind(role_id)
        .bind(dept_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn insert_role_menus(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    role_id: i64,
    menu_ids: &[i64],
) -> Result<(), AppError> {
    for menu_id in normalized_ids(menu_ids) {
        sqlx::query(
            "INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
        )
        .bind(role_id)
        .bind(menu_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

fn normalized_ids(ids: &[i64]) -> Vec<i64> {
    let mut ids = ids.iter().copied().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_user_select_sorts_role_ids_and_names_by_role_id() {
        let sql = role_user_select_sql();

        assert!(sql.contains("ARRAY_AGG(all_ur.role_id ORDER BY all_ur.role_id)"));
        assert!(sql.contains("ARRAY_AGG(r.name ORDER BY all_ur.role_id)"));
    }
}
