use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::{
    domain::data_scope::model::DataScopeFilter,
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone)]
pub struct SystemUserRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub gender: i16,
    pub email: String,
    pub phone: String,
    pub description: String,
    pub status: i16,
    pub is_system: bool,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
    pub dept_id: i64,
    pub dept_name: String,
    pub role_ids: Vec<i64>,
    pub role_names: Vec<String>,
    pub pwd_reset_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct UserListFilter<'a> {
    pub description: Option<&'a str>,
    pub status: Option<i16>,
    pub create_time_start: Option<NaiveDateTime>,
    pub create_time_end: Option<NaiveDateTime>,
    pub dept_id: Option<i64>,
    pub user_ids: &'a [i64],
    pub role_id: Option<i64>,
    pub data_scope: &'a DataScopeFilter,
    pub order_by: &'a str,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct UserCreateRecord {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub password_hash: String,
    pub gender: i16,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub status: i16,
    pub dept_id: i64,
    pub role_ids: Vec<i64>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UserUpdateRecord {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub gender: i16,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub status: i16,
    pub dept_id: i64,
    pub role_ids: Vec<i64>,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

impl SystemUserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn count(&self, filter: &UserListFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_user AS u");
        query.push(" WHERE 1 = 1");
        push_user_filters(&mut query, filter);

        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list(&self, filter: &UserListFilter<'_>) -> Result<Vec<UserRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(user_select_sql());
        query.push(" WHERE 1 = 1");
        push_user_filters(&mut query, filter);
        query.push(user_group_by_sql());
        query.push(" ORDER BY ").push(filter.order_by);
        if let Some(limit) = filter.limit {
            query.push(" LIMIT ").push_bind(limit);
        }
        if let Some(offset) = filter.offset {
            query.push(" OFFSET ").push_bind(offset);
        }

        Ok(query
            .build_query_as::<UserRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get(&self, id: i64) -> Result<Option<UserRecord>, AppError> {
        let unrestricted = DataScopeFilter::unrestricted();
        let filter = UserListFilter {
            description: None,
            status: None,
            create_time_start: None,
            create_time_end: None,
            dept_id: None,
            user_ids: &[],
            role_id: None,
            data_scope: &unrestricted,
            order_by: "u.id DESC",
            limit: Some(1),
            offset: None,
        };
        let mut query = QueryBuilder::<Postgres>::new(user_select_sql());
        query.push(" WHERE u.id = ").push_bind(id);
        push_user_filters(&mut query, &filter);
        query.push(user_group_by_sql());
        query.push(" LIMIT 1");

        Ok(query
            .build_query_as::<UserRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn username_exists(
        &self,
        username: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists_by_unique_column("username", username, exclude_id)
            .await
    }

    pub async fn email_exists(
        &self,
        email: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists_by_unique_column("email", email, exclude_id)
            .await
    }

    pub async fn phone_exists(
        &self,
        phone: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists_by_unique_column("phone", phone, exclude_id)
            .await
    }

    pub async fn dept_exists(&self, dept_id: i64) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_dept WHERE id = $1);")
                .bind(dept_id)
                .fetch_one(&self.db)
                .await?,
        )
    }

    pub async fn role_ids_contain_admin(&self, role_ids: &[i64]) -> Result<bool, AppError> {
        let role_ids = normalized_ids(role_ids);
        if role_ids.is_empty() {
            return Ok(false);
        }

        Ok(sqlx::query_scalar::<_, bool>(
            r#"
SELECT EXISTS(
    SELECT 1
    FROM sys_role
    WHERE id = ANY($1) AND (id = 1 OR code = 'admin')
);
"#,
        )
        .bind(role_ids)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn missing_role_ids(&self, role_ids: &[i64]) -> Result<Vec<i64>, AppError> {
        let role_ids = normalized_ids(role_ids);
        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM sys_role WHERE id = ANY($1) ORDER BY id ASC;",
        )
        .bind(role_ids.clone())
        .fetch_all(&self.db)
        .await?;

        Ok(role_ids
            .into_iter()
            .filter(|id| !existing.contains(id))
            .collect())
    }

    pub async fn create(&self, record: &UserCreateRecord) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        sqlx::query(
            r#"
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar, description,
    status, is_system, pwd_reset_time, dept_id, create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, FALSE, $11, $12, $13, $14);
"#,
        )
        .bind(record.id)
        .bind(&record.username)
        .bind(&record.nickname)
        .bind(&record.password_hash)
        .bind(record.gender)
        .bind(&record.email)
        .bind(&record.phone)
        .bind(&record.avatar)
        .bind(&record.description)
        .bind(record.status)
        .bind(record.now)
        .bind(record.dept_id)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&mut *tx)
        .await?;

        replace_user_roles(&mut tx, record.id, &record.role_ids).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, record: &UserUpdateRecord) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        let result = sqlx::query(
            r#"
UPDATE sys_user
SET username = $1,
    nickname = $2,
    gender = $3,
    email = $4,
    phone = $5,
    avatar = $6,
    description = $7,
    status = $8,
    dept_id = $9,
    update_user = $10,
    update_time = $11
WHERE id = $12;
"#,
        )
        .bind(&record.username)
        .bind(&record.nickname)
        .bind(record.gender)
        .bind(&record.email)
        .bind(&record.phone)
        .bind(&record.avatar)
        .bind(&record.description)
        .bind(record.status)
        .bind(record.dept_id)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        replace_user_roles(&mut tx, record.id, &record.role_ids).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_many(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM sys_user_role WHERE user_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_user WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn replace_roles(&self, user_id: i64, role_ids: &[i64]) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        replace_user_roles(&mut tx, user_id, role_ids).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn password_hash(&self, user_id: i64) -> Result<Option<String>, AppError> {
        Ok(
            sqlx::query_scalar::<_, Option<String>>("SELECT password FROM sys_user WHERE id = $1;")
                .bind(user_id)
                .fetch_optional(&self.db)
                .await?
                .flatten(),
        )
    }

    pub async fn update_password(
        &self,
        user_id: i64,
        password_hash: &str,
        updater_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_user
SET password = $1,
    pwd_reset_time = $2,
    update_user = $3,
    update_time = $4
WHERE id = $5;
"#,
        )
        .bind(password_hash)
        .bind(now)
        .bind(updater_id)
        .bind(now)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn update_avatar(
        &self,
        user_id: i64,
        avatar: &str,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        update_profile_column(&self.db, "avatar", user_id, avatar, now).await
    }

    pub async fn update_basic_info(
        &self,
        user_id: i64,
        nickname: &str,
        gender: i16,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_user
SET nickname = $1,
    gender = $2,
    update_user = $3,
    update_time = $4
WHERE id = $5;
"#,
        )
        .bind(nickname)
        .bind(gender)
        .bind(user_id)
        .bind(now)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn update_phone(
        &self,
        user_id: i64,
        phone: &str,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        update_profile_column(&self.db, "phone", user_id, phone, now).await
    }

    pub async fn update_email(
        &self,
        user_id: i64,
        email: &str,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        update_profile_column(&self.db, "email", user_id, email, now).await
    }

    async fn exists_by_unique_column(
        &self,
        column: &'static str,
        value: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT EXISTS(SELECT 1 FROM sys_user WHERE ");
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

fn push_user_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &UserListFilter<'_>) {
    if let Some(description) = filter
        .description
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
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
    if let Some(status) = filter.status.filter(|status| *status > 0) {
        query.push(" AND u.status = ").push_bind(status);
    }
    if let Some(start) = filter.create_time_start {
        query.push(" AND u.create_time >= ").push_bind(start);
    }
    if let Some(end) = filter.create_time_end {
        query.push(" AND u.create_time <= ").push_bind(end);
    }
    if let Some(dept_id) = filter.dept_id.filter(|dept_id| *dept_id > 0) {
        query.push(" AND u.dept_id = ").push_bind(dept_id);
    }
    if !filter.user_ids.is_empty() {
        query.push(" AND u.id IN (");
        let mut separated = query.separated(", ");
        for id in filter.user_ids {
            separated.push_bind(*id);
        }
        separated.push_unseparated(")");
    }
    if let Some(role_id) = filter.role_id.filter(|role_id| *role_id > 0) {
        query.push(
            r#"
 AND EXISTS(
    SELECT 1
    FROM sys_user_role AS role_filter
    WHERE role_filter.user_id = u.id AND role_filter.role_id = "#,
        );
        query.push_bind(role_id).push(")");
    }
    filter.data_scope.append_and_clause(query);
}

fn user_select_sql() -> &'static str {
    r#"
SELECT
    u.id,
    u.username,
    u.nickname,
    COALESCE(u.avatar, '') AS avatar,
    u.gender,
    COALESCE(u.email, '') AS email,
    COALESCE(u.phone, '') AS phone,
    COALESCE(u.description, '') AS description,
    u.status,
    u.is_system,
    u.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    u.update_time,
    COALESCE(uu.nickname, '') AS update_user_string,
    u.dept_id,
    COALESCE(d.name, '') AS dept_name,
    COALESCE(
        ARRAY_AGG(ur.role_id ORDER BY ur.role_id)
            FILTER (WHERE ur.role_id IS NOT NULL),
        ARRAY[]::BIGINT[]
    ) AS role_ids,
    COALESCE(
        ARRAY_AGG(r.name ORDER BY ur.role_id)
            FILTER (WHERE r.name IS NOT NULL),
        ARRAY[]::TEXT[]
    ) AS role_names,
    u.pwd_reset_time
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
LEFT JOIN sys_user_role AS ur ON ur.user_id = u.id
LEFT JOIN sys_role AS r ON r.id = ur.role_id
"#
}

fn user_group_by_sql() -> &'static str {
    r#"
GROUP BY
    u.id,
    u.username,
    u.nickname,
    u.avatar,
    u.gender,
    u.email,
    u.phone,
    u.description,
    u.status,
    u.is_system,
    u.create_time,
    cu.nickname,
    u.update_time,
    uu.nickname,
    u.dept_id,
    d.name,
    u.pwd_reset_time
"#
}

async fn replace_user_roles(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: i64,
    role_ids: &[i64],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1;")
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    for role_id in normalized_ids(role_ids) {
        sqlx::query(
            r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
        )
        .bind(next_id())
        .bind(user_id)
        .bind(role_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn update_profile_column(
    db: &PgPool,
    column: &'static str,
    user_id: i64,
    value: &str,
    now: NaiveDateTime,
) -> Result<(), AppError> {
    let mut query = QueryBuilder::<Postgres>::new("UPDATE sys_user SET ");
    query
        .push(column)
        .push(" = ")
        .push_bind(value)
        .push(", update_user = ")
        .push_bind(user_id)
        .push(", update_time = ")
        .push_bind(now)
        .push(" WHERE id = ")
        .push_bind(user_id);

    let result = query.build().execute(db).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
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
    fn user_select_sorts_role_ids_and_names_by_role_id() {
        let sql = user_select_sql();

        assert!(sql.contains("ARRAY_AGG(ur.role_id ORDER BY ur.role_id)"));
        assert!(sql.contains("ARRAY_AGG(r.name ORDER BY ur.role_id)"));
    }
}
