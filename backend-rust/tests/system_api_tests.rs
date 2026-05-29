use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend_rust::{
    application::system::{
        dept_service::{build_dept_tree, DeptResp},
        menu_service::{build_menu_tree, MenuResp},
        role_service::{RoleDetailResp, RoleResp},
    },
    infrastructure::security::jwt::JwtService,
    interfaces::http::{build_router, common::CommonTreeNode},
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
        "/system/role/1",
        "/system/role/1/user/id",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        assert_eq!(
            body,
            json!({"code": "401", "msg": "未授权，请重新登录", "data": null}),
            "{uri}"
        );
    }
}
