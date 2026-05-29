use backend_rust::{
    application::{
        data_scope::resolver::{resolve_data_scope, DataScopeContext, DeptTree, RoleDeptScope},
        system::user_service::{
            ensure_user_role_ids_can_be_assigned, normalize_user_command, user_sort_sql,
            UserCommand, UserImportResp,
        },
    },
    domain::{
        auth::model::{CurrentUser, RoleContext},
        data_scope::model::{DataPermissionTarget, DataScopeFilter},
    },
    shared::error::AppError,
};

#[test]
fn user_list_data_scope_matches_expected_visibility() {
    let context = DataScopeContext {
        dept_tree: DeptTree::from_edges([(1, 10), (10, 11), (1, 12)]),
        role_dept_scope: RoleDeptScope::from_pairs([(50, 12)]),
    };
    let target = DataPermissionTarget {
        dept_column: Some("u.dept_id"),
        user_column: Some("u.create_user"),
    };
    let records = [
        TestUserRecord {
            id: 1,
            dept_id: 10,
            create_user: 901,
        },
        TestUserRecord {
            id: 2,
            dept_id: 11,
            create_user: 902,
        },
        TestUserRecord {
            id: 3,
            dept_id: 12,
            create_user: 903,
        },
        TestUserRecord {
            id: 4,
            dept_id: 99,
            create_user: 904,
        },
        TestUserRecord {
            id: 5,
            dept_id: 99,
            create_user: 30,
        },
    ];

    let admin =
        resolve_data_scope(&current_user(1, 1, role(1, "admin", 1)), &target, &context).unwrap();
    assert_eq!(visible_ids(&admin, &records), vec![1, 2, 3, 4, 5]);

    let dept_and_child = resolve_data_scope(
        &current_user(10, 10, role(20, "dept_child", 2)),
        &target,
        &context,
    )
    .unwrap();
    assert_eq!(visible_ids(&dept_and_child, &records), vec![1, 2]);

    let dept_only = resolve_data_scope(
        &current_user(20, 10, role(30, "dept", 3)),
        &target,
        &context,
    )
    .unwrap();
    assert_eq!(visible_ids(&dept_only, &records), vec![1]);

    let self_only = resolve_data_scope(
        &current_user(30, 10, role(40, "self", 4)),
        &target,
        &context,
    )
    .unwrap();
    assert_eq!(visible_ids(&self_only, &records), vec![5]);

    let custom = resolve_data_scope(
        &current_user(40, 10, role(50, "custom", 5)),
        &target,
        &context,
    )
    .unwrap();
    assert_eq!(visible_ids(&custom, &records), vec![3]);
}

#[test]
fn user_sort_sql_uses_whitelist_and_stable_default() {
    assert_eq!(
        user_sort_sql(&["createTime,desc".to_owned(), "username,asc".to_owned()]),
        "u.create_time DESC, u.username ASC, u.id DESC"
    );
    assert_eq!(
        user_sort_sql(&["t1.createTime,desc".to_owned(), "t1.id,desc".to_owned()]),
        "u.create_time DESC, u.id DESC"
    );
    assert_eq!(
        user_sort_sql(&["username;drop table sys_user,desc".to_owned()]),
        "u.create_time DESC, u.id DESC"
    );
}

#[test]
fn user_command_normalization_rejects_empty_required_fields() {
    let err = normalize_user_command(UserCommand {
        username: " ".to_owned(),
        nickname: "Alice".to_owned(),
        password: "admin123".to_owned(),
        gender: 1,
        email: String::new(),
        phone: String::new(),
        avatar: String::new(),
        description: String::new(),
        status: 1,
        dept_id: 10,
        role_ids: vec![2],
    })
    .unwrap_err();

    assert!(matches!(err, AppError::BadRequest(_)));
}

#[test]
fn admin_role_id_cannot_be_assigned_through_user_management() {
    let err = ensure_user_role_ids_can_be_assigned(&[1, 2]).unwrap_err();

    assert!(matches!(err, AppError::BadRequest(_)));
}

#[test]
fn import_parse_response_uses_vue_compatible_fields() {
    let value = serde_json::to_value(UserImportResp {
        import_key: "k1".to_owned(),
        total_rows: 3,
        valid_rows: 2,
        duplicate_user_rows: 1,
        duplicate_email_rows: 0,
        duplicate_phone_rows: 0,
    })
    .unwrap();

    assert_eq!(value["importKey"], "k1");
    assert_eq!(value["totalRows"], 3);
    assert_eq!(value["validRows"], 2);
    assert_eq!(value["duplicateUserRows"], 1);
    assert_eq!(value["duplicateEmailRows"], 0);
    assert_eq!(value["duplicatePhoneRows"], 0);
}

#[derive(Debug, Clone, Copy)]
struct TestUserRecord {
    id: i64,
    dept_id: i64,
    create_user: i64,
}

fn visible_ids(filter: &DataScopeFilter, records: &[TestUserRecord]) -> Vec<i64> {
    records
        .iter()
        .filter(|record| {
            filter.is_unrestricted()
                || filter.dept_ids().contains(&record.dept_id)
                || filter.self_user_id == Some(record.create_user)
        })
        .map(|record| record.id)
        .collect()
}

fn current_user(id: i64, dept_id: i64, role: RoleContext) -> CurrentUser {
    CurrentUser {
        id,
        username: format!("user{id}"),
        dept_id,
        roles: vec![role],
        permissions: vec![],
    }
}

fn role(id: i64, code: &str, data_scope: i16) -> RoleContext {
    RoleContext {
        id,
        name: code.to_owned(),
        code: code.to_owned(),
        data_scope,
    }
}
