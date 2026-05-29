use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::system::{ensure_max_chars, format_datetime, format_optional_datetime},
    infrastructure::persistence::system_misc_repositories::{
        new_id, normalized_ids, StorageFilter, StorageRecord, StorageSaveRecord,
        SystemMiscRepository,
    },
    shared::error::AppError,
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageQuery {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub storage_type: Option<i16>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageCommand {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default, rename = "type")]
    pub storage_type: i16,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub bucket_name: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub status: i16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatusCommand {
    #[serde(default)]
    pub status: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    #[serde(rename = "type")]
    pub storage_type: i16,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub region: String,
    pub bucket_name: String,
    pub domain: String,
    pub description: String,
    pub is_default: bool,
    pub sort: i32,
    pub status: i16,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
}

#[derive(Debug, Clone)]
pub struct StorageService {
    repo: SystemMiscRepository,
}

impl StorageService {
    pub fn new(db: PgPool) -> Self {
        Self {
            repo: SystemMiscRepository::new(db),
        }
    }

    pub async fn list(&self, query: StorageQuery) -> Result<Vec<StorageResp>, AppError> {
        Ok(self
            .repo
            .list_storages(&StorageFilter {
                description: query.description.as_deref(),
                storage_type: query.storage_type,
            })
            .await?
            .into_iter()
            .map(StorageResp::from)
            .collect())
    }

    pub async fn get(&self, id: i64) -> Result<StorageResp, AppError> {
        self.repo
            .get_storage(id)
            .await?
            .map(StorageResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user_id: i64, command: StorageCommand) -> Result<i64, AppError> {
        let command = normalize_storage_command(command)?;
        ensure_unique_code(&self.repo, &command.code, None).await?;
        let id = new_id();
        let record = storage_record(id, user_id, &command);
        self.repo.create_storage(&record).await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: StorageCommand,
    ) -> Result<(), AppError> {
        let mut command = normalize_storage_command(command)?;
        let existing = self.repo.get_storage(id).await?.ok_or(AppError::NotFound)?;
        ensure_unique_code(&self.repo, &command.code, Some(id)).await?;
        if existing.is_default {
            command.is_default = true;
            if command.status != 1 {
                return Err(AppError::bad_request("默认存储不允许禁用"));
            }
        }
        let record = storage_record(id, user_id, &command);
        self.repo.update_storage(&record).await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        for id in &ids {
            let storage = self
                .repo
                .get_storage(*id)
                .await?
                .ok_or(AppError::NotFound)?;
            if storage.is_default {
                return Err(AppError::bad_request(format!(
                    "[{}] 是默认存储，不允许删除",
                    storage.name
                )));
            }
        }
        if self.repo.storage_has_files(&ids).await? {
            return Err(AppError::bad_request("所选存储存在文件关联，不允许删除"));
        }
        self.repo.delete_storages(&ids).await
    }

    pub async fn update_status(
        &self,
        user_id: i64,
        id: i64,
        command: StorageStatusCommand,
    ) -> Result<(), AppError> {
        let status = normalize_status(command.status)?;
        let existing = self.repo.get_storage(id).await?.ok_or(AppError::NotFound)?;
        if existing.is_default && status != 1 {
            return Err(AppError::bad_request("默认存储不允许禁用"));
        }
        self.repo
            .update_storage_status(id, status, user_id, Utc::now().naive_utc())
            .await
    }

    pub async fn set_default(&self, id: i64) -> Result<(), AppError> {
        let storage = self.repo.get_storage(id).await?.ok_or(AppError::NotFound)?;
        if storage.status != 1 {
            return Err(AppError::bad_request("只能将启用的存储设为默认存储"));
        }
        self.repo.set_default_storage(id).await
    }
}

impl From<StorageRecord> for StorageResp {
    fn from(record: StorageRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            code: record.code,
            storage_type: record.storage_type,
            access_key: record.access_key,
            secret_key: record.secret_key,
            endpoint: record.endpoint,
            region: record.region,
            bucket_name: record.bucket_name,
            domain: record.domain,
            description: record.description,
            is_default: record.is_default,
            sort: record.sort,
            status: record.status,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
        }
    }
}

fn storage_record<'a>(id: i64, user_id: i64, command: &'a StorageCommand) -> StorageSaveRecord<'a> {
    StorageSaveRecord {
        id,
        name: &command.name,
        code: &command.code,
        storage_type: command.storage_type,
        access_key: non_empty(&command.access_key),
        secret_key: non_empty(&command.secret_key),
        endpoint: non_empty(&command.endpoint),
        region: non_empty(&command.region),
        bucket_name: &command.bucket_name,
        domain: non_empty(&command.domain),
        description: non_empty(&command.description),
        is_default: command.is_default,
        sort: command.sort,
        status: command.status,
        user_id,
        now: Utc::now().naive_utc(),
    }
}

pub fn normalize_storage_command(mut command: StorageCommand) -> Result<StorageCommand, AppError> {
    command.name = command.name.trim().to_owned();
    command.code = command.code.trim().to_owned();
    command.access_key = command.access_key.trim().to_owned();
    command.secret_key = command.secret_key.trim().to_owned();
    command.endpoint = command.endpoint.trim().to_owned();
    command.region = command.region.trim().to_owned();
    command.bucket_name = command.bucket_name.trim().to_owned();
    command.domain = command.domain.trim().to_owned();
    command.description = command.description.trim().to_owned();
    if command.name.is_empty() {
        return Err(AppError::bad_request("名称不能为空"));
    }
    if command.code.is_empty() {
        return Err(AppError::bad_request("编码不能为空"));
    }
    if command.storage_type == 0 {
        command.storage_type = 1;
    }
    if command.storage_type != 1 && command.storage_type != 2 {
        return Err(AppError::bad_request("存储类型不正确"));
    }
    if command.bucket_name.is_empty() {
        return Err(AppError::bad_request("存储桶不能为空"));
    }
    if command.storage_type == 2 && command.endpoint.is_empty() {
        return Err(AppError::bad_request("对象存储 Endpoint 不能为空"));
    }
    if command.status == 0 {
        command.status = 1;
    }
    command.status = normalize_status(command.status)?;
    if command.is_default && command.status != 1 {
        return Err(AppError::bad_request("默认存储必须启用"));
    }
    if command.sort <= 0 {
        command.sort = 999;
    }
    ensure_max_chars("名称", &command.name, 100)?;
    ensure_max_chars("编码", &command.code, 30)?;
    ensure_max_chars("存储桶", &command.bucket_name, 255)?;
    ensure_max_chars("域名", &command.domain, 255)?;
    ensure_max_chars("描述", &command.description, 200)?;
    Ok(command)
}

fn normalize_status(status: i16) -> Result<i16, AppError> {
    if status == 1 || status == 2 {
        Ok(status)
    } else {
        Err(AppError::bad_request("状态不正确"))
    }
}

async fn ensure_unique_code(
    repo: &SystemMiscRepository,
    code: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if repo.storage_code_exists(code, exclude_id).await? {
        return Err(AppError::bad_request(format!("保存失败，[{code}] 已存在")));
    }
    Ok(())
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_response_uses_vue_field_names() {
        let value = serde_json::to_value(StorageResp {
            id: 1,
            name: "本地".to_owned(),
            code: "local".to_owned(),
            storage_type: 1,
            access_key: String::new(),
            secret_key: String::new(),
            endpoint: String::new(),
            region: String::new(),
            bucket_name: "./data/file/".to_owned(),
            domain: "/file/".to_owned(),
            description: String::new(),
            is_default: true,
            sort: 1,
            status: 1,
            create_user_string: "admin".to_owned(),
            create_time: "2026-05-29 10:00:00".to_owned(),
            update_user_string: String::new(),
            update_time: String::new(),
        })
        .unwrap();

        assert_eq!(value["type"], 1);
        assert_eq!(value["bucketName"], "./data/file/");
        assert_eq!(value["isDefault"], true);
        assert_eq!(value["createUserString"], "admin");
    }

    #[test]
    fn storage_command_requires_enabled_default_storage() {
        let err = normalize_storage_command(StorageCommand {
            name: "本地".to_owned(),
            code: "local".to_owned(),
            storage_type: 1,
            access_key: String::new(),
            secret_key: String::new(),
            endpoint: String::new(),
            region: String::new(),
            bucket_name: "./data/file/".to_owned(),
            domain: "/file/".to_owned(),
            description: String::new(),
            is_default: true,
            sort: 1,
            status: 2,
        })
        .unwrap_err();

        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
