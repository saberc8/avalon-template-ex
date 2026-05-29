use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    application::system::{ensure_max_chars, format_datetime, format_optional_datetime},
    infrastructure::persistence::system_misc_repositories::{
        new_id, normalized_ids, ClientFilter, ClientRecord, ClientSaveRecord, SystemMiscRepository,
    },
    shared::{
        error::AppError,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub client_type: Option<String>,
    #[serde(
        default,
        alias = "authType[]",
        deserialize_with = "deserialize_string_vec"
    )]
    pub auth_type: Vec<String>,
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
pub struct ClientCommand {
    #[serde(default)]
    pub client_type: String,
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    pub auth_type: Vec<String>,
    #[serde(default)]
    pub active_timeout: i64,
    #[serde(default)]
    pub timeout: i64,
    #[serde(default)]
    pub status: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResp {
    pub id: i64,
    pub client_id: String,
    pub client_type: String,
    pub auth_type: Vec<String>,
    pub active_timeout: i64,
    pub timeout: i64,
    pub status: i16,
    pub create_user: i64,
    pub create_time: String,
    pub update_user: i64,
    pub update_time: String,
    pub create_user_string: String,
    pub update_user_string: String,
}

#[derive(Debug, Clone)]
pub struct ClientService {
    repo: SystemMiscRepository,
}

impl ClientService {
    pub fn new(db: PgPool) -> Self {
        Self {
            repo: SystemMiscRepository::new(db),
        }
    }

    pub async fn page(&self, query: ClientQuery) -> Result<PageResult<ClientResp>, AppError> {
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let auth_types = normalize_auth_types(query.auth_type)?;
        let filter = ClientFilter {
            client_type: query.client_type.as_deref(),
            status: query.status,
            auth_types: &auth_types,
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.repo.count_clients(&filter).await?;
        let list = self
            .repo
            .list_clients(&filter)
            .await?
            .into_iter()
            .map(ClientResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn get(&self, id: i64) -> Result<ClientResp, AppError> {
        self.repo
            .get_client(id)
            .await?
            .map(ClientResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user_id: i64, command: ClientCommand) -> Result<i64, AppError> {
        let command = normalize_client_command(command)?;
        let id = new_id();
        let record = ClientSaveRecord {
            id,
            client_id: uuid::Uuid::new_v4().simple().to_string(),
            client_type: command.client_type,
            auth_type: json!(command.auth_type),
            active_timeout: command.active_timeout,
            timeout: command.timeout,
            status: command.status,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.create_client(&record).await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: ClientCommand,
    ) -> Result<(), AppError> {
        let command = normalize_client_command(command)?;
        if self.repo.get_client(id).await?.is_none() {
            return Err(AppError::NotFound);
        }
        let record = ClientSaveRecord {
            id,
            client_id: String::new(),
            client_type: command.client_type,
            auth_type: json!(command.auth_type),
            active_timeout: command.active_timeout,
            timeout: command.timeout,
            status: command.status,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.update_client(&record).await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        self.repo.delete_clients(&ids).await
    }
}

impl From<ClientRecord> for ClientResp {
    fn from(record: ClientRecord) -> Self {
        Self {
            id: record.id,
            client_id: record.client_id,
            client_type: record.client_type,
            auth_type: auth_type_vec(record.auth_type),
            active_timeout: record.active_timeout,
            timeout: record.timeout,
            status: record.status,
            create_user: record.create_user,
            create_time: format_datetime(record.create_time),
            update_user: record.update_user.unwrap_or_default(),
            update_time: format_optional_datetime(record.update_time),
            create_user_string: record.create_user_string,
            update_user_string: record.update_user_string,
        }
    }
}

pub fn normalize_client_command(mut command: ClientCommand) -> Result<ClientCommand, AppError> {
    command.client_type = command.client_type.trim().to_owned();
    command.auth_type = normalize_auth_types(command.auth_type)?;
    if command.client_type.is_empty() {
        return Err(AppError::bad_request("客户端类型不能为空"));
    }
    if command.auth_type.is_empty() {
        return Err(AppError::bad_request("认证类型不能为空"));
    }
    if command.active_timeout < -1 {
        return Err(AppError::bad_request("活跃超时时间不正确"));
    }
    if command.timeout < -1 {
        return Err(AppError::bad_request("超时时间不正确"));
    }
    if command.status == 0 {
        command.status = 1;
    }
    if command.status != 1 && command.status != 2 {
        return Err(AppError::bad_request("状态不正确"));
    }
    ensure_max_chars("客户端类型", &command.client_type, 50)?;
    Ok(command)
}

fn normalize_auth_types(values: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut auth_types = Vec::new();
    for value in values {
        for item in value.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            ensure_max_chars("认证类型", item, 50)?;
            auth_types.push(item.to_owned());
        }
    }
    auth_types.sort();
    auth_types.dedup();
    Ok(auth_types)
}

fn auth_type_vec(value: Value) -> Vec<String> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect(),
        Value::String(value) => normalize_auth_types(vec![value]).unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn deserialize_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::Null => Vec::new(),
        Value::String(value) => vec![value],
        Value::Array(values) => values
            .into_iter()
            .filter_map(|value| match value {
                Value::String(value) => Some(value),
                Value::Number(value) => Some(value.to_string()),
                Value::Bool(value) => Some(value.to_string()),
                _ => None,
            })
            .collect(),
        Value::Number(value) => vec![value.to_string()],
        Value::Bool(value) => vec![value.to_string()],
        Value::Object(_) => Vec::new(),
    })
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

    #[test]
    fn client_response_uses_vue_field_names_and_auth_type_array() {
        let value = serde_json::to_value(ClientResp {
            id: 1,
            client_id: "client-id".to_owned(),
            client_type: "PC".to_owned(),
            auth_type: vec!["ACCOUNT".to_owned(), "EMAIL".to_owned()],
            active_timeout: 1800,
            timeout: 86400,
            status: 1,
            create_user: 1,
            create_time: "2026-05-29 10:00:00".to_owned(),
            update_user: 0,
            update_time: String::new(),
            create_user_string: "admin".to_owned(),
            update_user_string: String::new(),
        })
        .unwrap();

        assert_eq!(value["clientId"], "client-id");
        assert_eq!(value["clientType"], "PC");
        assert_eq!(value["authType"], json!(["ACCOUNT", "EMAIL"]));
        assert_eq!(value["activeTimeout"], 1800);
        assert_eq!(value["createUserString"], "admin");
    }

    #[test]
    fn client_command_normalizes_comma_auth_types() {
        let command = normalize_client_command(ClientCommand {
            client_type: " PC ".to_owned(),
            auth_type: vec!["ACCOUNT,EMAIL".to_owned()],
            active_timeout: 1800,
            timeout: 86400,
            status: 0,
        })
        .unwrap();

        assert_eq!(command.client_type, "PC");
        assert_eq!(command.auth_type, vec!["ACCOUNT", "EMAIL"]);
        assert_eq!(command.status, 1);
    }
}
