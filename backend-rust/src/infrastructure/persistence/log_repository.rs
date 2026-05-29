use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::{
    domain::data_scope::model::DataScopeFilter,
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone)]
pub struct LogRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct LogRecord {
    pub id: i64,
    pub trace_id: String,
    pub description: String,
    pub module: String,
    pub log_type: i16,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: String,
    pub request_body: String,
    pub status_code: i32,
    pub response_headers: String,
    pub response_body: String,
    pub time_taken: i64,
    pub ip: String,
    pub address: String,
    pub browser: String,
    pub os: String,
    pub status: i16,
    pub error_msg: String,
    pub create_user: Option<i64>,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
}

#[derive(Debug, Clone)]
pub struct LogListFilter<'a> {
    pub description: Option<&'a str>,
    pub module: Option<&'a str>,
    pub ip: Option<&'a str>,
    pub create_user_string: Option<&'a str>,
    pub status: Option<i16>,
    pub log_type: Option<i16>,
    pub create_time_start: Option<NaiveDateTime>,
    pub create_time_end: Option<NaiveDateTime>,
    pub data_scope: &'a DataScopeFilter,
    pub order_by: &'a str,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct LogCreateRecord<'a> {
    pub id: i64,
    pub trace_id: &'a str,
    pub description: &'a str,
    pub module: &'a str,
    pub log_type: i16,
    pub request_url: &'a str,
    pub request_method: &'a str,
    pub request_headers: &'a str,
    pub request_body: &'a str,
    pub status_code: i32,
    pub response_headers: &'a str,
    pub response_body: &'a str,
    pub time_taken: i64,
    pub ip: &'a str,
    pub address: &'a str,
    pub browser: &'a str,
    pub os: &'a str,
    pub status: i16,
    pub error_msg: &'a str,
    pub create_user: Option<i64>,
    pub create_time: NaiveDateTime,
}

impl LogRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn count(&self, filter: &LogListFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT COUNT(*) FROM sys_log AS l LEFT JOIN sys_user AS cu ON cu.id = l.create_user",
        );
        query.push(" WHERE 1 = 1");
        push_log_filters(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list(&self, filter: &LogListFilter<'_>) -> Result<Vec<LogRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(log_select_sql());
        query.push(" WHERE 1 = 1");
        push_log_filters(&mut query, filter);
        query.push(" ORDER BY ").push(filter.order_by);
        if let Some(limit) = filter.limit {
            query.push(" LIMIT ").push_bind(limit);
        }
        if let Some(offset) = filter.offset {
            query.push(" OFFSET ").push_bind(offset);
        }

        Ok(query
            .build_query_as::<LogRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get(&self, id: i64) -> Result<Option<LogRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(log_select_sql());
        query.push(" WHERE l.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<LogRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn insert(&self, record: &LogCreateRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_log (
    id, trace_id, description, module, type, request_url, request_method,
    request_headers, request_body, status_code, response_headers, response_body,
    time_taken, ip, address, browser, os, status, error_msg, create_user, create_time
)
VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
    $15, $16, $17, $18, $19, $20, $21
);
"#,
        )
        .bind(record.id)
        .bind(record.trace_id)
        .bind(record.description)
        .bind(record.module)
        .bind(record.log_type)
        .bind(record.request_url)
        .bind(record.request_method)
        .bind(record.request_headers)
        .bind(record.request_body)
        .bind(record.status_code)
        .bind(record.response_headers)
        .bind(record.response_body)
        .bind(record.time_taken)
        .bind(record.ip)
        .bind(record.address)
        .bind(record.browser)
        .bind(record.os)
        .bind(record.status)
        .bind(record.error_msg)
        .bind(record.create_user)
        .bind(record.create_time)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

pub fn new_log_id() -> i64 {
    next_id()
}

fn log_select_sql() -> &'static str {
    r#"
SELECT
    l.id,
    COALESCE(l.trace_id, '') AS trace_id,
    l.description,
    l.module,
    l.type AS log_type,
    l.request_url,
    l.request_method,
    COALESCE(l.request_headers, '') AS request_headers,
    COALESCE(l.request_body, '') AS request_body,
    l.status_code,
    COALESCE(l.response_headers, '') AS response_headers,
    COALESCE(l.response_body, '') AS response_body,
    l.time_taken,
    COALESCE(l.ip, '') AS ip,
    COALESCE(l.address, '') AS address,
    COALESCE(l.browser, '') AS browser,
    COALESCE(l.os, '') AS os,
    l.status,
    COALESCE(l.error_msg, '') AS error_msg,
    l.create_user,
    l.create_time,
    COALESCE(cu.nickname, '') AS create_user_string
FROM sys_log AS l
LEFT JOIN sys_user AS cu ON cu.id = l.create_user
"#
}

fn push_log_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &LogListFilter<'_>) {
    if let Some(description) = non_empty(filter.description) {
        query
            .push(" AND l.description ILIKE ")
            .push_bind(format!("%{description}%"));
    }
    if let Some(module) = non_empty(filter.module) {
        query
            .push(" AND l.module ILIKE ")
            .push_bind(format!("%{module}%"));
    }
    if let Some(ip) = non_empty(filter.ip) {
        let pattern = format!("%{ip}%");
        query
            .push(" AND (COALESCE(l.ip, '') ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR COALESCE(l.address, '') ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(create_user_string) = non_empty(filter.create_user_string) {
        query
            .push(" AND COALESCE(cu.nickname, '') ILIKE ")
            .push_bind(format!("%{create_user_string}%"));
    }
    if let Some(status) = filter.status.filter(|value| *value > 0) {
        query.push(" AND l.status = ").push_bind(status);
    }
    if let Some(log_type) = filter.log_type.filter(|value| *value > 0) {
        query.push(" AND l.type = ").push_bind(log_type);
    }
    if let Some(start) = filter.create_time_start {
        query.push(" AND l.create_time >= ").push_bind(start);
    }
    if let Some(end) = filter.create_time_end {
        query.push(" AND l.create_time <= ").push_bind(end);
    }
    filter.data_scope.append_and_clause(query);
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
