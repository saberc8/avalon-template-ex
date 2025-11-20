use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use sqlx::Row;

use crate::AppState;

use super::response::{api_fail, api_ok, ApiJson};

/// 通用 label/value 结构，对齐 Go 版 LabelValue。
#[derive(Serialize)]
struct LabelValue {
    label: String,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "String::is_empty")]
    extra: String,
}

/// 通用树节点结构，用于部门/菜单树。
#[derive(Serialize)]
struct TreeNode {
    key: i64,
    title: String,
    disabled: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeNode>,
}

/// GET /common/dict/option/site
pub async fn list_site_options(State(state): State<Arc<AppState>>) -> ApiJson {
    let rows = match sqlx::query(
        r#"
SELECT code,
       COALESCE(value, default_value, '') AS value
FROM sys_option
WHERE category = 'SITE'
ORDER BY id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询网站配置失败"),
    };

    let mut list = Vec::new();
    for r in rows {
        let code: String = r.get("code");
        let value: String = r.get("value");
        list.push(LabelValue {
            label: code,
            value: serde_json::Value::String(value),
            extra: String::new(),
        });
    }

    api_ok(list)
}

/// GET /common/tree/menu
pub async fn list_menu_tree(State(state): State<Arc<AppState>>) -> ApiJson {
    let rows = match sqlx::query(
        r#"
SELECT id, title, parent_id, sort, status
FROM sys_menu
WHERE type IN (1, 2)
ORDER BY sort ASC, id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询菜单失败"),
    };

    if rows.is_empty() {
        let list: Vec<TreeNode> = Vec::new();
        return api_ok(list);
    }

    #[derive(Clone)]
    struct MenuRow {
        id: i64,
        title: String,
        parent_id: i64,
        sort: i32,
        status: i16,
    }

    let mut flat_rows = Vec::new();
    for r in rows {
        flat_rows.push(MenuRow {
            id: r.get("id"),
            title: r.get("title"),
            parent_id: r.get("parent_id"),
            sort: r.get("sort"),
            status: r.get("status"),
        });
    }

    use std::collections::HashMap;
    let mut node_map: HashMap<i64, TreeNode> = HashMap::new();
    for m in &flat_rows {
        node_map.insert(
            m.id,
            TreeNode {
                key: m.id,
                title: m.title.clone(),
                disabled: m.status != 1,
                children: Vec::new(),
            },
        );
    }

    let mut roots = Vec::new();
    for m in &flat_rows {
        if m.parent_id == 0 {
            if let Some(node) = node_map.get(&m.id).cloned() {
                roots.push(node);
            }
            continue;
        }
        if let Some(parent) = node_map.get_mut(&m.parent_id) {
            if let Some(node) = node_map.get(&m.id).cloned() {
                parent.children.push(node);
            }
        } else if let Some(node) = node_map.get(&m.id).cloned() {
            roots.push(node);
        }
    }

    api_ok(roots)
}

/// GET /common/tree/dept
pub async fn list_dept_tree(State(state): State<Arc<AppState>>) -> ApiJson {
    let rows = match sqlx::query(
        r#"
SELECT id, name, parent_id, sort, status, is_system
FROM sys_dept
ORDER BY sort ASC, id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询部门失败"),
    };

    if rows.is_empty() {
        let list: Vec<TreeNode> = Vec::new();
        return api_ok(list);
    }

    #[derive(Clone)]
    struct DeptRow {
        id: i64,
        name: String,
        parent_id: i64,
        sort: i32,
        status: i16,
        is_system: bool,
    }

    let mut flat_rows = Vec::new();
    for r in rows {
        flat_rows.push(DeptRow {
            id: r.get("id"),
            name: r.get("name"),
            parent_id: r.get("parent_id"),
            sort: r.get("sort"),
            status: r.get("status"),
            is_system: r.get("is_system"),
        });
    }

    use std::collections::HashMap;
    let mut node_map: HashMap<i64, TreeNode> = HashMap::new();
    for d in &flat_rows {
        node_map.insert(
            d.id,
            TreeNode {
                key: d.id,
                title: d.name.clone(),
                disabled: false,
                children: Vec::new(),
            },
        );
    }

    let mut roots = Vec::new();
    for d in &flat_rows {
        if d.parent_id == 0 {
            if let Some(node) = node_map.get(&d.id).cloned() {
                roots.push(node);
            }
            continue;
        }
        if let Some(parent) = node_map.get_mut(&d.parent_id) {
            if let Some(node) = node_map.get(&d.id).cloned() {
                parent.children.push(node);
            }
        } else if let Some(node) = node_map.get(&d.id).cloned() {
            roots.push(node);
        }
    }

    api_ok(roots)
}

/// GET /common/dict/user
pub async fn list_user_dict(State(state): State<Arc<AppState>>) -> ApiJson {
    // 目前仅支持 status=1 的启用用户
    let rows = match sqlx::query(
        r#"
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '') AS username
FROM sys_user
WHERE status = 1
ORDER BY id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    let mut list = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let nickname: String = r.get("nickname");
        let username: String = r.get("username");
        list.push(LabelValue {
            label: nickname,
            value: serde_json::Value::from(id),
            extra: username,
        });
    }

    api_ok(list)
}

/// GET /common/dict/role
pub async fn list_role_dict(State(state): State<Arc<AppState>>) -> ApiJson {
    let rows = match sqlx::query(
        r#"
SELECT id, name, code
FROM sys_role
ORDER BY sort ASC, id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询角色失败"),
    };

    let mut list = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let name: String = r.get("name");
        let code: String = r.get("code");
        list.push(LabelValue {
            label: name,
            value: serde_json::Value::from(id),
            extra: code,
        });
    }

    api_ok(list)
}

/// GET /common/dict/:code
pub async fn list_dict_by_code(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> ApiJson {
    let code = code.trim().to_string();
    if code.is_empty() {
        let list: Vec<LabelValue> = Vec::new();
        return api_ok(list);
    }

    let rows = match sqlx::query(
        r#"
SELECT t1.label,
       t1.value,
       COALESCE(t1.color, '') AS extra
FROM sys_dict_item AS t1
LEFT JOIN sys_dict AS t2 ON t1.dict_id = t2.id
WHERE t1.status = 1
  AND t2.code = $1
ORDER BY t1.sort ASC, t1.id ASC;
"#,
    )
    .bind(&code)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询字典失败"),
    };

    let mut list = Vec::new();
    for r in rows {
        let label: String = r.get("label");
        let value: String = r.get("value");
        let extra: String = r.get("extra");
        list.push(LabelValue {
            label,
            value: serde_json::Value::String(value),
            extra,
        });
    }

    api_ok(list)
}

