use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Postgres, QueryBuilder};

use crate::{
    application::system::{
        dept_service::{DeptResp, DeptService},
        menu_service::{MenuQuery, MenuResp, MenuService},
        option_service::{LabelValueResp, OptionService},
    },
    domain::auth::model::CurrentUser,
    shared::{error::AppError, response::ApiResponse},
};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/common/tree/dept", get(dept_tree))
        .route("/common/tree/menu", get(menu_tree))
        .route("/common/dict/user", get(user_dict))
        .route("/common/dict/role", get(role_dict))
        .route("/common/dict/option/site", get(site_option_dict))
        .route("/common/dict/:code", get(common_dict))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonTreeNode {
    pub key: i64,
    pub id: i64,
    pub title: String,
    pub name: String,
    pub disabled: bool,
    pub children: Vec<CommonTreeNode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserDictQuery {
    #[serde(default)]
    status: Option<i16>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoleDictQuery {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    status: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonLabelValue {
    pub label: String,
    pub value: Value,
    pub extra: String,
    pub color: String,
    pub disabled: bool,
}

#[derive(Debug, FromRow)]
struct UserDictRecord {
    id: i64,
    nickname: String,
    username: String,
    status: i16,
}

#[derive(Debug, FromRow)]
struct RoleDictRecord {
    id: i64,
    name: String,
    code: String,
    status: i16,
}

#[derive(Debug, FromRow)]
struct DictItemRecord {
    label: String,
    value: String,
    color: String,
    status: i16,
}

async fn dept_tree(
    State(state): State<AppState>,
    _current_user: CurrentUser,
) -> Result<Json<ApiResponse<Vec<CommonTreeNode>>>, AppError> {
    let service = DeptService::new(state.db);
    let tree = service.common_tree().await?;

    Ok(Json(ApiResponse::ok(
        tree.into_iter().map(CommonTreeNode::from).collect(),
    )))
}

async fn menu_tree(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    axum::extract::Query(query): axum::extract::Query<MenuQuery>,
) -> Result<Json<ApiResponse<Vec<CommonTreeNode>>>, AppError> {
    let service = MenuService::new(state.db);
    let tree = service.common_tree(query).await?;

    Ok(Json(ApiResponse::ok(
        tree.into_iter().map(CommonTreeNode::from).collect(),
    )))
}

async fn user_dict(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Query(query): Query<UserDictQuery>,
) -> Result<Json<ApiResponse<Vec<CommonLabelValue>>>, AppError> {
    let mut sql = QueryBuilder::<Postgres>::new(
        "SELECT id, nickname, username, status FROM sys_user WHERE 1 = 1",
    );
    if let Some(status) = query.status.filter(|status| *status > 0) {
        sql.push(" AND status = ").push_bind(status);
    }
    sql.push(" ORDER BY id ASC");
    let list = sql
        .build_query_as::<UserDictRecord>()
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|record| CommonLabelValue {
            label: record.nickname,
            value: json!(record.id),
            extra: record.username,
            color: String::new(),
            disabled: record.status != 1,
        })
        .collect();

    Ok(Json(ApiResponse::ok(list)))
}

async fn role_dict(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Query(query): Query<RoleDictQuery>,
) -> Result<Json<ApiResponse<Vec<CommonLabelValue>>>, AppError> {
    let mut sql =
        QueryBuilder::<Postgres>::new("SELECT id, name, code, status FROM sys_role WHERE 1 = 1");
    if let Some(name) = query
        .name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        sql.push(" AND (name ILIKE ")
            .push_bind(format!("%{name}%"))
            .push(" OR code ILIKE ")
            .push_bind(format!("%{name}%"))
            .push(")");
    }
    if let Some(status) = query.status.filter(|status| *status > 0) {
        sql.push(" AND status = ").push_bind(status);
    }
    sql.push(" ORDER BY sort ASC, id ASC");
    let list = sql
        .build_query_as::<RoleDictRecord>()
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|record| CommonLabelValue {
            label: record.name,
            value: json!(record.id),
            extra: record.code,
            color: String::new(),
            disabled: record.status != 1,
        })
        .collect();

    Ok(Json(ApiResponse::ok(list)))
}

async fn common_dict(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<Vec<CommonLabelValue>>>, AppError> {
    let list = sqlx::query_as::<_, DictItemRecord>(
        r#"
SELECT di.label,
       di.value,
       COALESCE(di.color, '') AS color,
       di.status
FROM sys_dict_item AS di
INNER JOIN sys_dict AS d ON d.id = di.dict_id
WHERE d.code = $1
ORDER BY di.sort ASC, di.id ASC;
"#,
    )
    .bind(code.trim())
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|record| CommonLabelValue {
        label: record.label,
        value: json!(record.value),
        extra: String::new(),
        color: record.color,
        disabled: record.status != 1,
    })
    .collect();

    Ok(Json(ApiResponse::ok(list)))
}

async fn site_option_dict(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<LabelValueResp>>>, AppError> {
    let service = OptionService::new(state.db);

    Ok(Json(ApiResponse::ok(service.site_label_values().await?)))
}

impl From<DeptResp> for CommonTreeNode {
    fn from(dept: DeptResp) -> Self {
        Self {
            key: dept.id,
            id: dept.id,
            title: dept.name.clone(),
            name: dept.name,
            disabled: false,
            children: dept
                .children
                .into_iter()
                .map(CommonTreeNode::from)
                .collect(),
        }
    }
}

impl From<MenuResp> for CommonTreeNode {
    fn from(menu: MenuResp) -> Self {
        Self {
            key: menu.id,
            id: menu.id,
            title: menu.title.clone(),
            name: menu.title,
            disabled: menu.status != 1,
            children: menu
                .children
                .into_iter()
                .map(CommonTreeNode::from)
                .collect(),
        }
    }
}
