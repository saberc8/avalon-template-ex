use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::system::{
        ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none,
    },
    infrastructure::persistence::system_misc_repositories::{
        new_id, normalized_ids, DictFilter, DictItemFilter, DictItemRecord, DictItemSaveRecord,
        DictRecord, SystemMiscRepository,
    },
    shared::{
        error::AppError,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictQuery {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<i16>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
    #[serde(default)]
    pub dict_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictCommand {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemCommand {
    #[serde(default)]
    pub dict_id: i64,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub is_system: bool,
    pub description: String,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemResp {
    pub id: i64,
    pub label: String,
    pub value: String,
    pub color: String,
    pub sort: i32,
    pub description: String,
    pub status: i16,
    pub dict_id: i64,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
}

#[derive(Debug, Clone)]
pub struct DictService {
    repo: SystemMiscRepository,
}

impl DictService {
    pub fn new(db: PgPool) -> Self {
        Self {
            repo: SystemMiscRepository::new(db),
        }
    }

    pub async fn list(&self, query: DictQuery) -> Result<Vec<DictResp>, AppError> {
        Ok(self
            .repo
            .list_dicts(&DictFilter {
                description: query.description.as_deref(),
            })
            .await?
            .into_iter()
            .map(DictResp::from)
            .collect())
    }

    pub async fn get(&self, id: i64) -> Result<DictResp, AppError> {
        self.repo
            .get_dict(id)
            .await?
            .map(DictResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user_id: i64, command: DictCommand) -> Result<i64, AppError> {
        let command = normalize_dict_command(command)?;
        ensure_unique_dict(&self.repo, &command, None).await?;
        let id = new_id();
        self.repo
            .create_dict(
                id,
                &command.name,
                &command.code,
                trim_to_none(command.description).as_deref(),
                user_id,
                Utc::now().naive_utc(),
            )
            .await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: DictCommand,
    ) -> Result<(), AppError> {
        let command = normalize_dict_command(command)?;
        let existing = self.repo.get_dict(id).await?.ok_or(AppError::NotFound)?;
        if command.code != existing.code {
            return Err(AppError::bad_request("字典编码不允许修改"));
        }
        ensure_unique_dict(&self.repo, &command, Some(id)).await?;
        self.repo
            .update_dict(
                id,
                &command.name,
                trim_to_none(command.description).as_deref(),
                user_id,
                Utc::now().naive_utc(),
            )
            .await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        if let Some(name) = self.repo.first_system_dict_name(&ids).await? {
            return Err(AppError::bad_request(format!(
                "所选字典 [{name}] 是系统内置字典，不允许删除"
            )));
        }
        self.repo.delete_dicts(&ids).await
    }

    pub async fn item_page(
        &self,
        query: DictItemQuery,
    ) -> Result<PageResult<DictItemResp>, AppError> {
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let dict_id = query
            .dict_id
            .as_deref()
            .map(parse_optional_id)
            .transpose()?;
        let filter = DictItemFilter {
            dict_id,
            description: query.description.as_deref(),
            status: query.status.filter(|status| *status > 0),
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.repo.count_dict_items(&filter).await?;
        let list = self
            .repo
            .list_dict_items(&filter)
            .await?
            .into_iter()
            .map(DictItemResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn get_item(&self, id: i64) -> Result<DictItemResp, AppError> {
        self.repo
            .get_dict_item(id)
            .await?
            .map(DictItemResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create_item(
        &self,
        user_id: i64,
        command: DictItemCommand,
    ) -> Result<i64, AppError> {
        let command = normalize_dict_item_command(command)?;
        ensure_dict_item_refs(&self.repo, &command, None).await?;
        let id = new_id();
        let color = trim_to_none(command.color.clone());
        let description = trim_to_none(command.description.clone());
        let record = DictItemSaveRecord {
            id,
            label: &command.label,
            value: &command.value,
            color: color.as_deref(),
            sort: command.sort,
            description: description.as_deref(),
            status: command.status,
            dict_id: command.dict_id,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.create_dict_item(&record).await?;
        Ok(id)
    }

    pub async fn update_item(
        &self,
        user_id: i64,
        id: i64,
        command: DictItemCommand,
    ) -> Result<(), AppError> {
        let command = normalize_dict_item_command(command)?;
        if self.repo.get_dict_item(id).await?.is_none() {
            return Err(AppError::NotFound);
        }
        ensure_dict_item_refs(&self.repo, &command, Some(id)).await?;
        let color = trim_to_none(command.color.clone());
        let description = trim_to_none(command.description.clone());
        let record = DictItemSaveRecord {
            id,
            label: &command.label,
            value: &command.value,
            color: color.as_deref(),
            sort: command.sort,
            description: description.as_deref(),
            status: command.status,
            dict_id: command.dict_id,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.update_dict_item(&record).await
    }

    pub async fn delete_items(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        self.repo.delete_dict_items(&ids).await
    }

    pub async fn clear_cache(&self, _code: String) -> Result<(), AppError> {
        Ok(())
    }
}

impl From<DictRecord> for DictResp {
    fn from(record: DictRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            code: record.code,
            is_system: record.is_system,
            description: record.description,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
        }
    }
}

impl From<DictItemRecord> for DictItemResp {
    fn from(record: DictItemRecord) -> Self {
        Self {
            id: record.id,
            label: record.label,
            value: record.value,
            color: record.color,
            sort: record.sort,
            description: record.description,
            status: record.status,
            dict_id: record.dict_id,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
        }
    }
}

fn normalize_dict_command(mut command: DictCommand) -> Result<DictCommand, AppError> {
    command.name = command.name.trim().to_owned();
    command.code = command.code.trim().to_owned();
    command.description = command.description.trim().to_owned();
    if command.name.is_empty() {
        return Err(AppError::bad_request("名称不能为空"));
    }
    if command.code.is_empty() {
        return Err(AppError::bad_request("编码不能为空"));
    }
    ensure_max_chars("名称", &command.name, 30)?;
    ensure_max_chars("编码", &command.code, 30)?;
    ensure_max_chars("描述", &command.description, 200)?;
    Ok(command)
}

fn normalize_dict_item_command(mut command: DictItemCommand) -> Result<DictItemCommand, AppError> {
    command.label = command.label.trim().to_owned();
    command.value = command.value.trim().to_owned();
    command.color = command.color.trim().to_owned();
    command.description = command.description.trim().to_owned();
    if command.dict_id <= 0 {
        return Err(AppError::bad_request("字典不能为空"));
    }
    if command.label.is_empty() {
        return Err(AppError::bad_request("标签不能为空"));
    }
    if command.value.is_empty() {
        return Err(AppError::bad_request("值不能为空"));
    }
    if command.sort <= 0 {
        command.sort = 999;
    }
    if command.status == 0 {
        command.status = 1;
    }
    if command.status != 1 && command.status != 2 {
        return Err(AppError::bad_request("状态不正确"));
    }
    ensure_max_chars("标签", &command.label, 30)?;
    ensure_max_chars("值", &command.value, 255)?;
    ensure_max_chars("颜色", &command.color, 30)?;
    ensure_max_chars("描述", &command.description, 200)?;
    Ok(command)
}

async fn ensure_unique_dict(
    repo: &SystemMiscRepository,
    command: &DictCommand,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if repo.dict_name_exists(&command.name, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.name
        )));
    }
    if repo.dict_code_exists(&command.code, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.code
        )));
    }
    Ok(())
}

async fn ensure_dict_item_refs(
    repo: &SystemMiscRepository,
    command: &DictItemCommand,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if !repo.dict_exists(command.dict_id).await? {
        return Err(AppError::bad_request("字典不存在"));
    }
    if repo
        .dict_item_value_exists(command.dict_id, &command.value, exclude_id)
        .await?
    {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.value
        )));
    }
    Ok(())
}

fn parse_optional_id(value: &str) -> Result<i64, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0);
    }
    value
        .parse::<i64>()
        .map_err(|_| AppError::bad_request("ID 参数不正确"))
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
    fn dict_response_uses_vue_field_names() {
        let value = serde_json::to_value(DictResp {
            id: 1,
            name: "用户状态".to_owned(),
            code: "user_status".to_owned(),
            is_system: true,
            description: String::new(),
            create_user_string: "admin".to_owned(),
            create_time: "2026-05-29 10:00:00".to_owned(),
            update_user_string: String::new(),
            update_time: String::new(),
        })
        .unwrap();

        assert_eq!(value["isSystem"], true);
        assert_eq!(value["createUserString"], "admin");
    }
}
