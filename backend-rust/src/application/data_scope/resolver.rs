pub use crate::domain::data_scope::model::{DeptTree, RoleDeptScope};

use crate::{
    domain::{
        auth::model::CurrentUser,
        data_scope::model::{DataPermissionTarget, DataScope, DataScopeFilter},
    },
    shared::error::AppError,
};

#[derive(Debug, Clone, Default)]
pub struct DataScopeContext {
    pub dept_tree: DeptTree,
    pub role_dept_scope: RoleDeptScope,
}

pub fn resolve_data_scope(
    user: &CurrentUser,
    target: &DataPermissionTarget<'_>,
    context: &DataScopeContext,
) -> Result<DataScopeFilter, AppError> {
    target
        .validate()
        .map_err(|error| AppError::bad_request(error.to_string()))?;

    if user.roles.iter().any(|role| role.is_admin()) {
        return Ok(DataScopeFilter::unrestricted());
    }

    let mut dept_ids = Vec::new();
    let mut self_user_id = None;

    for role in &user.roles {
        let data_scope = DataScope::from_code(role.data_scope).ok_or_else(|| {
            AppError::bad_request(format!("未知的数据权限范围: {}", role.data_scope))
        })?;

        match data_scope {
            DataScope::All => return Ok(DataScopeFilter::unrestricted()),
            DataScope::DeptAndChild => {
                require_dept_column(target)?;
                dept_ids.extend(context.dept_tree.descendants_including(user.dept_id));
            }
            DataScope::Dept => {
                require_dept_column(target)?;
                dept_ids.push(user.dept_id);
            }
            DataScope::SelfOnly => {
                require_user_column(target)?;
                self_user_id = Some(user.id);
            }
            DataScope::Custom => {
                require_dept_column(target)?;
                dept_ids.extend(context.role_dept_scope.dept_ids_for_role(role.id));
            }
        }
    }

    DataScopeFilter::restricted(target, dept_ids, self_user_id)
        .map_err(|error| AppError::bad_request(error.to_string()))
}

fn require_dept_column(target: &DataPermissionTarget<'_>) -> Result<(), AppError> {
    if target.dept_column.is_some() {
        Ok(())
    } else {
        Err(AppError::bad_request("当前数据权限目标缺少部门列"))
    }
}

fn require_user_column(target: &DataPermissionTarget<'_>) -> Result<(), AppError> {
    if target.user_column.is_some() {
        Ok(())
    } else {
        Err(AppError::bad_request("当前数据权限目标缺少用户列"))
    }
}
