use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::{
        data_scope::resolver::{resolve_data_scope, DataScopeContext},
        system::{ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none},
    },
    domain::{
        auth::model::CurrentUser,
        data_scope::model::{DataPermissionTarget, DataScopeFilter},
    },
    infrastructure::persistence::{
        dept_repository::DeptRepository,
        system_dept_repository::{
            DeptBasicRecord, DeptCreateRecord, DeptListFilter, DeptRecord, DeptUpdateRecord,
            SystemDeptRepository,
        },
    },
    shared::{error::AppError, id::next_id},
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptQuery {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptCommand {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub parent_id: i64,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub status: i16,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptResp {
    pub id: i64,
    pub name: String,
    pub sort: i32,
    pub status: i16,
    pub is_system: bool,
    pub description: String,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
    pub parent_id: i64,
    pub children: Vec<DeptResp>,
}

#[derive(Debug, Clone)]
pub struct DeptService {
    depts: SystemDeptRepository,
    scope_depts: DeptRepository,
}

impl DeptService {
    pub fn new(db: PgPool) -> Self {
        Self {
            depts: SystemDeptRepository::new(db.clone()),
            scope_depts: DeptRepository::new(db),
        }
    }

    pub async fn tree(
        &self,
        user: &CurrentUser,
        query: DeptQuery,
    ) -> Result<Vec<DeptResp>, AppError> {
        let data_scope = self.resolve_data_scope(user).await?;
        let records = self
            .depts
            .list(&DeptListFilter {
                description: query.description.as_deref(),
                status: query.status,
                data_scope: &data_scope,
            })
            .await?;

        Ok(build_dept_tree(
            records.into_iter().map(DeptResp::from).collect(),
        ))
    }

    pub async fn list_for_export(
        &self,
        user: &CurrentUser,
        query: DeptQuery,
    ) -> Result<Vec<DeptResp>, AppError> {
        let data_scope = self.resolve_data_scope(user).await?;
        let records = self
            .depts
            .list(&DeptListFilter {
                description: query.description.as_deref(),
                status: query.status,
                data_scope: &data_scope,
            })
            .await?;

        Ok(records.into_iter().map(DeptResp::from).collect())
    }

    pub async fn common_tree(&self) -> Result<Vec<DeptResp>, AppError> {
        let records = self.depts.list_all_for_common_tree().await?;

        Ok(build_dept_tree(
            records.into_iter().map(DeptResp::from).collect(),
        ))
    }

    pub async fn get(&self, id: i64) -> Result<DeptResp, AppError> {
        self.depts
            .get(id)
            .await?
            .map(DeptResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, user: &CurrentUser, command: DeptCommand) -> Result<i64, AppError> {
        let command = normalize_dept_command(command)?;
        if command.parent_id == 0 {
            return Err(AppError::bad_request("上级部门不能为空"));
        }
        let parent = self
            .depts
            .basic(command.parent_id)
            .await?
            .ok_or_else(|| AppError::bad_request("上级部门不存在"))?;
        ensure_unique_name(&self.depts, &command.name, command.parent_id, None).await?;

        let id = next_id();
        self.depts
            .create(&DeptCreateRecord {
                id,
                name: command.name,
                parent_id: command.parent_id,
                ancestors: child_ancestors(&parent),
                sort: command.sort,
                status: command.status,
                description: trim_to_none(command.description),
                user_id: user.id,
                now: Utc::now().naive_utc(),
            })
            .await?;

        Ok(id)
    }

    pub async fn update(
        &self,
        user: &CurrentUser,
        id: i64,
        command: DeptCommand,
    ) -> Result<(), AppError> {
        let command = normalize_dept_command(command)?;
        if id == command.parent_id {
            return Err(AppError::bad_request("上级部门不能选择自己"));
        }
        if command.parent_id == 0 {
            return Err(AppError::bad_request("上级部门不能为空"));
        }

        let existing = self.depts.basic(id).await?.ok_or(AppError::NotFound)?;
        let parent = self
            .depts
            .basic(command.parent_id)
            .await?
            .ok_or_else(|| AppError::bad_request("上级部门不存在"))?;
        ensure_system_dept_update_allowed(&existing, &command)?;
        ensure_unique_name(&self.depts, &command.name, command.parent_id, Some(id)).await?;

        self.depts
            .update(&DeptUpdateRecord {
                id,
                name: command.name,
                parent_id: command.parent_id,
                ancestors: child_ancestors(&parent),
                sort: command.sort,
                status: command.status,
                description: trim_to_none(command.description),
                user_id: user.id,
                now: Utc::now().naive_utc(),
            })
            .await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalize_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        if let Some(name) = self.depts.first_system_name(&ids).await? {
            return Err(AppError::bad_request(format!(
                "所选部门 [{name}] 是系统内置部门，不允许删除"
            )));
        }
        if self.depts.has_children(&ids).await? {
            return Err(AppError::bad_request("所选部门存在下级部门，不允许删除"));
        }
        if self.depts.has_users(&ids).await? {
            return Err(AppError::bad_request(
                "所选部门存在用户关联，请解除关联后重试",
            ));
        }

        self.depts.delete_many(&ids).await
    }

    async fn resolve_data_scope(&self, user: &CurrentUser) -> Result<DataScopeFilter, AppError> {
        let role_ids = user.roles.iter().map(|role| role.id).collect::<Vec<_>>();
        let context = DataScopeContext {
            dept_tree: self.scope_depts.enabled_dept_tree().await?,
            role_dept_scope: self.scope_depts.role_dept_scope(&role_ids).await?,
        };
        let target = DataPermissionTarget {
            dept_column: Some("d.id"),
            user_column: Some("d.create_user"),
        };

        resolve_data_scope(user, &target, &context)
    }
}

pub fn build_dept_tree(records: Vec<DeptResp>) -> Vec<DeptResp> {
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
        sort_dept_ids(ids, &by_id);
    }

    let mut root_ids = records
        .iter()
        .filter(|record| record.parent_id == 0 || !by_id.contains_key(&record.parent_id))
        .map(|record| record.id)
        .collect::<Vec<_>>();
    sort_dept_ids(&mut root_ids, &by_id);

    root_ids
        .into_iter()
        .map(|id| build_dept_node(id, &by_id, &child_ids))
        .collect()
}

impl From<DeptRecord> for DeptResp {
    fn from(record: DeptRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            sort: record.sort,
            status: record.status,
            is_system: record.is_system,
            description: record.description,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
            parent_id: record.parent_id,
            children: vec![],
        }
    }
}

fn build_dept_node(
    id: i64,
    by_id: &HashMap<i64, DeptResp>,
    child_ids: &HashMap<i64, Vec<i64>>,
) -> DeptResp {
    let mut node = by_id
        .get(&id)
        .expect("department tree id must exist")
        .clone();
    node.children = child_ids
        .get(&id)
        .map(|ids| {
            ids.iter()
                .map(|child_id| build_dept_node(*child_id, by_id, child_ids))
                .collect()
        })
        .unwrap_or_default();
    node
}

fn sort_dept_ids(ids: &mut [i64], by_id: &HashMap<i64, DeptResp>) {
    ids.sort_by(|left, right| {
        let left = by_id.get(left).expect("department sort id must exist");
        let right = by_id.get(right).expect("department sort id must exist");
        left.sort.cmp(&right.sort).then(left.id.cmp(&right.id))
    });
}

fn normalize_dept_command(mut command: DeptCommand) -> Result<DeptCommand, AppError> {
    command.name = command.name.trim().to_owned();
    command.description = command.description.trim().to_owned();
    if command.name.is_empty() {
        return Err(AppError::bad_request("名称不能为空"));
    }
    ensure_max_chars("名称", &command.name, 30)?;
    ensure_max_chars("描述", &command.description, 200)?;
    if command.sort <= 0 {
        command.sort = 1;
    }
    if command.status == 0 {
        command.status = 1;
    }
    Ok(command)
}

async fn ensure_unique_name(
    repository: &SystemDeptRepository,
    name: &str,
    parent_id: i64,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if repository.name_exists(name, parent_id, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，当前上级下已存在 [{name}]"
        )));
    }
    Ok(())
}

fn ensure_system_dept_update_allowed(
    existing: &DeptBasicRecord,
    command: &DeptCommand,
) -> Result<(), AppError> {
    if existing.is_system && command.status == 2 {
        return Err(AppError::bad_request(format!(
            "[{}] 是系统内置部门，不允许禁用",
            existing.name
        )));
    }
    if existing.is_system && command.parent_id != existing.parent_id {
        return Err(AppError::bad_request(format!(
            "[{}] 是系统内置部门，不允许变更上级部门",
            existing.name
        )));
    }
    Ok(())
}

fn child_ancestors(parent: &DeptBasicRecord) -> String {
    if parent.ancestors.trim().is_empty() {
        parent.id.to_string()
    } else {
        format!("{},{}", parent.ancestors, parent.id)
    }
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
    fn dept_command_rejects_name_that_exceeds_database_limit() {
        let command = DeptCommand {
            name: "x".repeat(31),
            parent_id: 1,
            sort: 1,
            status: 1,
            description: String::new(),
        };

        assert!(matches!(
            normalize_dept_command(command),
            Err(AppError::BadRequest(_))
        ));
    }
}
