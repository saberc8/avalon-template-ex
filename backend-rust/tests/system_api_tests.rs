use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    routing::get,
    Json, Router,
};
use backend_rust::{
    application::system::{
        dept_service::{build_dept_tree, DeptResp},
        menu_service::{build_menu_tree, MenuResp},
        role_service::{RoleDetailResp, RoleResp, RoleUserResp},
        user_service::{UserDetailResp, UserImportResp, UserResp},
    },
    application::user_profile_service::AvatarResp,
    infrastructure::security::jwt::JwtService,
    interfaces::http::{build_router, common::CommonTreeNode},
    shared::response::ApiResponse,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn test_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost:5432/avalon_admin")
        .unwrap()
}

fn test_jwt() -> JwtService {
    JwtService::new("test-secret".to_owned(), 24)
}

fn dept(id: i64, parent_id: i64, name: &str) -> DeptResp {
    DeptResp {
        id,
        name: name.to_owned(),
        sort: 1,
        status: 1,
        is_system: false,
        description: String::new(),
        create_user_string: "admin".to_owned(),
        create_time: "2026-05-29 10:00:00".to_owned(),
        update_user_string: String::new(),
        update_time: String::new(),
        parent_id,
        children: vec![],
    }
}

fn menu(id: i64, parent_id: i64, title: &str) -> MenuResp {
    MenuResp {
        id,
        title: title.to_owned(),
        parent_id,
        menu_type: 2,
        path: "/system/demo".to_owned(),
        name: "SystemDemo".to_owned(),
        component: "system/demo/index".to_owned(),
        redirect: String::new(),
        icon: "settings".to_owned(),
        is_external: false,
        is_cache: false,
        is_hidden: false,
        permission: String::new(),
        sort: 1,
        status: 1,
        create_user_string: "admin".to_owned(),
        create_time: "2026-05-29 10:00:00".to_owned(),
        update_user_string: String::new(),
        update_time: String::new(),
        children: vec![],
    }
}

mod system {
    use super::*;

    pub mod dept {
        use super::*;

        #[test]
        fn system_dept_tree_keeps_vue_field_names_and_children() {
            let tree = build_dept_tree(vec![dept(2, 1, "研发部"), dept(1, 0, "总部")]);

            assert_eq!(tree.len(), 1);
            let value = serde_json::to_value(&tree[0]).unwrap();
            assert_eq!(value["id"], 1);
            assert_eq!(value["name"], "总部");
            assert_eq!(value["parentId"], 0);
            assert_eq!(value["isSystem"], false);
            assert_eq!(value["createUserString"], "admin");
            assert_eq!(value["children"][0]["name"], "研发部");
        }

        #[test]
        fn system_dept_tree_promotes_data_scope_orphans() {
            let tree = build_dept_tree(vec![dept(3, 99, "范围内部门")]);

            assert_eq!(tree.len(), 1);
            assert_eq!(tree[0].id, 3);
        }
    }

    pub mod menu {
        use super::*;

        #[test]
        fn system_menu_tree_keeps_vue_field_names_and_children() {
            let tree = build_menu_tree(vec![menu(11, 10, "用户管理"), menu(10, 0, "系统管理")]);

            assert_eq!(tree.len(), 1);
            let value = serde_json::to_value(&tree[0]).unwrap();
            assert_eq!(value["id"], 10);
            assert_eq!(value["title"], "系统管理");
            assert_eq!(value["parentId"], 0);
            assert_eq!(value["type"], 2);
            assert_eq!(value["isExternal"], false);
            assert_eq!(value["isCache"], false);
            assert_eq!(value["isHidden"], false);
            assert_eq!(value["children"][0]["title"], "用户管理");
        }
    }

    pub mod role {
        use super::*;

        #[test]
        fn system_role_detail_keeps_permission_field_names() {
            let detail = RoleDetailResp {
                role: RoleResp {
                    id: 1,
                    name: "系统管理员".to_owned(),
                    code: "admin".to_owned(),
                    sort: 1,
                    description: String::new(),
                    data_scope: 1,
                    is_system: true,
                    create_user_string: "admin".to_owned(),
                    create_time: "2026-05-29 10:00:00".to_owned(),
                    update_user_string: String::new(),
                    update_time: String::new(),
                    disabled: true,
                },
                menu_ids: vec![1000, 1010],
                dept_ids: vec![1],
                menu_check_strictly: true,
                dept_check_strictly: true,
            };

            let value = serde_json::to_value(detail).unwrap();
            assert_eq!(value["dataScope"], 1);
            assert_eq!(value["isSystem"], true);
            assert_eq!(value["disabled"], true);
            assert_eq!(value["menuIds"], json!([1000, 1010]));
            assert_eq!(value["deptIds"], json!([1]));
            assert_eq!(value["menuCheckStrictly"], true);
            assert_eq!(value["deptCheckStrictly"], true);
        }

        #[test]
        fn system_role_user_keeps_vue_and_go_field_names() {
            let user = RoleUserResp {
                id: 100,
                role_id: 1,
                user_id: 1,
                username: "admin".to_owned(),
                nickname: "系统管理员".to_owned(),
                gender: 1,
                status: 1,
                is_system: true,
                description: "系统初始用户".to_owned(),
                dept_id: 1,
                dept_name: "总部".to_owned(),
                role_ids: vec![1, 2],
                role_names: vec!["系统管理员".to_owned(), "普通用户".to_owned()],
                disabled: true,
            };

            let value = serde_json::to_value(user).unwrap();
            assert_eq!(value["id"], 100);
            assert_eq!(value["roleId"], 1);
            assert_eq!(value["userId"], 1);
            assert_eq!(value["username"], "admin");
            assert_eq!(value["nickname"], "系统管理员");
            assert_eq!(value["deptId"], 1);
            assert_eq!(value["deptName"], "总部");
            assert_eq!(value["roleIds"], json!([1, 2]));
            assert_eq!(value["roleNames"], json!(["系统管理员", "普通用户"]));
            assert_eq!(value["disabled"], true);
        }
    }

    pub mod user {
        use super::*;

        #[test]
        fn system_user_keeps_vue_field_names() {
            let user = UserResp {
                id: 1,
                username: "admin".to_owned(),
                nickname: "系统管理员".to_owned(),
                avatar: String::new(),
                gender: 1,
                email: String::new(),
                phone: String::new(),
                description: "系统初始用户".to_owned(),
                status: 1,
                is_system: true,
                create_user_string: "admin".to_owned(),
                create_time: "2026-05-29 10:00:00".to_owned(),
                update_user_string: String::new(),
                update_time: String::new(),
                dept_id: 1,
                dept_name: "总部".to_owned(),
                role_ids: vec![1],
                role_names: vec!["系统管理员".to_owned()],
                disabled: true,
            };

            let detail = UserDetailResp {
                user,
                pwd_reset_time: "2026-05-29 10:00:00".to_owned(),
            };
            let value = serde_json::to_value(detail).unwrap();

            assert_eq!(value["id"], 1);
            assert_eq!(value["username"], "admin");
            assert_eq!(value["isSystem"], true);
            assert_eq!(value["createUserString"], "admin");
            assert_eq!(value["deptId"], 1);
            assert_eq!(value["deptName"], "总部");
            assert_eq!(value["roleIds"], json!([1]));
            assert_eq!(value["roleNames"], json!(["系统管理员"]));
            assert_eq!(value["pwdResetTime"], "2026-05-29 10:00:00");
            assert_eq!(value["disabled"], true);
        }

        #[test]
        fn system_user_import_and_avatar_responses_keep_vue_field_names() {
            let import = UserImportResp {
                import_key: "import-key".to_owned(),
                total_rows: 3,
                valid_rows: 2,
                duplicate_user_rows: 1,
                duplicate_email_rows: 0,
                duplicate_phone_rows: 0,
            };
            let value = serde_json::to_value(import).unwrap();

            assert_eq!(value["importKey"], "import-key");
            assert_eq!(value["totalRows"], 3);
            assert_eq!(value["validRows"], 2);
            assert_eq!(value["duplicateUserRows"], 1);
            assert_eq!(value["duplicateEmailRows"], 0);
            assert_eq!(value["duplicatePhoneRows"], 0);

            let avatar = serde_json::to_value(AvatarResp {
                avatar: "/file/avatar/avatar.png".to_owned(),
            })
            .unwrap();
            assert_eq!(avatar["avatar"], "/file/avatar/avatar.png");
        }
    }

    pub mod common {
        use super::*;

        #[test]
        fn system_common_tree_node_keeps_arco_and_admin_alias_fields() {
            let node = CommonTreeNode {
                key: 1,
                id: 1,
                title: "总部".to_owned(),
                name: "总部".to_owned(),
                disabled: false,
                children: vec![CommonTreeNode {
                    key: 2,
                    id: 2,
                    title: "研发部".to_owned(),
                    name: "研发部".to_owned(),
                    disabled: false,
                    children: vec![],
                }],
            };

            let value = serde_json::to_value(node).unwrap();
            assert_eq!(value["key"], 1);
            assert_eq!(value["id"], 1);
            assert_eq!(value["title"], "总部");
            assert_eq!(value["name"], "总部");
            assert_eq!(value["disabled"], false);
            assert_eq!(value["children"][0]["id"], 2);
        }

        #[test]
        fn system_common_dept_tree_nodes_are_never_disabled_for_vue_parity() {
            let node = CommonTreeNode::from(dept_with_status(1, 0, "禁用部门", 2));

            assert!(!node.disabled);
        }
    }
}

#[tokio::test]
async fn system_routes_are_registered_and_use_response_envelope_for_missing_auth() {
    let app = build_router(
        test_pool(),
        &["http://localhost:3000".to_owned()],
        test_jwt(),
    )
    .unwrap();

    for uri in [
        "/common/tree/dept",
        "/common/tree/menu",
        "/system/dept/tree",
        "/system/dept/1",
        "/system/menu/tree",
        "/system/menu/1",
        "/system/role/list",
        "/system/role/1/user",
        "/system/role/1",
        "/system/role/1/user/id",
        "/system/user",
        "/system/user/list",
        "/system/user/1",
        "/system/dict/list",
        "/system/dict/item",
        "/system/dict/1",
        "/system/option",
        "/system/storage/list",
        "/system/storage/1",
        "/system/client",
        "/system/client/1",
        "/system/file",
        "/system/file/statistics",
        "/system/file/check?sha256=abc",
        "/system/file/dir/1/size",
        "/common/dict/user",
        "/common/dict/role",
        "/common/dict/user_status",
        "/system/log",
        "/system/log/1",
        "/system/log/export/login",
        "/system/log/export/operation",
        "/monitor/online",
        "/user/profile/social",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "401", "{uri}");
        assert_eq!(body["msg"], "未授权，请重新登录", "{uri}");
        assert_eq!(body["data"], Value::Null, "{uri}");
        assert_eq!(body["success"], false, "{uri}");
        assert_epoch_millis_timestamp(&body, uri);
    }

    for (method, uri) in [
        ("POST", "/system/role/1/user"),
        ("DELETE", "/system/role/user"),
        ("POST", "/system/dict"),
        ("PUT", "/system/dict/1"),
        ("DELETE", "/system/dict"),
        ("DELETE", "/system/dict/cache/user_status"),
        ("POST", "/system/dict/item"),
        ("PUT", "/system/dict/item/1"),
        ("DELETE", "/system/dict/item"),
        ("PUT", "/system/option"),
        ("PATCH", "/system/option/value"),
        ("POST", "/system/storage"),
        ("PUT", "/system/storage/1"),
        ("DELETE", "/system/storage"),
        ("PUT", "/system/storage/1/status"),
        ("PUT", "/system/storage/1/default"),
        ("POST", "/system/client"),
        ("PUT", "/system/client/1"),
        ("DELETE", "/system/client"),
        ("POST", "/system/file/upload"),
        ("PUT", "/system/file/1"),
        ("DELETE", "/system/file"),
        ("POST", "/system/file/dir"),
        ("POST", "/common/file"),
        ("DELETE", "/monitor/online/test-token"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("[]"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{method} {uri}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "401", "{method} {uri}");
        assert_eq!(body["msg"], "未授权，请重新登录", "{method} {uri}");
        assert_eq!(body["data"], Value::Null, "{method} {uri}");
        assert_eq!(body["success"], false, "{method} {uri}");
        assert_epoch_millis_timestamp(&body, uri);
    }
}

#[tokio::test]
async fn success_path_api_compatibility_fixture_routes_include_vue_envelope_and_keys() {
    let app = Router::new()
        .route("/common/tree/dept", get(fixture_common_dept_tree))
        .route("/common/tree/menu", get(fixture_common_menu_tree))
        .route("/system/dept/tree", get(fixture_system_dept_tree))
        .route("/system/menu/tree", get(fixture_system_menu_tree))
        .route("/system/role/list", get(fixture_system_role_list));

    let cases = [
        ("/common/tree/dept", vec!["id", "name", "children"]),
        ("/common/tree/menu", vec!["id", "name", "children"]),
        ("/system/dept/tree", vec!["id", "name", "children"]),
        ("/system/menu/tree", vec!["id", "title", "children"]),
        ("/system/role/list", vec!["id", "dataScope"]),
    ];

    for (uri, keys) in cases {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(body["code"], "200", "{uri}");
        assert_eq!(body["success"], true, "{uri}");
        assert_epoch_millis_timestamp(&body, uri);

        let first = &body["data"][0];
        for key in keys {
            assert!(first.get(key).is_some(), "{uri} missing {key}");
        }
    }
}

#[tokio::test]
#[ignore = "requires migrated PostgreSQL seed data; run with DATABASE_URL pointing at a local test database"]
async fn real_system_role_list_success_path_uses_vue_envelope() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/avalon_admin".to_owned());
    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("connect to seeded PostgreSQL test database");
    let jwt = test_jwt();
    let token = jwt.issue(1, "admin").expect("issue test JWT");
    let app = build_router(db, &["http://localhost:3000".to_owned()], jwt).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/system/role/list")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice::<Value>(&body).unwrap();
    assert_eq!(body["code"], "200");
    assert_eq!(body["success"], true);
    assert_epoch_millis_timestamp(&body, "/system/role/list");

    let first = &body["data"][0];
    assert!(first.get("id").is_some());
    assert!(first.get("name").is_some());
    assert!(first.get("code").is_some());
    assert!(first.get("dataScope").is_some());
}

#[tokio::test]
#[ignore = "requires migrated PostgreSQL seed data; run with DATABASE_URL pointing at a local test database"]
async fn real_system_role_user_list_success_path_uses_vue_page_envelope() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/avalon_admin".to_owned());
    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("connect to seeded PostgreSQL test database");
    let jwt = test_jwt();
    let token = jwt.issue(1, "admin").expect("issue test JWT");
    let app = build_router(db, &["http://localhost:3000".to_owned()], jwt).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/system/role/1/user?page=1&size=10")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice::<Value>(&body).unwrap();
    assert_eq!(body["code"], "200");
    assert_eq!(body["success"], true);
    assert_epoch_millis_timestamp(&body, "/system/role/1/user");
    assert!(body["data"]["total"].as_i64().unwrap_or_default() >= 1);

    let first = &body["data"]["list"][0];
    for key in [
        "id",
        "roleId",
        "userId",
        "username",
        "nickname",
        "gender",
        "status",
        "isSystem",
        "description",
        "deptId",
        "deptName",
        "roleIds",
        "roleNames",
        "disabled",
    ] {
        assert!(first.get(key).is_some(), "missing {key}");
    }
}

#[tokio::test]
#[ignore = "requires migrated PostgreSQL seed data; run with DATABASE_URL pointing at a local test database"]
async fn real_system_role_user_assign_and_unassign_accept_raw_id_arrays() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/avalon_admin".to_owned());
    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("connect to seeded PostgreSQL test database");
    let test_user_id = 9_000_000_001_i64;
    cleanup_role_user_test_data(&db, test_user_id).await;
    sqlx::query(
        r#"
INSERT INTO sys_user (
    id, username, nickname, gender, status, is_system, description,
    dept_id, create_user, create_time
)
VALUES ($1, 'rust_role_assign_test', 'Rust角色分配测试', 0, 1, FALSE, 'role assign test',
        1, 1, NOW());
"#,
    )
    .bind(test_user_id)
    .execute(&db)
    .await
    .expect("insert temporary role assignment user");

    let jwt = test_jwt();
    let token = jwt.issue(1, "admin").expect("issue test JWT");
    let app = build_router(db.clone(), &["http://localhost:3000".to_owned()], jwt).unwrap();

    let assign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/role/2/user")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("[\"{test_user_id}\", 0]")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_success_bool(assign).await;

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/role/2/user?description=rust_role_assign_test")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(list).await;
    assert_eq!(body["code"], "200");
    assert_eq!(body["success"], true);
    let user_role_id = body["data"]["list"][0]["id"]
        .as_i64()
        .expect("sys_user_role.id should be returned as id");
    assert_eq!(body["data"]["list"][0]["roleId"], 2);
    assert_eq!(body["data"]["list"][0]["userId"], test_user_id);

    let unassign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/system/role/user")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("[\"{user_role_id}\", 0]")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_success_bool(unassign).await;

    let list_after_unassign = app
        .oneshot(
            Request::builder()
                .uri("/system/role/2/user?description=rust_role_assign_test")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(list_after_unassign).await;
    assert_eq!(body["code"], "200");
    assert_eq!(body["data"]["total"], 0);

    cleanup_role_user_test_data(&db, test_user_id).await;
}

#[tokio::test]
#[ignore = "requires migrated PostgreSQL seed data; run with DATABASE_URL pointing at a local test database"]
async fn real_system_role_user_system_protection_blocks_admin_assignment_and_unassignment() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/avalon_admin".to_owned());
    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("connect to seeded PostgreSQL test database");
    let test_user_id = 9_000_000_002_i64;
    cleanup_role_user_test_data(&db, test_user_id).await;
    sqlx::query(
        r#"
INSERT INTO sys_user (
    id, username, nickname, gender, status, is_system, description,
    dept_id, create_user, create_time
)
VALUES ($1, 'rust_admin_assign_block_test', 'Rust管理员分配保护测试', 0, 1, FALSE,
        'admin assign protection test', 1, 1, NOW());
"#,
    )
    .bind(test_user_id)
    .execute(&db)
    .await
    .expect("insert temporary admin assignment protection user");

    let admin_user_role_id = sqlx::query_scalar::<_, i64>(
        r#"
SELECT ur.id
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
JOIN sys_role AS r ON r.id = ur.role_id
WHERE u.is_system = TRUE
  AND r.code = 'admin'
LIMIT 1;
"#,
    )
    .fetch_one(&db)
    .await
    .expect("seeded admin user role association should exist");

    let jwt = test_jwt();
    let token = jwt.issue(1, "admin").expect("issue test JWT");
    let app = build_router(db.clone(), &["http://localhost:3000".to_owned()], jwt).unwrap();

    let assign_admin = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/role/1/user")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("[\"{test_user_id}\"]")))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(assign_admin).await;
    assert_eq!(body["code"], "400");
    assert_eq!(body["success"], false);

    let assigned_admin_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sys_user_role WHERE role_id = 1 AND user_id = $1;",
    )
    .bind(test_user_id)
    .fetch_one(&db)
    .await
    .expect("count temporary admin assignment");
    assert_eq!(assigned_admin_count, 0);

    let unassign_admin = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/system/role/user")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!("[\"{admin_user_role_id}\"]")))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(unassign_admin).await;
    assert_eq!(body["code"], "400");
    assert_eq!(body["success"], false);

    let admin_relation_still_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_user_role WHERE id = $1);")
            .bind(admin_user_role_id)
            .fetch_one(&db)
            .await
            .expect("check protected admin relation still exists");
    assert!(admin_relation_still_exists);

    cleanup_role_user_test_data(&db, test_user_id).await;
}

fn dept_with_status(id: i64, parent_id: i64, name: &str, status: i16) -> DeptResp {
    DeptResp {
        status,
        ..dept(id, parent_id, name)
    }
}

fn assert_epoch_millis_timestamp(body: &Value, context: &str) {
    let timestamp = body["timestamp"]
        .as_str()
        .unwrap_or_else(|| panic!("{context} timestamp must be a string"));
    assert!(timestamp.parse::<i64>().is_ok(), "{context} timestamp");
}

async fn response_json(response: axum::response::Response) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice::<Value>(&body).unwrap()
}

async fn assert_success_bool(response: axum::response::Response) {
    let body = response_json(response).await;
    assert_eq!(body["code"], "200");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"], true);
}

async fn cleanup_role_user_test_data(db: &sqlx::PgPool, test_user_id: i64) {
    sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1;")
        .bind(test_user_id)
        .execute(db)
        .await
        .expect("cleanup temporary role assignment relations");
    sqlx::query("DELETE FROM sys_user WHERE id = $1;")
        .bind(test_user_id)
        .execute(db)
        .await
        .expect("cleanup temporary role assignment user");
}

async fn fixture_common_dept_tree() -> Json<ApiResponse<Vec<CommonTreeNode>>> {
    Json(ApiResponse::ok(vec![CommonTreeNode::from(dept(
        1, 0, "总部",
    ))]))
}

async fn fixture_common_menu_tree() -> Json<ApiResponse<Vec<CommonTreeNode>>> {
    Json(ApiResponse::ok(vec![CommonTreeNode::from(menu(
        10,
        0,
        "系统管理",
    ))]))
}

async fn fixture_system_dept_tree() -> Json<ApiResponse<Vec<DeptResp>>> {
    Json(ApiResponse::ok(build_dept_tree(vec![dept(1, 0, "总部")])))
}

async fn fixture_system_menu_tree() -> Json<ApiResponse<Vec<MenuResp>>> {
    Json(ApiResponse::ok(build_menu_tree(vec![menu(
        10,
        0,
        "系统管理",
    )])))
}

async fn fixture_system_role_list() -> Json<ApiResponse<Vec<RoleResp>>> {
    Json(ApiResponse::ok(vec![RoleResp {
        id: 1,
        name: "系统管理员".to_owned(),
        code: "admin".to_owned(),
        sort: 1,
        description: String::new(),
        data_scope: 1,
        is_system: true,
        create_user_string: "admin".to_owned(),
        create_time: "2026-05-29 10:00:00".to_owned(),
        update_user_string: String::new(),
        update_time: String::new(),
        disabled: true,
    }]))
}
