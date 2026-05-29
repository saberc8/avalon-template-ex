use backend_rust::{
    application::data_scope::resolver::{
        resolve_data_scope, DataScopeContext, DeptTree, RoleDeptScope,
    },
    domain::{
        auth::model::{CurrentUser, RoleContext},
        data_scope::model::DataPermissionTarget,
    },
    shared::error::AppError,
};
use sqlx::{Postgres, QueryBuilder};

fn sample_target() -> DataPermissionTarget<'static> {
    DataPermissionTarget {
        dept_column: Some("dept_id"),
        user_column: Some("create_user"),
    }
}

fn sample_user_with_scope(data_scope: i16) -> CurrentUser {
    sample_user_with_dept_and_scope(10, data_scope)
}

fn sample_user_with_dept_and_scope(dept_id: i64, data_scope: i16) -> CurrentUser {
    CurrentUser {
        id: 100,
        username: "tester".to_string(),
        dept_id,
        roles: vec![RoleContext {
            id: 20,
            name: "测试角色".to_string(),
            code: "test_role".to_string(),
            data_scope,
        }],
        permissions: vec![],
    }
}

fn sample_context() -> DataScopeContext {
    DataScopeContext {
        dept_tree: DeptTree::default(),
        role_dept_scope: RoleDeptScope::default(),
    }
}

fn context_with_dept_edges(edges: impl IntoIterator<Item = (i64, i64)>) -> DataScopeContext {
    DataScopeContext {
        dept_tree: DeptTree::from_edges(edges),
        role_dept_scope: RoleDeptScope::default(),
    }
}

fn context_with_role_depts(pairs: impl IntoIterator<Item = (i64, i64)>) -> DataScopeContext {
    DataScopeContext {
        dept_tree: DeptTree::default(),
        role_dept_scope: RoleDeptScope::from_pairs(pairs),
    }
}

#[test]
fn data_scope_all_data_role_returns_unrestricted_filter() {
    let user = sample_user_with_scope(1);

    let filter = resolve_data_scope(&user, &sample_target(), &sample_context()).unwrap();

    assert!(filter.is_unrestricted());
}

#[test]
fn data_scope_unrestricted_filter_does_not_append_sql() {
    let user = sample_user_with_scope(1);
    let filter = resolve_data_scope(&user, &sample_target(), &sample_context()).unwrap();
    let mut query = QueryBuilder::<Postgres>::new("SELECT * FROM sys_user");

    filter.append_where_clause(&mut query);

    assert_eq!(filter.to_debug_sql(), "");
    assert_eq!(query.sql(), "SELECT * FROM sys_user");
}

#[test]
fn data_scope_self_scope_uses_create_user_column() {
    let user = sample_user_with_scope(4);

    let filter = resolve_data_scope(&user, &sample_target(), &sample_context()).unwrap();

    assert_eq!(filter.to_debug_sql(), "(create_user = $user_id)");
}

#[test]
fn data_scope_self_scope_appends_bound_where_condition() {
    let user = sample_user_with_scope(4);
    let filter = resolve_data_scope(&user, &sample_target(), &sample_context()).unwrap();
    let mut query = QueryBuilder::<Postgres>::new("SELECT * FROM sys_user");

    filter.append_where_clause(&mut query);

    assert_eq!(
        query.sql(),
        "SELECT * FROM sys_user WHERE (create_user = $1)"
    );
}

#[test]
fn data_scope_dept_and_child_scope_collects_descendants() {
    let user = sample_user_with_dept_and_scope(10, 2);

    let filter = resolve_data_scope(
        &user,
        &sample_target(),
        &context_with_dept_edges([(10, 11), (11, 12)]),
    )
    .unwrap();

    assert_eq!(filter.dept_ids(), &[10, 11, 12]);
}

#[test]
fn data_scope_custom_scope_uses_role_department_mapping() {
    let user = sample_user_with_scope(5);

    let filter = resolve_data_scope(
        &user,
        &sample_target(),
        &context_with_role_depts([(20, 30), (20, 31), (20, 30)]),
    )
    .unwrap();

    assert_eq!(filter.dept_ids(), &[30, 31]);
    assert_eq!(filter.to_debug_sql(), "(dept_id IN ($dept_ids))");
}

#[test]
fn data_scope_mixed_roles_keep_or_semantics() {
    let mut user = sample_user_with_dept_and_scope(10, 3);
    user.roles.push(RoleContext {
        id: 21,
        name: "本人角色".to_string(),
        code: "self_role".to_string(),
        data_scope: 4,
    });

    let filter = resolve_data_scope(&user, &sample_target(), &sample_context()).unwrap();

    assert_eq!(filter.dept_ids(), &[10]);
    assert_eq!(
        filter.to_debug_sql(),
        "(dept_id IN ($dept_ids) OR create_user = $user_id)"
    );
}

#[test]
fn data_scope_dept_scope_without_dept_column_is_bad_request() {
    let user = sample_user_with_scope(3);
    let target = DataPermissionTarget {
        dept_column: None,
        user_column: Some("create_user"),
    };

    let err = resolve_data_scope(&user, &target, &sample_context()).unwrap_err();

    assert!(matches!(err, AppError::BadRequest(_)));
}
