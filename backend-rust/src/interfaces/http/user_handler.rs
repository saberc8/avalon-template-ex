use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use sqlx::Row;

use crate::application::auth::{RouteItem, UserInfo};
use crate::security::TokenService;
use crate::AppState;

use super::response::{api_fail, api_ok, ApiJson};

/// 从 Authorization 头中解析当前用户 ID。
fn current_user_id(headers: &HeaderMap, token_svc: &TokenService) -> Result<i64, ()> {
    let authz = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let claims = token_svc.parse(authz).map_err(|_| ())?;
    Ok(claims.user_id)
}

/// GET /auth/user/info
pub async fn get_user_info(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiJson {
    let user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };

    // 查询基础用户信息
    let row = match sqlx::query(
        r#"
SELECT u.id,
       u.username,
       u.nickname,
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.avatar, ''),
       COALESCE(u.description, ''),
       u.pwd_reset_time,
       u.create_time,
       COALESCE(d.name, '') AS dept_name
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE u.id = $1
LIMIT 1;
"#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_fail("500", "获取用户信息失败"),
    };

    let row = match row {
        Some(r) => r,
        None => return api_fail("401", "未授权，请重新登录"),
    };

    let id: i64 = row.get("id");
    let username: String = row.get("username");
    let nickname: String = row.get("nickname");
    let gender: i16 = row.get("gender");
    let email: String = row.get("email");
    let phone: String = row.get("phone");
    let avatar: String = row.get("avatar");
    let description: String = row.get("description");
    let dept_name: String = row.get("dept_name");
    let create_time: chrono::NaiveDateTime = row.get("create_time");
    let pwd_reset_time: Option<chrono::NaiveDateTime> = row.try_get("pwd_reset_time").ok();

    // 查询角色编码列表
    let role_rows = match sqlx::query(
        r#"
SELECT r.code
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = $1;
"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "获取角色信息失败"),
    };
    let roles: Vec<String> = role_rows.into_iter().map(|r| r.get("code")).collect();

    // 查询权限标识列表
    let perm_rows = match sqlx::query(
        r#"
SELECT DISTINCT m.permission
FROM sys_menu AS m
LEFT JOIN sys_role_menu AS rm ON rm.menu_id = m.id
LEFT JOIN sys_role AS r ON r.id = rm.role_id
LEFT JOIN sys_user_role AS ur ON ur.role_id = r.id
LEFT JOIN sys_user AS u ON u.id = ur.user_id
WHERE u.id = $1
  AND m.status = 1
  AND m.permission IS NOT NULL;
"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "获取权限信息失败"),
    };
    let permissions: Vec<String> =
        perm_rows.into_iter().map(|r| r.get("permission")).collect();

    let pwd_reset_time_str = pwd_reset_time
        .map(|t| {
            let dt: chrono::DateTime<chrono::Local> =
                chrono::DateTime::from_utc(t, *chrono::Local::now().offset());
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_default();
    let registration_date = {
        let dt: chrono::DateTime<chrono::Local> =
            chrono::DateTime::from_utc(create_time, *chrono::Local::now().offset());
        dt.format("%Y-%m-%d").to_string()
    };

    let info = UserInfo {
        id,
        username,
        nickname,
        gender,
        email,
        phone,
        avatar,
        description,
        pwd_reset_time: pwd_reset_time_str,
        pwd_expired: false,
        registration_date,
        dept_name,
        roles,
        permissions,
    };

    api_ok(info)
}

/// GET /auth/user/route
pub async fn list_user_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiJson {
    let user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };

    // 查询角色列表
    let role_rows = match sqlx::query(
        r#"
SELECT r.id, r.code
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = $1;
"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "获取角色信息失败"),
    };
    if role_rows.is_empty() {
        let routes: Vec<RouteItem> = Vec::new();
        return api_ok(routes);
    }

    let mut role_ids = Vec::new();
    let mut role_codes = Vec::new();
    for r in role_rows {
        let id: i64 = r.get("id");
        let code: String = r.get("code");
        role_ids.push(id);
        role_codes.push(code);
    }

    // 查询角色对应的菜单
    let menu_rows = match sqlx::query(
        r#"
SELECT m.id,
       m.parent_id,
       m.title,
       m.type,
       COALESCE(m.path, '') AS path,
       COALESCE(m.name, '') AS name,
       COALESCE(m.component, '') AS component,
       COALESCE(m.redirect, '') AS redirect,
       COALESCE(m.icon, '') AS icon,
       COALESCE(m.is_external, false) AS is_external,
       COALESCE(m.is_cache, false) AS is_cache,
       COALESCE(m.is_hidden, false) AS is_hidden,
       COALESCE(m.permission, '') AS permission,
       COALESCE(m.sort, 0) AS sort,
       m.status
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
WHERE rm.role_id = ANY($1);
"#,
    )
    .bind(&role_ids)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "获取菜单信息失败"),
    };

    // 构建路由树（过滤掉按钮类型 type=3）
    let mut flat: Vec<RouteItem> = Vec::new();
    for r in menu_rows {
        let menu_type: i16 = r.get("type");
        if menu_type == 3 {
            continue;
        }
        let item = RouteItem {
            id: r.get("id"),
            parent_id: r.get("parent_id"),
            title: r.get("title"),
            menu_type,
            path: r.get("path"),
            name: r.get("name"),
            component: r.get("component"),
            redirect: r.get("redirect"),
            icon: r.get("icon"),
            is_external: r.get("is_external"),
            is_hidden: r.get("is_hidden"),
            is_cache: r.get("is_cache"),
            permission: r.get("permission"),
            roles: role_codes.clone(),
            sort: r.get("sort"),
            status: r.get("status"),
            children: Vec::new(),
            active_menu: "".to_string(),
            always_show: false,
            breadcrumb: true,
            show_in_tabs: true,
            affix: false,
        };
        flat.push(item);
    }

    if flat.is_empty() {
        let routes: Vec<RouteItem> = Vec::new();
        return api_ok(routes);
    }

    // 按 sort/id 排序
    flat.sort_by(|a, b| {
        if a.sort == b.sort {
            a.id.cmp(&b.id)
        } else {
            a.sort.cmp(&b.sort)
        }
    });

    // 构建 id -> 节点映射
    use std::collections::HashMap;
    let mut node_map: HashMap<i64, RouteItem> = HashMap::new();
    for item in flat {
        node_map.insert(item.id, item);
    }

    // 第二次遍历组装树结构
    let mut roots: Vec<RouteItem> = Vec::new();
    let keys: Vec<i64> = node_map.keys().cloned().collect();
    for id in keys {
        let mut item = match node_map.remove(&id) {
            Some(i) => i,
            None => continue,
        };
        if item.parent_id == 0 || !node_map.contains_key(&item.parent_id) {
            roots.push(item);
        } else {
            if let Some(parent) = node_map.get_mut(&item.parent_id) {
                parent.children.push(item);
            } else {
                roots.push(item);
            }
        }
    }

    api_ok(roots)
}

