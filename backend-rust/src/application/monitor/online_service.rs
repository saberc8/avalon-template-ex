use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::{
        data_scope::resolver::{resolve_data_scope, DataScopeContext},
        system::format_datetime,
    },
    domain::{
        auth::model::{CurrentUser, UserAccount},
        data_scope::model::{DataPermissionTarget, DataScopeFilter},
    },
    infrastructure::persistence::{
        dept_repository::DeptRepository,
        online_repository::{
            new_online_id, OnlineListFilter, OnlineRecord, OnlineRepository, OnlineSaveRecord,
        },
    },
    shared::{
        error::AppError,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineUserQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::application::monitor::online_service::deserialize_string_vec"
    )]
    pub login_time: Vec<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineUserResp {
    pub id: i64,
    pub token: String,
    pub username: String,
    pub nickname: String,
    pub client_type: String,
    pub client_id: String,
    pub ip: String,
    pub address: String,
    pub browser: String,
    pub os: String,
    pub login_time: String,
    pub last_active_time: String,
}

#[derive(Debug, Clone)]
pub struct OnlineLoginCommand {
    pub token: String,
    pub client_type: String,
    pub client_id: String,
    pub ip: String,
    pub browser: String,
    pub os: String,
}

#[derive(Debug, Clone)]
pub struct OnlineService {
    online: OnlineRepository,
    depts: DeptRepository,
}

impl OnlineService {
    pub fn new(db: PgPool) -> Self {
        Self {
            online: OnlineRepository::new(db.clone()),
            depts: DeptRepository::new(db),
        }
    }

    pub async fn page(
        &self,
        current_user: &CurrentUser,
        query: OnlineUserQuery,
    ) -> Result<PageResult<OnlineUserResp>, AppError> {
        let normalized = NormalizedOnlineQuery::from_query(query)?;
        let page = PageQuery {
            page: normalized.page,
            size: normalized.size,
        }
        .normalized();
        let data_scope = self.resolve_data_scope(current_user).await?;
        let filter = normalized.to_filter(&data_scope, page.limit(), page.offset());
        let total = self.online.count(&filter).await?;
        let list = self
            .online
            .list(&filter)
            .await?
            .into_iter()
            .map(OnlineUserResp::from)
            .collect();

        Ok(PageResult::new(list, total))
    }

    pub async fn save_login(
        &self,
        user: &UserAccount,
        command: OnlineLoginCommand,
    ) -> Result<(), AppError> {
        let token = command.token.trim();
        if token.is_empty() {
            return Ok(());
        }
        let client_type = if command.client_type.trim().is_empty() {
            "PC"
        } else {
            command.client_type.trim()
        };
        let client_id = if command.client_id.trim().is_empty() {
            "default"
        } else {
            command.client_id.trim()
        };
        self.online
            .save_login(&OnlineSaveRecord {
                id: new_online_id(),
                token,
                user_id: user.id,
                username: &user.username,
                nickname: &user.nickname,
                client_type,
                client_id,
                ip: command.ip.trim(),
                address: "",
                browser: command.browser.trim(),
                os: command.os.trim(),
                now: Utc::now().naive_utc(),
            })
            .await
    }

    pub async fn kickout(&self, token: String) -> Result<(), AppError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(AppError::bad_request("Token 不能为空"));
        }
        self.online.delete_by_token(token).await
    }

    async fn resolve_data_scope(
        &self,
        current_user: &CurrentUser,
    ) -> Result<DataScopeFilter, AppError> {
        let role_ids = current_user
            .roles
            .iter()
            .map(|role| role.id)
            .collect::<Vec<_>>();
        let context = DataScopeContext {
            dept_tree: self.depts.enabled_dept_tree().await?,
            role_dept_scope: self.depts.role_dept_scope(&role_ids).await?,
        };

        resolve_data_scope(
            current_user,
            &DataPermissionTarget {
                dept_column: Some("cu.dept_id"),
                user_column: Some("o.user_id"),
            },
            &context,
        )
    }
}

impl From<OnlineRecord> for OnlineUserResp {
    fn from(record: OnlineRecord) -> Self {
        Self {
            id: record.id,
            token: record.token,
            username: record.username,
            nickname: record.nickname,
            client_type: record.client_type,
            client_id: record.client_id,
            ip: record.ip,
            address: record.address,
            browser: record.browser,
            os: record.os,
            login_time: format_datetime(record.login_time),
            last_active_time: format_datetime(record.last_active_time),
        }
    }
}

#[derive(Debug, Clone)]
struct NormalizedOnlineQuery {
    page: u64,
    size: u64,
    nickname: Option<String>,
    login_time_start: Option<NaiveDateTime>,
    login_time_end: Option<NaiveDateTime>,
    order_by: String,
}

impl NormalizedOnlineQuery {
    fn from_query(query: OnlineUserQuery) -> Result<Self, AppError> {
        let (login_time_start, login_time_end) = parse_time_range(&query.login_time)?;
        Ok(Self {
            page: query.page,
            size: query.size,
            nickname: query
                .nickname
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            login_time_start,
            login_time_end,
            order_by: online_order_sql(&query.sort),
        })
    }

    fn to_filter<'a>(
        &'a self,
        data_scope: &'a DataScopeFilter,
        limit: i64,
        offset: i64,
    ) -> OnlineListFilter<'a> {
        OnlineListFilter {
            nickname: self.nickname.as_deref(),
            login_time_start: self.login_time_start,
            login_time_end: self.login_time_end,
            data_scope,
            order_by: &self.order_by,
            limit,
            offset,
        }
    }
}

pub fn online_order_sql(sort: &[String]) -> String {
    let mut clauses = Vec::new();
    for item in sort {
        let parts = item.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 2 {
            continue;
        }
        let field = parts[0].trim_start_matches("t1.").trim_start_matches("o.");
        let column = match field {
            "id" => "o.id",
            "loginTime" | "login_time" | "createTime" | "create_time" => "o.login_time",
            "lastActiveTime" | "last_active_time" => "o.last_active_time",
            _ => continue,
        };
        let direction = match parts[1].to_ascii_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => continue,
        };
        clauses.push(format!("{column} {direction}"));
    }
    if clauses.is_empty() {
        "o.login_time DESC, o.id DESC".to_owned()
    } else {
        if !clauses.iter().any(|clause| clause.starts_with("o.id ")) {
            clauses.push("o.id DESC".to_owned());
        }
        clauses.join(", ")
    }
}

fn parse_time_range(
    values: &[String],
) -> Result<(Option<NaiveDateTime>, Option<NaiveDateTime>), AppError> {
    if values.is_empty() {
        return Ok((None, None));
    }
    let start = parse_datetime(values.first().map(String::as_str).unwrap_or_default())?;
    let end = parse_datetime(values.get(1).map(String::as_str).unwrap_or_default())?;
    Ok((start, end))
}

fn parse_datetime(value: &str) -> Result<Option<NaiveDateTime>, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if let Ok(value) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(Some(value));
    }
    if let Ok(value) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(value.and_hms_opt(0, 0, 0));
    }
    Err(AppError::bad_request("时间格式不正确"))
}

pub(crate) fn deserialize_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::String(value) => vec![value],
        serde_json::Value::Array(values) => values
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect(),
        _ => Vec::new(),
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
    fn online_response_uses_vue_field_names() {
        let value = serde_json::to_value(OnlineUserResp {
            id: 1,
            token: "token".to_owned(),
            username: "admin".to_owned(),
            nickname: "系统管理员".to_owned(),
            client_type: "PC".to_owned(),
            client_id: "client".to_owned(),
            ip: "127.0.0.1".to_owned(),
            address: String::new(),
            browser: "Chrome".to_owned(),
            os: "macOS".to_owned(),
            login_time: "2026-05-29 10:00:00".to_owned(),
            last_active_time: "2026-05-29 10:01:00".to_owned(),
        })
        .unwrap();

        assert_eq!(value["clientType"], "PC");
        assert_eq!(value["loginTime"], "2026-05-29 10:00:00");
        assert_eq!(value["lastActiveTime"], "2026-05-29 10:01:00");
    }

    #[test]
    fn online_sort_sql_uses_vue_create_time_alias() {
        assert_eq!(
            online_order_sql(&["createTime,desc".to_owned()]),
            "o.login_time DESC, o.id DESC"
        );
    }
}
