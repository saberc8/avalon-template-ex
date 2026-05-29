use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::{
        data_scope::resolver::{resolve_data_scope, DataScopeContext},
        system::{ensure_max_chars, format_datetime},
    },
    domain::{
        auth::model::CurrentUser,
        data_scope::model::{DataPermissionTarget, DataScopeFilter},
    },
    infrastructure::persistence::{
        dept_repository::DeptRepository,
        log_repository::{new_log_id, LogCreateRecord, LogListFilter, LogRecord, LogRepository},
    },
    shared::{
        error::AppError,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

pub const LOGIN_LOG_TYPE: i16 = 1;
pub const OPERATION_LOG_TYPE: i16 = 2;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub create_user_string: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::application::monitor::log_service::deserialize_string_vec"
    )]
    pub create_time: Vec<String>,
    #[serde(default)]
    pub status: Option<i16>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResp {
    pub id: i64,
    pub description: String,
    pub module: String,
    pub time_taken: i64,
    pub ip: String,
    pub address: String,
    pub browser: String,
    pub os: String,
    pub status: i16,
    pub error_msg: String,
    pub create_user_string: String,
    pub create_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDetailResp {
    #[serde(flatten)]
    pub log: LogResp,
    pub trace_id: String,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: String,
    pub request_body: String,
    pub status_code: i32,
    pub response_headers: String,
    pub response_body: String,
}

#[derive(Debug, Clone)]
pub struct LogService {
    logs: LogRepository,
    depts: DeptRepository,
}

impl LogService {
    pub fn new(db: PgPool) -> Self {
        Self {
            logs: LogRepository::new(db.clone()),
            depts: DeptRepository::new(db),
        }
    }

    pub async fn page(
        &self,
        current_user: &CurrentUser,
        query: LogQuery,
    ) -> Result<PageResult<LogResp>, AppError> {
        let normalized = NormalizedLogQuery::from_query(query)?;
        let page = PageQuery {
            page: normalized.page,
            size: normalized.size,
        }
        .normalized();
        let data_scope = self.resolve_data_scope(current_user).await?;
        let filter = normalized.to_filter(&data_scope, Some(page.limit()), Some(page.offset()));
        let total = self.logs.count(&filter).await?;
        let list = self
            .logs
            .list(&filter)
            .await?
            .into_iter()
            .map(LogResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn detail(&self, id: i64) -> Result<LogDetailResp, AppError> {
        self.logs
            .get(id)
            .await?
            .map(LogDetailResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn export(
        &self,
        current_user: &CurrentUser,
        mut query: LogQuery,
        log_type: i16,
    ) -> Result<Vec<LogResp>, AppError> {
        query.page = 1;
        query.size = 10_000;
        if log_type == LOGIN_LOG_TYPE {
            query.module = Some("登录".to_owned());
        }
        let normalized = NormalizedLogQuery::from_query(query)?.with_log_type(log_type);
        let data_scope = self.resolve_data_scope(current_user).await?;
        let filter = normalized.to_filter(&data_scope, Some(10_000), Some(0));
        Ok(self
            .logs
            .list(&filter)
            .await?
            .into_iter()
            .map(LogResp::from)
            .collect())
    }

    pub async fn create(&self, record: &LogCreateRecord<'_>) -> Result<(), AppError> {
        self.logs.insert(record).await
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
                user_column: Some("l.create_user"),
            },
            &context,
        )
    }
}

impl From<LogRecord> for LogResp {
    fn from(record: LogRecord) -> Self {
        Self {
            id: record.id,
            description: record.description,
            module: record.module,
            time_taken: record.time_taken,
            ip: record.ip,
            address: record.address,
            browser: record.browser,
            os: record.os,
            status: record.status,
            error_msg: record.error_msg,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
        }
    }
}

impl From<LogRecord> for LogDetailResp {
    fn from(record: LogRecord) -> Self {
        let trace_id = record.trace_id.clone();
        let request_url = record.request_url.clone();
        let request_method = record.request_method.clone();
        let request_headers = record.request_headers.clone();
        let request_body = record.request_body.clone();
        let status_code = record.status_code;
        let response_headers = record.response_headers.clone();
        let response_body = record.response_body.clone();
        Self {
            log: LogResp::from(record),
            trace_id,
            request_url,
            request_method,
            request_headers,
            request_body,
            status_code,
            response_headers,
            response_body,
        }
    }
}

#[derive(Debug, Clone)]
struct NormalizedLogQuery {
    page: u64,
    size: u64,
    description: Option<String>,
    module: Option<String>,
    ip: Option<String>,
    create_user_string: Option<String>,
    create_time_start: Option<NaiveDateTime>,
    create_time_end: Option<NaiveDateTime>,
    status: Option<i16>,
    log_type: Option<i16>,
    order_by: String,
}

impl NormalizedLogQuery {
    fn from_query(query: LogQuery) -> Result<Self, AppError> {
        let (create_time_start, create_time_end) = parse_time_range(&query.create_time)?;
        let module = trim_to_option(query.module);
        let log_type = if module.is_none() {
            Some(OPERATION_LOG_TYPE)
        } else {
            None
        };
        Ok(Self {
            page: query.page,
            size: query.size,
            description: trim_to_option(query.description),
            module,
            ip: trim_to_option(query.ip),
            create_user_string: trim_to_option(query.create_user_string),
            create_time_start,
            create_time_end,
            status: query.status.filter(|value| *value > 0),
            log_type,
            order_by: log_order_sql(&query.sort),
        })
    }

    fn with_log_type(mut self, log_type: i16) -> Self {
        self.log_type = Some(log_type);
        self
    }

    fn to_filter<'a>(
        &'a self,
        data_scope: &'a DataScopeFilter,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> LogListFilter<'a> {
        LogListFilter {
            description: self.description.as_deref(),
            module: self.module.as_deref(),
            ip: self.ip.as_deref(),
            create_user_string: self.create_user_string.as_deref(),
            status: self.status,
            log_type: self.log_type,
            create_time_start: self.create_time_start,
            create_time_end: self.create_time_end,
            data_scope,
            order_by: &self.order_by,
            limit,
            offset,
        }
    }
}

pub fn log_order_sql(sort: &[String]) -> String {
    let mut clauses = Vec::new();
    for item in sort {
        let parts = item.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 2 {
            continue;
        }
        let field = parts[0].trim_start_matches("t1.").trim_start_matches("l.");
        let column = match field {
            "id" => "l.id",
            "createTime" | "create_time" => "l.create_time",
            "timeTaken" | "time_taken" => "l.time_taken",
            "status" => "l.status",
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
        "l.create_time DESC, l.id DESC".to_owned()
    } else {
        if !clauses.iter().any(|clause| clause.starts_with("l.id ")) {
            clauses.push("l.id DESC".to_owned());
        }
        clauses.join(", ")
    }
}

pub fn log_csv(list: &[LogResp]) -> String {
    let mut csv = String::from("ID,时间,模块,描述,用户,IP,地点,浏览器,系统,状态,耗时,错误\n");
    for log in list {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            log.id,
            csv_cell(&log.create_time),
            csv_cell(&log.module),
            csv_cell(&log.description),
            csv_cell(&log.create_user_string),
            csv_cell(&log.ip),
            csv_cell(&log.address),
            csv_cell(&log.browser),
            csv_cell(&log.os),
            log.status,
            log.time_taken,
            csv_cell(&log.error_msg)
        ));
    }
    csv
}

#[derive(Debug, Clone)]
pub struct LogRecordInput<'a> {
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
    pub browser: &'a str,
    pub os: &'a str,
    pub status: i16,
    pub error_msg: &'a str,
    pub create_user: Option<i64>,
}

pub fn build_log_record(input: LogRecordInput<'_>) -> LogCreateRecord<'_> {
    LogCreateRecord {
        id: new_log_id(),
        trace_id: "",
        description: input.description,
        module: input.module,
        log_type: input.log_type,
        request_url: input.request_url,
        request_method: input.request_method,
        request_headers: input.request_headers,
        request_body: input.request_body,
        status_code: input.status_code,
        response_headers: input.response_headers,
        response_body: input.response_body,
        time_taken: input.time_taken,
        ip: input.ip,
        address: "",
        browser: input.browser,
        os: input.os,
        status: input.status,
        error_msg: input.error_msg,
        create_user: input.create_user,
        create_time: chrono::Utc::now().naive_utc(),
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

fn trim_to_option(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

pub fn validate_log_description(value: &str) -> Result<(), AppError> {
    ensure_max_chars("日志描述", value, 255)
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
    use serde_json::json;

    use super::*;

    #[test]
    fn log_response_uses_vue_field_names() {
        let value = serde_json::to_value(LogResp {
            id: 1,
            description: "登录成功".to_owned(),
            module: "登录".to_owned(),
            time_taken: 12,
            ip: "127.0.0.1".to_owned(),
            address: String::new(),
            browser: "Chrome".to_owned(),
            os: "macOS".to_owned(),
            status: 1,
            error_msg: String::new(),
            create_user_string: "admin".to_owned(),
            create_time: "2026-05-29 10:00:00".to_owned(),
        })
        .unwrap();

        assert_eq!(value["timeTaken"], 12);
        assert_eq!(value["createUserString"], "admin");
        assert_eq!(value["errorMsg"], "");
    }

    #[test]
    fn log_sort_sql_uses_whitelist() {
        assert_eq!(
            log_order_sql(&["createTime,desc".to_owned(), "timeTaken,asc".to_owned()]),
            "l.create_time DESC, l.time_taken ASC, l.id DESC"
        );
        assert_eq!(
            log_order_sql(&["createTime;drop table sys_log,desc".to_owned()]),
            "l.create_time DESC, l.id DESC"
        );
    }

    #[test]
    fn log_query_deserializes_vue_date_range() {
        let query = serde_json::from_value::<LogQuery>(json!({
            "createTime": ["2026-05-29 00:00:00", "2026-05-29 23:59:59"],
            "sort": ["createTime,desc"]
        }))
        .unwrap();

        assert_eq!(query.create_time.len(), 2);
    }
}
