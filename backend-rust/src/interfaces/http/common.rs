use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::{
    application::system::{
        dept_service::{DeptResp, DeptService},
        menu_service::{MenuQuery, MenuResp, MenuService},
    },
    domain::auth::model::CurrentUser,
    shared::{error::AppError, response::ApiResponse},
};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/common/tree/dept", get(dept_tree))
        .route("/common/tree/menu", get(menu_tree))
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
