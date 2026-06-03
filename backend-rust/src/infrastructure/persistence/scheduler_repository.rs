use chrono::NaiveDateTime;
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::shared::{error::AppError, id::next_id};

#[derive(Debug, Clone)]
pub struct SchedulerRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobRecord {
    pub id: i64,
    pub name: String,
    pub group_name: String,
    pub task_type: i16,
    pub cron_expression: String,
    pub status: i16,
    pub concurrent: bool,
    pub misfire_policy: i16,
    pub max_retry: i32,
    pub timeout_seconds: i32,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: Value,
    pub http_body: String,
    pub builtin_key: String,
    pub description: String,
    pub last_trigger_time: Option<NaiveDateTime>,
    pub next_trigger_time: Option<NaiveDateTime>,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone)]
pub struct JobSaveRecord {
    pub id: i64,
    pub name: String,
    pub group_name: String,
    pub task_type: i16,
    pub cron_expression: String,
    pub status: i16,
    pub concurrent: bool,
    pub misfire_policy: i16,
    pub max_retry: i32,
    pub timeout_seconds: i32,
    pub http_method: Option<String>,
    pub http_url: Option<String>,
    pub http_headers: Value,
    pub http_body: Option<String>,
    pub builtin_key: Option<String>,
    pub description: Option<String>,
    pub user_id: i64,
    pub now: NaiveDateTime,
    pub next_trigger_time: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobTriggerRecord {
    pub id: i64,
    pub job_id: i64,
    pub source: i16,
    pub fire_time: NaiveDateTime,
    pub status: i16,
    pub attempt: i32,
    pub max_attempts: i32,
    pub error_msg: String,
    pub queued_time: Option<NaiveDateTime>,
    pub start_time: Option<NaiveDateTime>,
    pub finish_time: Option<NaiveDateTime>,
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobLogRecord {
    pub id: i64,
    pub trigger_id: i64,
    pub job_id: i64,
    pub attempt: i32,
    pub status: i16,
    pub executor: String,
    pub request_snapshot: Value,
    pub response_status: i32,
    pub response_body: String,
    pub error_msg: String,
    pub start_time: NaiveDateTime,
    pub finish_time: Option<NaiveDateTime>,
    pub time_taken: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PendingTriggerMessageRecord {
    pub trigger_id: i64,
    pub job_id: i64,
    pub task_type: i16,
    pub attempt: i32,
    pub max_attempts: i32,
}

#[derive(Debug, Clone)]
pub struct JobFilter<'a> {
    pub description: Option<&'a str>,
    pub group_name: Option<&'a str>,
    pub task_type: Option<i16>,
    pub status: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct JobLogFilter {
    pub job_id: i64,
    pub status: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSource {
    Schedule = 1,
    Manual = 2,
}

pub const TRIGGER_STATUS_PENDING: i16 = 1;
pub const TRIGGER_STATUS_QUEUED: i16 = 2;
pub const TRIGGER_STATUS_RUNNING: i16 = 3;
pub const TRIGGER_STATUS_SUCCESS: i16 = 4;
pub const TRIGGER_STATUS_FAILED: i16 = 5;
pub const TRIGGER_STATUS_DEAD: i16 = 6;

impl SchedulerRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn count_jobs(&self, filter: &JobFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_job j");
        push_job_filter(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_jobs(&self, filter: &JobFilter<'_>) -> Result<Vec<JobRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(job_select_sql());
        push_job_filter(&mut query, filter);
        query
            .push(" ORDER BY j.create_time DESC, j.id DESC LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);
        Ok(query
            .build_query_as::<JobRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_job(&self, id: i64) -> Result<Option<JobRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(job_select_sql());
        query.push(" WHERE j.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<JobRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn list_due_jobs(
        &self,
        now: NaiveDateTime,
        limit: i64,
    ) -> Result<Vec<JobRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(job_select_sql());
        query
            .push(" WHERE j.status = 1 AND j.next_trigger_time IS NOT NULL AND j.next_trigger_time <= ")
            .push_bind(now)
            .push(" ORDER BY j.next_trigger_time ASC, j.id ASC LIMIT ")
            .push_bind(limit);
        Ok(query
            .build_query_as::<JobRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn create_job(&self, record: &JobSaveRecord) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_job (
    id, name, group_name, task_type, cron_expression, status, concurrent,
    misfire_policy, max_retry, timeout_seconds, http_method, http_url,
    http_headers, http_body, builtin_key, description, next_trigger_time,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, $6, $7,
    $8, $9, $10, $11, $12,
    $13, $14, $15, $16, $17,
    $18, $19
);
"#,
        )
        .bind(record.id)
        .bind(&record.name)
        .bind(&record.group_name)
        .bind(record.task_type)
        .bind(&record.cron_expression)
        .bind(record.status)
        .bind(record.concurrent)
        .bind(record.misfire_policy)
        .bind(record.max_retry)
        .bind(record.timeout_seconds)
        .bind(&record.http_method)
        .bind(&record.http_url)
        .bind(&record.http_headers)
        .bind(&record.http_body)
        .bind(&record.builtin_key)
        .bind(&record.description)
        .bind(record.next_trigger_time)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_job(&self, record: &JobSaveRecord) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_job
SET name = $1,
    group_name = $2,
    task_type = $3,
    cron_expression = $4,
    status = $5,
    concurrent = $6,
    misfire_policy = $7,
    max_retry = $8,
    timeout_seconds = $9,
    http_method = $10,
    http_url = $11,
    http_headers = $12,
    http_body = $13,
    builtin_key = $14,
    description = $15,
    next_trigger_time = $16,
    update_user = $17,
    update_time = $18
WHERE id = $19;
"#,
        )
        .bind(&record.name)
        .bind(&record.group_name)
        .bind(record.task_type)
        .bind(&record.cron_expression)
        .bind(record.status)
        .bind(record.concurrent)
        .bind(record.misfire_policy)
        .bind(record.max_retry)
        .bind(record.timeout_seconds)
        .bind(&record.http_method)
        .bind(&record.http_url)
        .bind(&record.http_headers)
        .bind(&record.http_body)
        .bind(&record.builtin_key)
        .bind(&record.description)
        .bind(record.next_trigger_time)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn delete_jobs(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM sys_job_log WHERE job_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_job_trigger WHERE job_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_job WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn update_job_status(&self, id: i64, status: i16) -> Result<(), AppError> {
        let result =
            sqlx::query("UPDATE sys_job SET status = $1, update_time = NOW() WHERE id = $2;")
                .bind(status)
                .bind(id)
                .execute(&self.db)
                .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn update_job_trigger_times(
        &self,
        id: i64,
        last_trigger_time: NaiveDateTime,
        next_trigger_time: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_job
SET last_trigger_time = $1,
    next_trigger_time = $2,
    update_time = NOW()
WHERE id = $3;
"#,
        )
        .bind(last_trigger_time)
        .bind(next_trigger_time)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn create_trigger_from_job(
        &self,
        job: &JobRecord,
        source: TriggerSource,
        fire_time: NaiveDateTime,
        now: NaiveDateTime,
    ) -> Result<JobTriggerRecord, AppError> {
        let id = next_id();
        let payload = job_payload(job);
        sqlx::query(
            r#"
INSERT INTO sys_job_trigger (
    id, job_id, source, fire_time, status, attempt, max_attempts, payload, create_time
) VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $8);
"#,
        )
        .bind(id)
        .bind(job.id)
        .bind(source as i16)
        .bind(fire_time)
        .bind(TRIGGER_STATUS_PENDING)
        .bind(job.max_retry + 1)
        .bind(payload)
        .bind(now)
        .execute(&self.db)
        .await?;

        self.get_trigger(id)
            .await?
            .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("created trigger not found")))
    }

    pub async fn get_trigger(&self, id: i64) -> Result<Option<JobTriggerRecord>, AppError> {
        Ok(sqlx::query_as::<_, JobTriggerRecord>(
            r#"
SELECT id, job_id, source, fire_time, status, attempt, max_attempts,
       COALESCE(error_msg, '') AS error_msg,
       queued_time, start_time, finish_time, create_time
FROM sys_job_trigger
WHERE id = $1
LIMIT 1;
"#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn list_pending_messages(
        &self,
        limit: i64,
    ) -> Result<Vec<PendingTriggerMessageRecord>, AppError> {
        Ok(sqlx::query_as::<_, PendingTriggerMessageRecord>(
            r#"
SELECT t.id AS trigger_id,
       t.job_id,
       j.task_type,
       t.attempt + 1 AS attempt,
       t.max_attempts
FROM sys_job_trigger t
JOIN sys_job j ON j.id = t.job_id
WHERE t.status = $1
ORDER BY t.create_time ASC, t.id ASC
LIMIT $2;
"#,
        )
        .bind(TRIGGER_STATUS_PENDING)
        .bind(limit)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn mark_trigger_queued(&self, id: i64, now: NaiveDateTime) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_job_trigger
SET status = $1,
    queued_time = $2
WHERE id = $3 AND status = $4;
"#,
        )
        .bind(TRIGGER_STATUS_QUEUED)
        .bind(now)
        .bind(id)
        .bind(TRIGGER_STATUS_PENDING)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn mark_trigger_running(
        &self,
        id: i64,
        attempt: i32,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_job_trigger
SET status = $1,
    attempt = $2,
    start_time = $3
WHERE id = $4;
"#,
        )
        .bind(TRIGGER_STATUS_RUNNING)
        .bind(attempt)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn finish_trigger(
        &self,
        id: i64,
        status: i16,
        error_msg: Option<&str>,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_job_trigger
SET status = $1,
    error_msg = $2,
    finish_time = $3
WHERE id = $4;
"#,
        )
        .bind(status)
        .bind(error_msg)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn insert_job_log(&self, record: &JobLogInsertRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_job_log (
    id, trigger_id, job_id, attempt, status, executor, request_snapshot,
    response_status, response_body, error_msg, start_time, finish_time, time_taken
) VALUES (
    $1, $2, $3, $4, $5, $6, $7,
    $8, $9, $10, $11, $12, $13
);
"#,
        )
        .bind(record.id)
        .bind(record.trigger_id)
        .bind(record.job_id)
        .bind(record.attempt)
        .bind(record.status)
        .bind(record.executor)
        .bind(record.request_snapshot)
        .bind(record.response_status)
        .bind(record.response_body)
        .bind(record.error_msg)
        .bind(record.start_time)
        .bind(record.finish_time)
        .bind(record.time_taken)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn count_job_logs(&self, filter: &JobLogFilter) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_job_log l");
        push_job_log_filter(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_job_logs(
        &self,
        filter: &JobLogFilter,
    ) -> Result<Vec<JobLogRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(job_log_select_sql());
        push_job_log_filter(&mut query, filter);
        query
            .push(" ORDER BY l.start_time DESC, l.id DESC LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);
        Ok(query
            .build_query_as::<JobLogRecord>()
            .fetch_all(&self.db)
            .await?)
    }
}

#[derive(Debug, Clone)]
pub struct JobLogInsertRecord<'a> {
    pub id: i64,
    pub trigger_id: i64,
    pub job_id: i64,
    pub attempt: i32,
    pub status: i16,
    pub executor: &'a str,
    pub request_snapshot: &'a Value,
    pub response_status: Option<i32>,
    pub response_body: Option<&'a str>,
    pub error_msg: Option<&'a str>,
    pub start_time: NaiveDateTime,
    pub finish_time: Option<NaiveDateTime>,
    pub time_taken: i64,
}

pub fn normalized_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut ids = ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn job_select_sql() -> &'static str {
    r#"
SELECT j.id,
       j.name,
       j.group_name,
       j.task_type,
       j.cron_expression,
       j.status,
       j.concurrent,
       j.misfire_policy,
       j.max_retry,
       j.timeout_seconds,
       COALESCE(j.http_method, '') AS http_method,
       COALESCE(j.http_url, '') AS http_url,
       COALESCE(j.http_headers, '{}'::jsonb) AS http_headers,
       COALESCE(j.http_body, '') AS http_body,
       COALESCE(j.builtin_key, '') AS builtin_key,
       COALESCE(j.description, '') AS description,
       j.last_trigger_time,
       j.next_trigger_time,
       j.create_time,
       COALESCE(cu.nickname, cu.username, '') AS create_user_string,
       j.update_time,
       COALESCE(uu.nickname, uu.username, '') AS update_user_string
FROM sys_job j
LEFT JOIN sys_user cu ON cu.id = j.create_user
LEFT JOIN sys_user uu ON uu.id = j.update_user
"#
}

fn job_log_select_sql() -> &'static str {
    r#"
SELECT l.id,
       l.trigger_id,
       l.job_id,
       l.attempt,
       l.status,
       COALESCE(l.executor, '') AS executor,
       COALESCE(l.request_snapshot, '{}'::jsonb) AS request_snapshot,
       COALESCE(l.response_status, 0) AS response_status,
       COALESCE(l.response_body, '') AS response_body,
       COALESCE(l.error_msg, '') AS error_msg,
       l.start_time,
       l.finish_time,
       l.time_taken
FROM sys_job_log l
"#
}

fn push_job_filter(query: &mut QueryBuilder<'_, Postgres>, filter: &JobFilter<'_>) {
    query.push(" WHERE 1 = 1");
    if let Some(description) = non_empty(filter.description) {
        let pattern = format!("%{description}%");
        query
            .push(" AND (j.name ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR j.group_name ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR COALESCE(j.description, '') ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(group_name) = non_empty(filter.group_name) {
        query
            .push(" AND j.group_name = ")
            .push_bind(group_name.to_owned());
    }
    if let Some(task_type) = filter.task_type {
        query.push(" AND j.task_type = ").push_bind(task_type);
    }
    if let Some(status) = filter.status {
        query.push(" AND j.status = ").push_bind(status);
    }
}

fn push_job_log_filter(query: &mut QueryBuilder<'_, Postgres>, filter: &JobLogFilter) {
    query.push(" WHERE l.job_id = ").push_bind(filter.job_id);
    if let Some(status) = filter.status {
        query.push(" AND l.status = ").push_bind(status);
    }
}

fn job_payload(job: &JobRecord) -> Value {
    json!({
        "jobId": job.id,
        "taskType": job.task_type,
        "name": job.name,
        "httpMethod": job.http_method,
        "httpUrl": job.http_url,
        "builtinKey": job.builtin_key,
        "timeoutSeconds": job.timeout_seconds
    })
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn ensure_affected(rows: u64) -> Result<(), AppError> {
    if rows == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_ids_removes_invalid_and_duplicate_values() {
        assert_eq!(normalized_ids(vec![3, 0, 3, -1, 2]), vec![2, 3]);
    }
}
