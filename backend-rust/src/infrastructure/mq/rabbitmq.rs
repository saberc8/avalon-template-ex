use lapin::{
    options::*,
    publisher_confirm::Confirmation,
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use serde::{Deserialize, Serialize};

use crate::shared::error::AppError;

#[derive(Debug, Clone)]
pub struct RabbitMqConfig {
    pub url: String,
    pub exchange: String,
    pub execute_queue: String,
    pub retry_queue: String,
    pub dead_queue: String,
    pub execute_routing_key: String,
    pub retry_routing_key: String,
    pub dead_routing_key: String,
    pub retry_ttl_ms: u32,
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            url: "amqp://guest:guest@127.0.0.1:5672/%2f".to_owned(),
            exchange: "avalon.scheduler".to_owned(),
            execute_queue: "avalon.scheduler.execute".to_owned(),
            retry_queue: "avalon.scheduler.retry".to_owned(),
            dead_queue: "avalon.scheduler.dead".to_owned(),
            execute_routing_key: "scheduler.execute".to_owned(),
            retry_routing_key: "scheduler.retry".to_owned(),
            dead_routing_key: "scheduler.dead".to_owned(),
            retry_ttl_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerMessage {
    pub trigger_id: i64,
    pub job_id: i64,
    pub task_type: i16,
    pub attempt: i32,
    pub max_attempts: i32,
}

#[derive(Clone)]
pub struct RabbitMqClient {
    channel: Channel,
    config: RabbitMqConfig,
}

impl RabbitMqClient {
    pub async fn connect(config: RabbitMqConfig) -> Result<Self, AppError> {
        let connection = Connection::connect(&config.url, ConnectionProperties::default())
            .await
            .map_err(|error| AppError::Anyhow(anyhow::anyhow!("connect RabbitMQ: {error}")))?;
        let channel = connection.create_channel().await.map_err(|error| {
            AppError::Anyhow(anyhow::anyhow!("create RabbitMQ channel: {error}"))
        })?;
        let client = Self { channel, config };
        client.declare_topology().await?;
        Ok(client)
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn config(&self) -> &RabbitMqConfig {
        &self.config
    }

    pub async fn publish_execute(&self, message: &SchedulerMessage) -> Result<(), AppError> {
        self.publish(&self.config.execute_routing_key, message)
            .await
    }

    pub async fn publish_retry(&self, message: &SchedulerMessage) -> Result<(), AppError> {
        self.publish(&self.config.retry_routing_key, message).await
    }

    pub async fn publish_dead(&self, message: &SchedulerMessage) -> Result<(), AppError> {
        self.publish(&self.config.dead_routing_key, message).await
    }

    async fn publish(&self, routing_key: &str, message: &SchedulerMessage) -> Result<(), AppError> {
        let payload = serde_json::to_vec(message).map_err(|error| {
            AppError::Anyhow(anyhow::anyhow!("encode scheduler message: {error}"))
        })?;
        let confirmation = self
            .channel
            .basic_publish(
                &self.config.exchange,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_content_type("application/json".into()),
            )
            .await
            .map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("publish scheduler message: {error}"))
            })?
            .await
            .map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("confirm scheduler message: {error}"))
            })?;
        if !matches!(confirmation, Confirmation::Ack(_)) {
            return Err(AppError::Anyhow(anyhow::anyhow!(
                "RabbitMQ did not ack scheduler message"
            )));
        }
        Ok(())
    }

    async fn declare_topology(&self) -> Result<(), AppError> {
        self.channel
            .exchange_declare(
                &self.config.exchange,
                ExchangeKind::Direct,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("declare scheduler exchange: {error}"))
            })?;

        self.declare_queue(&self.config.execute_queue, FieldTable::default())
            .await?;
        self.bind_queue(&self.config.execute_queue, &self.config.execute_routing_key)
            .await?;

        let mut retry_args = FieldTable::default();
        retry_args.insert(
            "x-message-ttl".into(),
            AMQPValue::LongUInt(self.config.retry_ttl_ms),
        );
        retry_args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(self.config.exchange.clone().into()),
        );
        retry_args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(self.config.execute_routing_key.clone().into()),
        );
        self.declare_queue(&self.config.retry_queue, retry_args)
            .await?;
        self.bind_queue(&self.config.retry_queue, &self.config.retry_routing_key)
            .await?;

        self.declare_queue(&self.config.dead_queue, FieldTable::default())
            .await?;
        self.bind_queue(&self.config.dead_queue, &self.config.dead_routing_key)
            .await?;
        Ok(())
    }

    async fn declare_queue(&self, queue: &str, args: FieldTable) -> Result<(), AppError> {
        self.channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                args,
            )
            .await
            .map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("declare scheduler queue {queue}: {error}"))
            })?;
        Ok(())
    }

    async fn bind_queue(&self, queue: &str, routing_key: &str) -> Result<(), AppError> {
        self.channel
            .queue_bind(
                queue,
                &self.config.exchange,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|error| {
                AppError::Anyhow(anyhow::anyhow!("bind scheduler queue {queue}: {error}"))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_message_serializes_with_camel_case_fields() {
        let message = SchedulerMessage {
            trigger_id: 1,
            job_id: 2,
            task_type: 1,
            attempt: 1,
            max_attempts: 3,
        };

        let value = serde_json::to_value(message).unwrap();

        assert_eq!(value["triggerId"], 1);
        assert_eq!(value["jobId"], 2);
        assert_eq!(value["maxAttempts"], 3);
    }
}
