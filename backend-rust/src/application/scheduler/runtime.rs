use std::time::Duration;

use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use sqlx::PgPool;

use crate::{
    application::scheduler::{
        executor::execute_scheduler_message,
        http_safety::{HttpAllowlistMode, HttpSafetyConfig},
        service::SchedulerService,
    },
    infrastructure::mq::rabbitmq::{RabbitMqClient, RabbitMqConfig, SchedulerMessage},
    shared::{config::AppConfig, error::AppError},
};

pub fn http_safety_from_config(config: &AppConfig) -> Result<HttpSafetyConfig, AppError> {
    let mode = match config
        .scheduler_http_allowlist_mode
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "strict" => HttpAllowlistMode::Strict,
        "default" | "" => HttpAllowlistMode::Default,
        "open" => HttpAllowlistMode::Open,
        _ => {
            return Err(AppError::bad_request(
                "SCHEDULER_HTTP_ALLOWLIST_MODE 不正确",
            ))
        }
    };
    Ok(HttpSafetyConfig {
        mode,
        allowlist: config.scheduler_http_allowlist.clone(),
    })
}

pub fn rabbitmq_from_config(config: &AppConfig) -> RabbitMqConfig {
    RabbitMqConfig {
        url: config.rabbitmq_url.clone(),
        exchange: config.rabbitmq_exchange.clone(),
        execute_queue: config.rabbitmq_execute_queue.clone(),
        retry_queue: config.rabbitmq_retry_queue.clone(),
        dead_queue: config.rabbitmq_dead_queue.clone(),
        execute_routing_key: config.rabbitmq_execute_routing_key.clone(),
        retry_routing_key: config.rabbitmq_retry_routing_key.clone(),
        dead_routing_key: config.rabbitmq_dead_routing_key.clone(),
        retry_ttl_ms: config.rabbitmq_retry_ttl_ms,
    }
}

pub fn spawn_scheduler_runtime(
    db: PgPool,
    http_safety: HttpSafetyConfig,
    rabbitmq: RabbitMqConfig,
    worker_id: String,
    tick_seconds: u64,
    batch_size: i64,
) {
    tokio::spawn(async move {
        if let Err(error) = run_scheduler_runtime(
            db,
            http_safety,
            rabbitmq,
            worker_id,
            tick_seconds,
            batch_size,
        )
        .await
        {
            tracing::error!(error = ?error, "scheduler runtime stopped");
        }
    });
}

pub async fn run_scheduler_runtime(
    db: PgPool,
    http_safety: HttpSafetyConfig,
    rabbitmq: RabbitMqConfig,
    worker_id: String,
    tick_seconds: u64,
    batch_size: i64,
) -> Result<(), AppError> {
    let mq = RabbitMqClient::connect(rabbitmq).await?;
    let publisher_db = db.clone();
    let publisher_safety = http_safety.clone();
    let publisher = mq.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(tick_seconds.max(1)));
        loop {
            interval.tick().await;
            let service = SchedulerService::new(publisher_db.clone(), publisher_safety.clone());
            match service.enqueue_due_jobs(batch_size).await {
                Ok(count) if count > 0 => tracing::debug!(count, "scheduler enqueued due jobs"),
                Ok(_) => {}
                Err(error) => tracing::error!(error = ?error, "enqueue due jobs failed"),
            }
            match service
                .publish_pending_triggers(&publisher, batch_size)
                .await
            {
                Ok(count) if count > 0 => tracing::debug!(count, "published scheduler triggers"),
                Ok(_) => {}
                Err(error) => tracing::error!(error = ?error, "publish pending triggers failed"),
            }
        }
    });

    consume_execute_queue(db, http_safety, mq, worker_id).await
}

async fn consume_execute_queue(
    db: PgPool,
    http_safety: HttpSafetyConfig,
    mq: RabbitMqClient,
    worker_id: String,
) -> Result<(), AppError> {
    let mut consumer = mq
        .channel()
        .basic_consume(
            &mq.config().execute_queue,
            &worker_id,
            BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .map_err(|error| AppError::Anyhow(anyhow::anyhow!("consume scheduler queue: {error}")))?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.map_err(|error| {
            AppError::Anyhow(anyhow::anyhow!("receive scheduler message: {error}"))
        })?;
        let message =
            serde_json::from_slice::<SchedulerMessage>(&delivery.data).map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("decode scheduler message: {error}"))
            })?;
        let outcome =
            execute_scheduler_message(db.clone(), &http_safety, &worker_id, &message).await;
        match outcome {
            Ok(outcome) if outcome.success => {}
            Ok(outcome) if outcome.retryable => {
                let mut retry = message.clone();
                retry.attempt += 1;
                mq.publish_retry(&retry).await?;
            }
            Ok(_) => {
                mq.publish_dead(&message).await?;
            }
            Err(error) => {
                tracing::error!(error = ?error, trigger_id = message.trigger_id, "execute scheduler message failed");
                mq.publish_dead(&message).await?;
            }
        }
        delivery
            .ack(BasicAckOptions::default())
            .await
            .map_err(|error| AppError::Anyhow(anyhow::anyhow!("ack scheduler message: {error}")))?;
    }

    Ok(())
}
