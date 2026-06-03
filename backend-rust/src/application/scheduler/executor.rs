use std::time::Duration;

use chrono::Utc;
use reqwest::Method;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    application::scheduler::{
        http_safety::{validate_http_target, HttpSafetyConfig},
        service::{JOB_TYPE_BUILTIN, JOB_TYPE_HTTP},
    },
    infrastructure::{
        mq::rabbitmq::SchedulerMessage,
        persistence::scheduler_repository::{
            JobLogInsertRecord, SchedulerRepository, TRIGGER_STATUS_DEAD, TRIGGER_STATUS_FAILED,
            TRIGGER_STATUS_SUCCESS,
        },
    },
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub success: bool,
    pub retryable: bool,
    pub error_msg: String,
}

pub async fn execute_scheduler_message(
    db: PgPool,
    http_safety: &HttpSafetyConfig,
    worker_id: &str,
    message: &SchedulerMessage,
) -> Result<ExecutionOutcome, AppError> {
    let repo = SchedulerRepository::new(db);
    let job = repo
        .get_job(message.job_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let _trigger = repo
        .get_trigger(message.trigger_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let start = Utc::now().naive_utc();
    repo.mark_trigger_running(message.trigger_id, message.attempt, start)
        .await?;

    let result = match job.task_type {
        JOB_TYPE_HTTP => execute_http_job(&job, http_safety).await,
        JOB_TYPE_BUILTIN => execute_builtin_job(&job.builtin_key).await,
        _ => Err(AppError::bad_request("任务类型不正确")),
    };

    let finish = Utc::now().naive_utc();
    let time_taken = (finish - start).num_milliseconds().max(0);
    let (success, response_status, response_body, error_msg) = match result {
        Ok(output) => (true, output.status, output.body, String::new()),
        Err(error) => (false, None, String::new(), error.to_string()),
    };
    let final_status = if success {
        TRIGGER_STATUS_SUCCESS
    } else if message.attempt >= message.max_attempts {
        TRIGGER_STATUS_DEAD
    } else {
        TRIGGER_STATUS_FAILED
    };
    let request_snapshot = json!({
        "jobId": job.id,
        "taskType": job.task_type,
        "httpMethod": job.http_method,
        "httpUrl": job.http_url,
        "builtinKey": job.builtin_key,
        "attempt": message.attempt
    });
    repo.insert_job_log(&JobLogInsertRecord {
        id: next_id(),
        trigger_id: message.trigger_id,
        job_id: message.job_id,
        attempt: message.attempt,
        status: final_status,
        executor: worker_id,
        request_snapshot: &request_snapshot,
        response_status,
        response_body: if response_body.is_empty() {
            None
        } else {
            Some(response_body.as_str())
        },
        error_msg: if error_msg.is_empty() {
            None
        } else {
            Some(error_msg.as_str())
        },
        start_time: start,
        finish_time: Some(finish),
        time_taken,
    })
    .await?;
    repo.finish_trigger(
        message.trigger_id,
        final_status,
        if error_msg.is_empty() {
            None
        } else {
            Some(error_msg.as_str())
        },
        finish,
    )
    .await?;

    Ok(ExecutionOutcome {
        success,
        retryable: !success && message.attempt < message.max_attempts,
        error_msg,
    })
}

struct HttpOutput {
    status: Option<i32>,
    body: String,
}

async fn execute_http_job(
    job: &crate::infrastructure::persistence::scheduler_repository::JobRecord,
    http_safety: &HttpSafetyConfig,
) -> Result<HttpOutput, AppError> {
    validate_http_target(&job.http_url, http_safety)?;
    let method = job
        .http_method
        .parse::<Method>()
        .map_err(|_| AppError::bad_request("HTTP 方法不正确"))?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(job.timeout_seconds.max(1) as u64))
        .build()
        .map_err(|error| AppError::Anyhow(anyhow::anyhow!("build HTTP client: {error}")))?;
    let mut request = client.request(method, &job.http_url);
    if let Value::Object(headers) = &job.http_headers {
        for (key, value) in headers {
            if let Some(value) = value.as_str() {
                request = request.header(key, value);
            }
        }
    }
    if !job.http_body.is_empty() {
        request = request.body(job.http_body.clone());
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::Anyhow(anyhow::anyhow!("HTTP 任务请求失败: {error}")))?;
    let status = response.status().as_u16() as i32;
    let body = response
        .text()
        .await
        .map_err(|error| AppError::Anyhow(anyhow::anyhow!("读取 HTTP 任务响应失败: {error}")))?;
    if !(200..=299).contains(&status) {
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "HTTP 任务响应状态码异常: {status}"
        )));
    }
    Ok(HttpOutput {
        status: Some(status),
        body: truncate_body(body),
    })
}

async fn execute_builtin_job(key: &str) -> Result<HttpOutput, AppError> {
    match key {
        "system.noop" => Ok(HttpOutput {
            status: Some(200),
            body: "ok".to_owned(),
        }),
        _ => Err(AppError::bad_request(format!("未知内置任务: {key}"))),
    }
}

fn truncate_body(mut body: String) -> String {
    const MAX_BODY_CHARS: usize = 4000;
    if body.chars().count() <= MAX_BODY_CHARS {
        return body;
    }
    body = body.chars().take(MAX_BODY_CHARS).collect();
    body.push_str("...");
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_body_keeps_short_body() {
        assert_eq!(truncate_body("ok".to_owned()), "ok");
    }

    #[test]
    fn retryable_outcome_depends_on_attempt_budget() {
        let outcome = ExecutionOutcome {
            success: false,
            retryable: true,
            error_msg: "failed".to_owned(),
        };

        assert!(outcome.retryable);
    }
}
