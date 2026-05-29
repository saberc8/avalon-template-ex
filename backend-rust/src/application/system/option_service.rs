use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::PgPool;

use crate::{
    application::system::ensure_max_chars,
    infrastructure::persistence::system_misc_repositories::{
        OptionRecord, OptionUpdateRecord, SystemMiscRepository,
    },
    shared::error::AppError,
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionQuery {
    #[serde(default, alias = "code[]", deserialize_with = "deserialize_string_vec")]
    pub code: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionResetCommand {
    #[serde(default, alias = "code[]", deserialize_with = "deserialize_string_vec")]
    pub code: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionUpdateItem {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelValueResp {
    pub label: String,
    pub value: String,
    pub extra: String,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct OptionService {
    repo: SystemMiscRepository,
}

impl OptionService {
    pub fn new(db: PgPool) -> Self {
        Self {
            repo: SystemMiscRepository::new(db),
        }
    }

    pub async fn list(&self, query: OptionQuery) -> Result<Vec<OptionResp>, AppError> {
        let codes = normalize_codes(query.code)?;
        let category = query
            .category
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        Ok(self
            .repo
            .list_options(&codes, category)
            .await?
            .into_iter()
            .map(OptionResp::from)
            .collect())
    }

    pub async fn update(&self, user_id: i64, items: Vec<OptionUpdateItem>) -> Result<(), AppError> {
        if items.is_empty() {
            return Err(AppError::bad_request("配置项不能为空"));
        }

        let mut records = Vec::with_capacity(items.len());
        for mut item in items {
            item.code = item.code.trim().to_owned();
            if item.id <= 0 {
                return Err(AppError::bad_request("配置 ID 不能为空"));
            }
            if item.code.is_empty() {
                return Err(AppError::bad_request("配置编码不能为空"));
            }
            ensure_max_chars("配置编码", &item.code, 100)?;
            records.push(OptionUpdateRecord {
                id: item.id,
                code: item.code,
                value: option_value_to_string(item.value),
            });
        }

        self.repo
            .update_options(&records, user_id, Utc::now().naive_utc())
            .await
    }

    pub async fn reset(&self, user_id: i64, command: OptionResetCommand) -> Result<(), AppError> {
        let codes = normalize_codes(command.code)?;
        let category = command
            .category
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        if codes.is_empty() && category.is_none() {
            return Err(AppError::bad_request("配置编码或分类不能为空"));
        }

        self.repo
            .reset_options(&codes, category, user_id, Utc::now().naive_utc())
            .await
    }

    pub async fn site_label_values(&self) -> Result<Vec<LabelValueResp>, AppError> {
        Ok(self
            .repo
            .list_options(&[], Some("SITE"))
            .await?
            .into_iter()
            .map(|record| LabelValueResp {
                label: record.code,
                value: record.value,
                extra: record.name,
                disabled: false,
            })
            .collect())
    }
}

impl From<OptionRecord> for OptionResp {
    fn from(record: OptionRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            code: record.code,
            value: record.value,
            description: record.description,
        }
    }
}

pub fn option_value_to_string(value: Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value,
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn normalize_codes(codes: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut values = Vec::new();
    for code in codes {
        for item in code.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            ensure_max_chars("配置编码", item, 100)?;
            values.push(item.to_owned());
        }
    }
    values.sort();
    values.dedup();
    Ok(values)
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn option_update_value_accepts_vue_form_scalars() {
        assert_eq!(option_value_to_string(json!("ContiNew")), "ContiNew");
        assert_eq!(option_value_to_string(json!(5)), "5");
        assert_eq!(option_value_to_string(json!(true)), "true");
        assert_eq!(option_value_to_string(Value::Null), "");
    }

    #[test]
    fn option_response_uses_vue_field_names() {
        let value = serde_json::to_value(OptionResp {
            id: 1,
            name: "系统名称".to_owned(),
            code: "SITE_TITLE".to_owned(),
            value: "Avalon".to_owned(),
            description: "标题".to_owned(),
        })
        .unwrap();

        assert_eq!(value["code"], "SITE_TITLE");
        assert_eq!(value["value"], "Avalon");
    }
}
