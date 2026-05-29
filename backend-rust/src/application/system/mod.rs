pub mod client_service;
pub mod dept_service;
pub mod dict_service;
pub mod file_service;
pub mod menu_service;
pub mod option_service;
pub mod role_service;
pub mod storage_service;
pub mod user_service;

use chrono::NaiveDateTime;

use crate::shared::error::AppError;

pub(crate) fn format_datetime(value: NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn format_optional_datetime(value: Option<NaiveDateTime>) -> String {
    value.map(format_datetime).unwrap_or_default()
}

pub(crate) fn trim_to_none(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn ensure_max_chars(field_name: &str, value: &str, max: usize) -> Result<(), AppError> {
    if value.chars().count() > max {
        return Err(AppError::bad_request(format!(
            "{field_name}长度不能超过 {max} 个字符"
        )));
    }
    Ok(())
}
