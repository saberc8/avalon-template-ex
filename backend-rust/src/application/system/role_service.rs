use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    application::system::{
        ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none,
    },
    infrastructure::persistence::system_role_repository::{
        RoleCreateRecord, RoleListFilter, RolePermissionRecord, RoleRecord, RoleUpdateRecord,
        RoleUserListFilter, RoleUserRecord, SystemRoleRepository,
    },
    shared::{
        error::AppError,
        id::next_id,
        pagination::{PageQuery, PageResult},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQuery {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleUserPageQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleCommand {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub data_scope: i16,
    #[serde(default)]
    pub dept_ids: Vec<i64>,
    #[serde(default = "default_true")]
    pub dept_check_strictly: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionCommand {
    #[serde(default)]
    pub menu_ids: Vec<i64>,
    #[serde(default = "default_true")]
    pub menu_check_strictly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub sort: i32,
    pub description: String,
    pub data_scope: i16,
    pub is_system: bool,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDetailResp {
    #[serde(flatten)]
    pub role: RoleResp,
    pub menu_ids: Vec<i64>,
    pub dept_ids: Vec<i64>,
    pub menu_check_strictly: bool,
    pub dept_check_strictly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleUserResp {
    pub id: i64,
    pub role_id: i64,
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub gender: i16,
    pub status: i16,
    pub is_system: bool,
    pub description: String,
    pub dept_id: i64,
    pub dept_name: String,
    pub role_ids: Vec<i64>,
    pub role_names: Vec<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct RoleService {
    roles: SystemRoleRepository,
}

impl RoleService {
    pub fn new(db: PgPool) -> Self {
        Self {
            roles: SystemRoleRepository::new(db),
        }
    }

    pub async fn list(&self, query: RoleQuery) -> Result<Vec<RoleResp>, AppError> {
        let roles = self
            .roles
            .list(&RoleListFilter {
                description: query.description.as_deref(),
            })
            .await?
            .into_iter()
            .map(RoleResp::from)
            .collect();
        Ok(roles)
    }

    pub async fn get(&self, id: i64) -> Result<RoleDetailResp, AppError> {
        let record = self.roles.get(id).await?.ok_or(AppError::NotFound)?;
        let menu_ids = self.roles.menu_ids(id).await?;
        let dept_ids = self.roles.dept_ids(id).await?;
        Ok(RoleDetailResp {
            menu_check_strictly: record.menu_check_strictly,
            dept_check_strictly: record.dept_check_strictly,
            role: RoleResp::from(record),
            menu_ids,
            dept_ids,
        })
    }

    pub async fn create(&self, user_id: i64, command: RoleCommand) -> Result<i64, AppError> {
        let command = normalize_role_command(command)?;
        ensure_unique_name(&self.roles, &command.name, None).await?;
        ensure_unique_code(&self.roles, &command.code, None).await?;

        let id = next_id();
        self.roles
            .create(&RoleCreateRecord {
                id,
                name: command.name,
                code: command.code,
                data_scope: command.data_scope,
                description: trim_to_none(command.description),
                sort: command.sort,
                dept_check_strictly: command.dept_check_strictly,
                dept_ids: custom_dept_ids(command.data_scope, command.dept_ids),
                user_id,
                now: Utc::now().naive_utc(),
            })
            .await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: RoleCommand,
    ) -> Result<(), AppError> {
        let command = normalize_role_command(command)?;
        let existing = self.roles.get(id).await?.ok_or(AppError::NotFound)?;
        if command.code != existing.code {
            return Err(AppError::bad_request("角色编码不允许修改"));
        }
        if existing.is_system && command.data_scope != existing.data_scope {
            return Err(AppError::bad_request(format!(
                "[{}] 是系统内置角色，不允许修改角色数据权限",
                existing.name
            )));
        }
        ensure_unique_name(&self.roles, &command.name, Some(id)).await?;

        self.roles
            .update(&RoleUpdateRecord {
                id,
                name: command.name,
                code: command.code,
                data_scope: command.data_scope,
                description: trim_to_none(command.description),
                sort: command.sort,
                dept_check_strictly: command.dept_check_strictly,
                dept_ids: custom_dept_ids(command.data_scope, command.dept_ids),
                user_id,
                now: Utc::now().naive_utc(),
            })
            .await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalize_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        if let Some(name) = self.roles.first_system_name(&ids).await? {
            return Err(AppError::bad_request(format!(
                "所选角色 [{name}] 是系统内置角色，不允许删除"
            )));
        }
        if self.roles.has_users(&ids).await? {
            return Err(AppError::bad_request(
                "所选角色存在用户关联，请解除关联后重试",
            ));
        }

        self.roles.delete_many(&ids).await
    }

    pub async fn update_permission(
        &self,
        user_id: i64,
        id: i64,
        command: RolePermissionCommand,
    ) -> Result<(), AppError> {
        self.roles
            .update_permission(&RolePermissionRecord {
                id,
                menu_ids: normalize_ids(command.menu_ids),
                menu_check_strictly: command.menu_check_strictly,
                user_id,
                now: Utc::now().naive_utc(),
            })
            .await
    }

    pub async fn user_ids(&self, id: i64) -> Result<Vec<i64>, AppError> {
        if self.roles.get(id).await?.is_none() {
            return Err(AppError::NotFound);
        }
        self.roles.user_ids(id).await
    }

    pub async fn user_page(
        &self,
        role_id: i64,
        query: RoleUserPageQuery,
    ) -> Result<PageResult<RoleUserResp>, AppError> {
        ensure_positive_role_id(role_id)?;
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let description = query
            .description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let filter = RoleUserListFilter {
            role_id,
            description,
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.roles.count_role_users(&filter).await?;
        let list = self
            .roles
            .list_role_users(&filter)
            .await?
            .into_iter()
            .map(RoleUserResp::from)
            .collect();

        Ok(PageResult::new(list, total))
    }

    pub async fn assign_users(&self, role_id: i64, user_ids: Vec<i64>) -> Result<(), AppError> {
        ensure_positive_role_id(role_id)?;
        let role = self.roles.get(role_id).await?.ok_or(AppError::NotFound)?;
        ensure_role_can_be_assigned(&role)?;
        let user_ids = normalize_ids(user_ids);
        if user_ids.is_empty() {
            return Err(AppError::bad_request("用户ID列表不能为空"));
        }

        self.roles.assign_users(role_id, &user_ids).await
    }

    pub async fn unassign_user_roles(&self, user_role_ids: Vec<i64>) -> Result<(), AppError> {
        let user_role_ids = normalize_ids(user_role_ids);
        if user_role_ids.is_empty() {
            return Err(AppError::bad_request("用户角色ID列表不能为空"));
        }
        ensure_admin_user_role_can_be_unassigned(
            self.roles
                .has_protected_admin_user_role(&user_role_ids)
                .await?,
        )?;

        self.roles.unassign_user_roles(&user_role_ids).await
    }
}

impl From<RoleRecord> for RoleResp {
    fn from(record: RoleRecord) -> Self {
        let disabled = record.is_system && record.code == "admin";
        Self {
            id: record.id,
            name: record.name,
            code: record.code,
            sort: record.sort,
            description: record.description,
            data_scope: record.data_scope,
            is_system: record.is_system,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
            disabled,
        }
    }
}

impl From<RoleUserRecord> for RoleUserResp {
    fn from(record: RoleUserRecord) -> Self {
        let disabled = record.is_system && record.role_id == 1;
        Self {
            id: record.id,
            role_id: record.role_id,
            user_id: record.user_id,
            username: record.username,
            nickname: record.nickname,
            gender: record.gender,
            status: record.status,
            is_system: record.is_system,
            description: record.description,
            dept_id: record.dept_id,
            dept_name: record.dept_name,
            role_ids: record.role_ids,
            role_names: record.role_names,
            disabled,
        }
    }
}

fn ensure_positive_role_id(role_id: i64) -> Result<(), AppError> {
    if role_id <= 0 {
        return Err(AppError::bad_request("ID 参数不正确"));
    }
    Ok(())
}

fn ensure_role_can_be_assigned(role: &RoleRecord) -> Result<(), AppError> {
    if role.id <= 0 {
        return Err(AppError::bad_request("ID 参数不正确"));
    }
    if role.id == 1 || role.code == "admin" {
        return Err(AppError::bad_request("系统管理员角色不允许分配"));
    }
    Ok(())
}

fn ensure_admin_user_role_can_be_unassigned(has_protected_role: bool) -> Result<(), AppError> {
    if has_protected_role {
        return Err(AppError::bad_request("系统管理员角色关联不允许取消分配"));
    }
    Ok(())
}

fn normalize_role_command(mut command: RoleCommand) -> Result<RoleCommand, AppError> {
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
    if command.sort <= 0 {
        command.sort = 999;
    }
    if command.data_scope == 0 {
        command.data_scope = 4;
    }
    if !(1..=5).contains(&command.data_scope) {
        return Err(AppError::bad_request("数据权限范围不正确"));
    }
    command.dept_ids = normalize_ids(command.dept_ids);
    Ok(command)
}

async fn ensure_unique_name(
    repository: &SystemRoleRepository,
    name: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if repository.name_exists(name, exclude_id).await? {
        return Err(AppError::bad_request(format!("保存失败，[{name}] 已存在")));
    }
    Ok(())
}

async fn ensure_unique_code(
    repository: &SystemRoleRepository,
    code: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if repository.code_exists(code, exclude_id).await? {
        return Err(AppError::bad_request(format!("保存失败，[{code}] 已存在")));
    }
    Ok(())
}

fn custom_dept_ids(data_scope: i16, dept_ids: Vec<i64>) -> Vec<i64> {
    if data_scope == 5 {
        dept_ids
    } else {
        vec![]
    }
}

fn normalize_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut ids = ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn default_true() -> bool {
    true
}

fn default_page() -> u64 {
    crate::shared::pagination::DEFAULT_PAGE
}

fn default_size() -> u64 {
    crate::shared::pagination::DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn role_command_rejects_name_that_exceeds_database_limit() {
        let command = RoleCommand {
            name: "x".repeat(31),
            code: "role_code".to_owned(),
            sort: 1,
            description: String::new(),
            data_scope: 4,
            dept_ids: vec![],
            dept_check_strictly: true,
        };

        assert!(matches!(
            normalize_role_command(command),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn role_resp_disables_system_admin_role() {
        let resp = RoleResp::from(role_record(1, "系统管理员", "admin", true));

        assert!(resp.disabled);
    }

    #[test]
    fn role_resp_keeps_system_general_role_enabled() {
        let resp = RoleResp::from(role_record(2, "普通用户", "general", true));

        assert!(!resp.disabled);
    }

    #[test]
    fn admin_role_cannot_be_assigned_to_users() {
        let result = ensure_role_can_be_assigned(&role_record(1, "系统管理员", "admin", true));

        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn general_system_role_can_be_assigned_to_users() {
        let result = ensure_role_can_be_assigned(&role_record(2, "普通用户", "general", true));

        assert!(result.is_ok());
    }

    #[test]
    fn protected_admin_user_role_cannot_be_unassigned() {
        let err = ensure_admin_user_role_can_be_unassigned(true).unwrap_err();

        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn role_user_resp_disables_admin_system_user_association() {
        let resp = RoleUserResp::from(RoleUserRecord {
            id: 10,
            role_id: 1,
            user_id: 1,
            username: "admin".to_owned(),
            nickname: "系统管理员".to_owned(),
            gender: 1,
            status: 1,
            is_system: true,
            description: String::new(),
            dept_id: 1,
            dept_name: "总部".to_owned(),
            role_ids: vec![1],
            role_names: vec!["系统管理员".to_owned()],
        });

        assert!(resp.disabled);
    }

    fn role_record(id: i64, name: &str, code: &str, is_system: bool) -> RoleRecord {
        RoleRecord {
            id,
            name: name.to_owned(),
            code: code.to_owned(),
            sort: i32::try_from(id).unwrap_or(999),
            description: String::new(),
            data_scope: 1,
            is_system,
            menu_check_strictly: true,
            dept_check_strictly: true,
            create_time: NaiveDate::from_ymd_opt(2026, 5, 29)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            create_user_string: "admin".to_owned(),
            update_time: None,
            update_user_string: String::new(),
        }
    }
}
