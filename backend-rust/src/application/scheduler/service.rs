use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    application::{
        scheduler::{
            cron::next_fire_time,
            http_safety::{validate_http_target, HttpSafetyConfig},
        },
        system::{ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none},
    },
    infrastructure::mq::rabbitmq::{RabbitMqClient, SchedulerMessage},
    infrastructure::persistence::scheduler_repository::{
        normalized_ids, JobFilter, JobLogFilter, JobLogRecord, JobRecord, JobSaveRecord,
        JobTriggerRecord, SchedulerRepository, TriggerSource,
    },
    shared::{
        error::AppError,
        id::next_id,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

pub const JOB_TYPE_HTTP: i16 = 1;
pub const JOB_TYPE_BUILTIN: i16 = 2;
pub const JOB_STATUS_ENABLED: i16 = 1;
pub const JOB_STATUS_DISABLED: i16 = 2;
pub const MISFIRE_FIRE_ONCE: i16 = 1;
pub const MISFIRE_SKIP: i16 = 2;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub group_name: Option<String>,
    #[serde(default)]
    pub task_type: Option<i16>,
    #[serde(default)]
    pub status: Option<i16>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCommand {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub group_name: String,
    #[serde(default)]
    pub task_type: i16,
    #[serde(default)]
    pub cron_expression: String,
    #[serde(default)]
    pub status: i16,
    #[serde(default)]
    pub concurrent: bool,
    #[serde(default)]
    pub misfire_policy: i16,
    #[serde(default)]
    pub max_retry: i32,
    #[serde(default)]
    pub timeout_seconds: i32,
    #[serde(default)]
    pub http_method: String,
    #[serde(default)]
    pub http_url: String,
    #[serde(default)]
    pub http_headers: Value,
    #[serde(default)]
    pub http_body: String,
    #[serde(default)]
    pub builtin_key: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatusCommand {
    pub status: i16,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLogQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub status: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResp {
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
    pub last_trigger_time: String,
    pub next_trigger_time: String,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTriggerResp {
    pub id: i64,
    pub job_id: i64,
    pub source: i16,
    pub fire_time: String,
    pub status: i16,
    pub attempt: i32,
    pub max_attempts: i32,
    pub error_msg: String,
    pub queued_time: String,
    pub start_time: String,
    pub finish_time: String,
    pub create_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLogResp {
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
    pub start_time: String,
    pub finish_time: String,
    pub time_taken: i64,
}

#[derive(Debug, Clone)]
pub struct SchedulerService {
    repo: SchedulerRepository,
    http_safety: HttpSafetyConfig,
}

impl SchedulerService {
    pub fn new(db: PgPool, http_safety: HttpSafetyConfig) -> Self {
        Self {
            repo: SchedulerRepository::new(db),
            http_safety,
        }
    }

    pub async fn page(&self, query: JobQuery) -> Result<PageResult<JobResp>, AppError> {
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let filter = JobFilter {
            description: query.description.as_deref(),
            group_name: query.group_name.as_deref(),
            task_type: query.task_type,
            status: query.status,
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.repo.count_jobs(&filter).await?;
        let list = self
            .repo
            .list_jobs(&filter)
            .await?
            .into_iter()
            .map(JobResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn get(&self, id: i64) -> Result<JobResp, AppError> {
        self.repo
            .get_job(id)
            .await?
            .map(JobResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user_id: i64, command: JobCommand) -> Result<i64, AppError> {
        let command = normalize_job_command(command, &self.http_safety)?;
        let id = next_id();
        let now = Utc::now();
        let next_trigger_time = next_fire_time(&command.cron_expression, now)?.naive_utc();
        let record =
            JobSaveRecord::from_command(id, user_id, now.naive_utc(), next_trigger_time, &command);
        self.repo.create_job(&record).await?;
        Ok(id)
    }

    pub async fn update(&self, user_id: i64, id: i64, command: JobCommand) -> Result<(), AppError> {
        let command = normalize_job_command(command, &self.http_safety)?;
        if self.repo.get_job(id).await?.is_none() {
            return Err(AppError::NotFound);
        }
        let now = Utc::now();
        let next_trigger_time = next_fire_time(&command.cron_expression, now)?.naive_utc();
        let record =
            JobSaveRecord::from_command(id, user_id, now.naive_utc(), next_trigger_time, &command);
        self.repo.update_job(&record).await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        self.repo.delete_jobs(&ids).await
    }

    pub async fn update_status(&self, id: i64, status: i16) -> Result<(), AppError> {
        if status != JOB_STATUS_ENABLED && status != JOB_STATUS_DISABLED {
            return Err(AppError::bad_request("任务状态不正确"));
        }
        self.repo.update_job_status(id, status).await
    }

    pub async fn run_once(&self, id: i64) -> Result<JobTriggerResp, AppError> {
        let job = self.repo.get_job(id).await?.ok_or(AppError::NotFound)?;
        let now = Utc::now().naive_utc();
        let trigger = self
            .repo
            .create_trigger_from_job(&job, TriggerSource::Manual, now, now)
            .await?;
        Ok(JobTriggerResp::from(trigger))
    }

    pub async fn log_page(
        &self,
        job_id: i64,
        query: JobLogQuery,
    ) -> Result<PageResult<JobLogResp>, AppError> {
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let filter = JobLogFilter {
            job_id,
            status: query.status,
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.repo.count_job_logs(&filter).await?;
        let list = self
            .repo
            .list_job_logs(&filter)
            .await?
            .into_iter()
            .map(JobLogResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn enqueue_due_jobs(&self, limit: i64) -> Result<usize, AppError> {
        let now = Utc::now();
        let jobs = self.repo.list_due_jobs(now.naive_utc(), limit).await?;
        let mut count = 0usize;
        for job in jobs {
            let fire_time = job.next_trigger_time.unwrap_or_else(|| now.naive_utc());
            let next = next_fire_time(&job.cron_expression, now)?.naive_utc();
            self.repo
                .create_trigger_from_job(&job, TriggerSource::Schedule, fire_time, now.naive_utc())
                .await?;
            self.repo
                .update_job_trigger_times(job.id, fire_time, next)
                .await?;
            count += 1;
        }
        Ok(count)
    }

    pub async fn publish_pending_triggers(
        &self,
        publisher: &RabbitMqClient,
        limit: i64,
    ) -> Result<usize, AppError> {
        let pending = self.repo.list_pending_messages(limit).await?;
        let now = Utc::now().naive_utc();
        let mut count = 0usize;
        for item in pending {
            let message = SchedulerMessage {
                trigger_id: item.trigger_id,
                job_id: item.job_id,
                task_type: item.task_type,
                attempt: item.attempt,
                max_attempts: item.max_attempts,
            };
            publisher.publish_execute(&message).await?;
            self.repo.mark_trigger_queued(item.trigger_id, now).await?;
            count += 1;
        }
        Ok(count)
    }
}

pub fn normalize_job_command(
    mut command: JobCommand,
    http_safety: &HttpSafetyConfig,
) -> Result<JobCommand, AppError> {
    command.name = command.name.trim().to_owned();
    command.group_name = command.group_name.trim().to_owned();
    command.cron_expression = command.cron_expression.trim().to_owned();
    command.http_method = command.http_method.trim().to_ascii_uppercase();
    command.http_url = command.http_url.trim().to_owned();
    command.http_body = command.http_body.trim().to_owned();
    command.builtin_key = command.builtin_key.trim().to_owned();
    command.description = command.description.trim().to_owned();

    if command.name.is_empty() {
        return Err(AppError::bad_request("任务名称不能为空"));
    }
    if command.group_name.is_empty() {
        command.group_name = "default".to_owned();
    }
    ensure_max_chars("任务名称", &command.name, 100)?;
    ensure_max_chars("任务分组", &command.group_name, 50)?;
    ensure_max_chars("内置任务标识", &command.builtin_key, 120)?;
    ensure_max_chars("任务描述", &command.description, 255)?;
    crate::application::scheduler::cron::validate_cron_expression(&command.cron_expression)?;

    if command.status == 0 {
        command.status = JOB_STATUS_DISABLED;
    }
    if command.status != JOB_STATUS_ENABLED && command.status != JOB_STATUS_DISABLED {
        return Err(AppError::bad_request("任务状态不正确"));
    }
    if command.misfire_policy == 0 {
        command.misfire_policy = MISFIRE_FIRE_ONCE;
    }
    if command.misfire_policy != MISFIRE_FIRE_ONCE && command.misfire_policy != MISFIRE_SKIP {
        return Err(AppError::bad_request("错过触发策略不正确"));
    }
    if command.max_retry < 0 || command.max_retry > 10 {
        return Err(AppError::bad_request("最大重试次数必须在 0 到 10 之间"));
    }
    if command.timeout_seconds == 0 {
        command.timeout_seconds = 30;
    }
    if !(1..=3600).contains(&command.timeout_seconds) {
        return Err(AppError::bad_request("超时时间必须在 1 到 3600 秒之间"));
    }

    match command.task_type {
        JOB_TYPE_HTTP => normalize_http_command(command, http_safety),
        JOB_TYPE_BUILTIN => normalize_builtin_command(command),
        _ => Err(AppError::bad_request("任务类型不正确")),
    }
}

fn normalize_http_command(
    mut command: JobCommand,
    http_safety: &HttpSafetyConfig,
) -> Result<JobCommand, AppError> {
    if command.http_method.is_empty() {
        command.http_method = "POST".to_owned();
    }
    if !matches!(
        command.http_method.as_str(),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
    ) {
        return Err(AppError::bad_request("HTTP 方法不正确"));
    }
    validate_http_target(&command.http_url, http_safety)?;
    if !command.http_headers.is_object() && !command.http_headers.is_null() {
        return Err(AppError::bad_request("HTTP Headers 必须是 JSON 对象"));
    }
    if command.http_headers.is_null() {
        command.http_headers = json!({});
    }
    command.builtin_key.clear();
    Ok(command)
}

fn normalize_builtin_command(mut command: JobCommand) -> Result<JobCommand, AppError> {
    if command.builtin_key.is_empty() {
        return Err(AppError::bad_request("内置任务标识不能为空"));
    }
    command.http_method.clear();
    command.http_url.clear();
    command.http_headers = json!({});
    command.http_body.clear();
    Ok(command)
}

impl JobSaveRecord {
    fn from_command(
        id: i64,
        user_id: i64,
        now: NaiveDateTime,
        next_trigger_time: NaiveDateTime,
        command: &JobCommand,
    ) -> Self {
        Self {
            id,
            name: command.name.clone(),
            group_name: command.group_name.clone(),
            task_type: command.task_type,
            cron_expression: command.cron_expression.clone(),
            status: command.status,
            concurrent: command.concurrent,
            misfire_policy: command.misfire_policy,
            max_retry: command.max_retry,
            timeout_seconds: command.timeout_seconds,
            http_method: trim_to_none(command.http_method.clone()),
            http_url: trim_to_none(command.http_url.clone()),
            http_headers: command.http_headers.clone(),
            http_body: trim_to_none(command.http_body.clone()),
            builtin_key: trim_to_none(command.builtin_key.clone()),
            description: trim_to_none(command.description.clone()),
            user_id,
            now,
            next_trigger_time,
        }
    }
}

impl From<JobRecord> for JobResp {
    fn from(record: JobRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            group_name: record.group_name,
            task_type: record.task_type,
            cron_expression: record.cron_expression,
            status: record.status,
            concurrent: record.concurrent,
            misfire_policy: record.misfire_policy,
            max_retry: record.max_retry,
            timeout_seconds: record.timeout_seconds,
            http_method: record.http_method,
            http_url: record.http_url,
            http_headers: record.http_headers,
            http_body: record.http_body,
            builtin_key: record.builtin_key,
            description: record.description,
            last_trigger_time: format_optional_datetime(record.last_trigger_time),
            next_trigger_time: format_optional_datetime(record.next_trigger_time),
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
        }
    }
}

impl From<JobTriggerRecord> for JobTriggerResp {
    fn from(record: JobTriggerRecord) -> Self {
        Self {
            id: record.id,
            job_id: record.job_id,
            source: record.source,
            fire_time: format_datetime(record.fire_time),
            status: record.status,
            attempt: record.attempt,
            max_attempts: record.max_attempts,
            error_msg: record.error_msg,
            queued_time: format_optional_datetime(record.queued_time),
            start_time: format_optional_datetime(record.start_time),
            finish_time: format_optional_datetime(record.finish_time),
            create_time: format_datetime(record.create_time),
        }
    }
}

impl From<JobLogRecord> for JobLogResp {
    fn from(record: JobLogRecord) -> Self {
        Self {
            id: record.id,
            trigger_id: record.trigger_id,
            job_id: record.job_id,
            attempt: record.attempt,
            status: record.status,
            executor: record.executor,
            request_snapshot: record.request_snapshot,
            response_status: record.response_status,
            response_body: record.response_body,
            error_msg: record.error_msg,
            start_time: format_datetime(record.start_time),
            finish_time: format_optional_datetime(record.finish_time),
            time_taken: record.time_taken,
        }
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_size() -> u64 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::scheduler::http_safety::HttpAllowlistMode;

    fn safety_config() -> HttpSafetyConfig {
        HttpSafetyConfig {
            mode: HttpAllowlistMode::Default,
            allowlist: vec!["api.example.com".to_owned()],
        }
    }

    fn base_command() -> JobCommand {
        JobCommand {
            name: "  Sync Orders  ".to_owned(),
            group_name: String::new(),
            task_type: JOB_TYPE_HTTP,
            cron_expression: "*/5 * * * * *".to_owned(),
            status: 0,
            concurrent: false,
            misfire_policy: 0,
            max_retry: 2,
            timeout_seconds: 0,
            http_method: String::new(),
            http_url: "https://api.example.com/sync".to_owned(),
            http_headers: Value::Null,
            http_body: String::new(),
            builtin_key: "ignored".to_owned(),
            description: " test ".to_owned(),
        }
    }

    #[test]
    fn normalize_http_job_applies_defaults_and_clears_builtin_key() {
        let command = normalize_job_command(base_command(), &safety_config()).unwrap();

        assert_eq!(command.name, "Sync Orders");
        assert_eq!(command.group_name, "default");
        assert_eq!(command.status, JOB_STATUS_DISABLED);
        assert_eq!(command.misfire_policy, MISFIRE_FIRE_ONCE);
        assert_eq!(command.timeout_seconds, 30);
        assert_eq!(command.http_method, "POST");
        assert_eq!(command.http_headers, json!({}));
        assert_eq!(command.builtin_key, "");
    }

    #[test]
    fn normalize_http_job_rejects_url_outside_allowlist() {
        let mut command = base_command();
        command.http_url = "https://blocked.example.net/sync".to_owned();

        let err = normalize_job_command(command, &safety_config()).unwrap_err();

        assert!(err.to_string().contains("allowlist"));
    }

    #[test]
    fn normalize_builtin_job_requires_builtin_key_and_clears_http_fields() {
        let mut command = base_command();
        command.task_type = JOB_TYPE_BUILTIN;
        command.builtin_key = "system.noop".to_owned();

        let command = normalize_job_command(command, &safety_config()).unwrap();

        assert_eq!(command.builtin_key, "system.noop");
        assert_eq!(command.http_url, "");
        assert_eq!(command.http_headers, json!({}));
    }

    #[test]
    fn normalize_job_rejects_retry_outside_range() {
        let mut command = base_command();
        command.max_retry = 11;

        let err = normalize_job_command(command, &safety_config()).unwrap_err();

        assert!(err.to_string().contains("最大重试次数"));
    }
}
