use std::collections::{HashMap, HashSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::system::{
        ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none,
    },
    infrastructure::persistence::system_menu_repository::{
        MenuListFilter, MenuRecord, MenuSaveRecord, SystemMenuRepository,
    },
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuQuery {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuCommand {
    #[serde(rename = "type", default)]
    pub menu_type: i16,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub permission: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub component: String,
    #[serde(default)]
    pub redirect: String,
    #[serde(default)]
    pub is_external: Option<bool>,
    #[serde(default)]
    pub is_cache: Option<bool>,
    #[serde(default)]
    pub is_hidden: Option<bool>,
    #[serde(default)]
    pub parent_id: i64,
    #[serde(default)]
    pub status: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuResp {
    pub id: i64,
    pub title: String,
    pub parent_id: i64,
    #[serde(rename = "type")]
    pub menu_type: i16,
    pub path: String,
    pub name: String,
    pub component: String,
    pub redirect: String,
    pub icon: String,
    pub is_external: bool,
    pub is_cache: bool,
    pub is_hidden: bool,
    pub permission: String,
    pub sort: i32,
    pub status: i16,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
    pub children: Vec<MenuResp>,
}

#[derive(Debug, Clone)]
pub struct MenuService {
    menus: SystemMenuRepository,
}

impl MenuService {
    pub fn new(db: PgPool) -> Self {
        Self {
            menus: SystemMenuRepository::new(db),
        }
    }

    pub async fn tree(&self, query: MenuQuery) -> Result<Vec<MenuResp>, AppError> {
        let records = self
            .menus
            .list(&MenuListFilter {
                title: query.title_filter().as_deref(),
                status: query.status_filter(),
                only_catalog_and_menu: false,
            })
            .await?;

        Ok(build_menu_tree(
            records.into_iter().map(MenuResp::from).collect(),
        ))
    }

    pub async fn common_tree(&self, query: MenuQuery) -> Result<Vec<MenuResp>, AppError> {
        let records = self
            .menus
            .list(&MenuListFilter {
                title: query.title_filter().as_deref(),
                status: query.status_filter(),
                only_catalog_and_menu: true,
            })
            .await?;

        Ok(build_menu_tree(
            records.into_iter().map(MenuResp::from).collect(),
        ))
    }

    pub async fn get(&self, id: i64) -> Result<MenuResp, AppError> {
        self.menus
            .get(id)
            .await?
            .map(MenuResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user_id: i64, command: MenuCommand) -> Result<i64, AppError> {
        let id = next_id();
        let record = self
            .normalize_save_record(id, user_id, command, None)
            .await?;
        self.menus.create(&record).await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: MenuCommand,
    ) -> Result<(), AppError> {
        if id == command.parent_id {
            return Err(AppError::bad_request("上级菜单不能选择自己"));
        }
        let record = self
            .normalize_save_record(id, user_id, command, Some(id))
            .await?;
        self.menus.update(&record).await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalize_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }

        let delete_ids = self.collect_descendants(&ids).await?;
        self.menus.delete_many(&delete_ids).await
    }

    pub async fn clear_cache(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn normalize_save_record(
        &self,
        id: i64,
        user_id: i64,
        command: MenuCommand,
        exclude_id: Option<i64>,
    ) -> Result<MenuSaveRecord, AppError> {
        let command = normalize_menu_command(command)?;
        if self
            .menus
            .title_exists(&command.title, command.parent_id, exclude_id)
            .await?
        {
            return Err(AppError::bad_request(format!(
                "保存失败，当前上级下已存在 [{}]",
                command.title
            )));
        }

        Ok(MenuSaveRecord {
            id,
            title: command.title,
            parent_id: command.parent_id,
            menu_type: command.menu_type,
            path: trim_to_none(command.path),
            name: trim_to_none(command.name),
            component: trim_to_none(command.component),
            redirect: trim_to_none(command.redirect),
            icon: trim_to_none(command.icon),
            is_external: command.is_external.unwrap_or(false),
            is_cache: command.is_cache.unwrap_or(false),
            is_hidden: command.is_hidden.unwrap_or(false),
            permission: trim_to_none(command.permission),
            sort: command.sort,
            status: command.status,
            user_id,
            now: Utc::now().naive_utc(),
        })
    }

    async fn collect_descendants(&self, root_ids: &[i64]) -> Result<Vec<i64>, AppError> {
        let edges = self.menus.all_edges().await?;
        let mut children_by_parent = HashMap::<i64, Vec<i64>>::new();
        for (id, parent_id) in edges {
            children_by_parent.entry(parent_id).or_default().push(id);
        }

        let mut seen = HashSet::<i64>::new();
        let mut stack = root_ids.to_vec();
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            if let Some(children) = children_by_parent.get(&id) {
                stack.extend(children.iter().copied());
            }
        }

        let mut ids = seen.into_iter().collect::<Vec<_>>();
        ids.sort_unstable();
        Ok(ids)
    }
}

impl MenuQuery {
    fn title_filter(&self) -> Option<String> {
        None
    }

    fn status_filter(&self) -> Option<i16> {
        None
    }
}

pub fn build_menu_tree(records: Vec<MenuResp>) -> Vec<MenuResp> {
    let by_id = records
        .iter()
        .cloned()
        .map(|record| (record.id, record))
        .collect::<HashMap<_, _>>();
    let mut child_ids = HashMap::<i64, Vec<i64>>::new();
    for record in &records {
        if record.parent_id != 0 && by_id.contains_key(&record.parent_id) {
            child_ids
                .entry(record.parent_id)
                .or_default()
                .push(record.id);
        }
    }

    for ids in child_ids.values_mut() {
        sort_menu_ids(ids, &by_id);
    }

    let mut root_ids = records
        .iter()
        .filter(|record| record.parent_id == 0 || !by_id.contains_key(&record.parent_id))
        .map(|record| record.id)
        .collect::<Vec<_>>();
    sort_menu_ids(&mut root_ids, &by_id);

    root_ids
        .into_iter()
        .map(|id| build_menu_node(id, &by_id, &child_ids))
        .collect()
}

impl From<MenuRecord> for MenuResp {
    fn from(record: MenuRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            parent_id: record.parent_id,
            menu_type: record.menu_type,
            path: record.path,
            name: record.name,
            component: record.component,
            redirect: record.redirect,
            icon: record.icon,
            is_external: record.is_external,
            is_cache: record.is_cache,
            is_hidden: record.is_hidden,
            permission: record.permission,
            sort: record.sort,
            status: record.status,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
            children: vec![],
        }
    }
}

fn build_menu_node(
    id: i64,
    by_id: &HashMap<i64, MenuResp>,
    child_ids: &HashMap<i64, Vec<i64>>,
) -> MenuResp {
    let mut node = by_id.get(&id).expect("menu tree id must exist").clone();
    node.children = child_ids
        .get(&id)
        .map(|ids| {
            ids.iter()
                .map(|child_id| build_menu_node(*child_id, by_id, child_ids))
                .collect()
        })
        .unwrap_or_default();
    node
}

fn sort_menu_ids(ids: &mut [i64], by_id: &HashMap<i64, MenuResp>) {
    ids.sort_by(|left, right| {
        let left = by_id.get(left).expect("menu sort id must exist");
        let right = by_id.get(right).expect("menu sort id must exist");
        left.sort.cmp(&right.sort).then(left.id.cmp(&right.id))
    });
}

fn normalize_menu_command(mut command: MenuCommand) -> Result<MenuCommand, AppError> {
    if command.menu_type == 0 {
        command.menu_type = 1;
    }
    command.title = command.title.trim().to_owned();
    command.icon = command.icon.trim().to_owned();
    command.permission = command.permission.trim().to_owned();
    command.path = command.path.trim().to_owned();
    command.name = command.name.trim().trim_start_matches('/').to_owned();
    command.component = command.component.trim().trim_start_matches('/').to_owned();
    command.redirect = command.redirect.trim().to_owned();

    if command.title.is_empty() {
        return Err(AppError::bad_request("菜单标题不能为空"));
    }
    ensure_max_chars("标题", &command.title, 30)?;
    ensure_max_chars("图标", &command.icon, 50)?;
    ensure_max_chars("权限标识", &command.permission, 100)?;
    ensure_max_chars("路由地址", &command.path, 255)?;
    ensure_max_chars("组件名称", &command.name, 50)?;
    ensure_max_chars("组件路径", &command.component, 255)?;
    ensure_max_chars("重定向地址", &command.redirect, 255)?;

    let is_external = command.is_external.unwrap_or(false);
    if is_external {
        if !(command.path.starts_with("http://") || command.path.starts_with("https://")) {
            return Err(AppError::bad_request(
                "路由地址格式不正确，请以 http:// 或 https:// 开头",
            ));
        }
    } else if command.path.starts_with("http://") || command.path.starts_with("https://") {
        return Err(AppError::bad_request("路由地址格式不正确"));
    } else if !command.path.is_empty() && !command.path.starts_with('/') {
        command.path = format!("/{}", command.path);
    }

    if command.sort <= 0 {
        command.sort = 999;
    }
    if command.status == 0 {
        command.status = 1;
    }

    Ok(command)
}

fn normalize_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut ids = ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_command_rejects_title_that_exceeds_database_limit() {
        let command = MenuCommand {
            menu_type: 2,
            icon: String::new(),
            title: "x".repeat(31),
            sort: 1,
            permission: String::new(),
            path: "/demo".to_owned(),
            name: "Demo".to_owned(),
            component: "demo/index".to_owned(),
            redirect: String::new(),
            is_external: Some(false),
            is_cache: Some(false),
            is_hidden: Some(false),
            parent_id: 0,
            status: 1,
        };

        assert!(matches!(
            normalize_menu_command(command),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn menu_tree_query_filter_is_ignored_for_vue_parity() {
        let query = MenuQuery {
            title: Some("missing".to_owned()),
            description: Some("also-missing".to_owned()),
            status: Some(2),
        };

        assert!(query.title_filter().is_none());
        assert!(query.status_filter().is_none());
    }
}
