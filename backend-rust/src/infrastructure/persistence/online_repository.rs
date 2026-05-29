use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::{
    domain::data_scope::model::DataScopeFilter,
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone)]
pub struct OnlineRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct OnlineRecord {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub client_type: String,
    pub client_id: String,
    pub ip: String,
    pub address: String,
    pub browser: String,
    pub os: String,
    pub login_time: NaiveDateTime,
    pub last_active_time: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct OnlineListFilter<'a> {
    pub nickname: Option<&'a str>,
    pub login_time_start: Option<NaiveDateTime>,
    pub login_time_end: Option<NaiveDateTime>,
    pub data_scope: &'a DataScopeFilter,
    pub order_by: &'a str,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct OnlineSaveRecord<'a> {
    pub id: i64,
    pub token: &'a str,
    pub user_id: i64,
    pub username: &'a str,
    pub nickname: &'a str,
    pub client_type: &'a str,
    pub client_id: &'a str,
    pub ip: &'a str,
    pub address: &'a str,
    pub browser: &'a str,
    pub os: &'a str,
    pub now: NaiveDateTime,
}

impl OnlineRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn count(&self, filter: &OnlineListFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT COUNT(*) FROM sys_online_user AS o LEFT JOIN sys_user AS cu ON cu.id = o.user_id",
        );
        query.push(" WHERE 1 = 1");
        push_online_filters(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list(&self, filter: &OnlineListFilter<'_>) -> Result<Vec<OnlineRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(online_select_sql());
        query.push(" WHERE 1 = 1");
        push_online_filters(&mut query, filter);
        query
            .push(" ORDER BY ")
            .push(filter.order_by)
            .push(" LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);

        Ok(query
            .build_query_as::<OnlineRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn save_login(&self, record: &OnlineSaveRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_online_user (
    id, token, user_id, username, nickname, client_type, client_id, ip, address,
    browser, os, login_time, last_active_time, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12, $12)
ON CONFLICT (token) DO UPDATE
SET last_active_time = EXCLUDED.last_active_time;
"#,
        )
        .bind(record.id)
        .bind(record.token)
        .bind(record.user_id)
        .bind(record.username)
        .bind(record.nickname)
        .bind(record.client_type)
        .bind(record.client_id)
        .bind(record.ip)
        .bind(record.address)
        .bind(record.browser)
        .bind(record.os)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_by_token(&self, token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sys_online_user WHERE token = $1;")
            .bind(token)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

pub fn new_online_id() -> i64 {
    next_id()
}

fn online_select_sql() -> &'static str {
    r#"
SELECT
    o.id,
    o.token,
    o.user_id,
    o.username,
    o.nickname,
    o.client_type,
    o.client_id,
    COALESCE(o.ip, '') AS ip,
    COALESCE(o.address, '') AS address,
    COALESCE(o.browser, '') AS browser,
    COALESCE(o.os, '') AS os,
    o.login_time,
    o.last_active_time
FROM sys_online_user AS o
LEFT JOIN sys_user AS cu ON cu.id = o.user_id
"#
}

fn push_online_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &OnlineListFilter<'_>) {
    if let Some(nickname) = non_empty(filter.nickname) {
        let pattern = format!("%{nickname}%");
        query
            .push(" AND (o.nickname ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR o.username ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(start) = filter.login_time_start {
        query.push(" AND o.login_time >= ").push_bind(start);
    }
    if let Some(end) = filter.login_time_end {
        query.push(" AND o.login_time <= ").push_bind(end);
    }
    filter.data_scope.append_and_clause(query);
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
