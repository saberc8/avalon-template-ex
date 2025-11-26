use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::application::auth::format_time;
use crate::security::TokenService;
use crate::{id, AppState};

use super::response::{api_fail, api_ok, ApiJson, PageResult};

/// 从请求头中解析当前登录用户 ID。
fn current_user_id(headers: &HeaderMap, token_svc: &TokenService) -> Result<i64, ()> {
    let authz = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let claims = token_svc.parse(authz).map_err(|_| ())?;
    Ok(claims.user_id)
}

/// 角色基础响应结构，对齐前端 RoleResp。
#[derive(Debug, Serialize, Clone)]
pub struct RoleResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub sort: i32,
    pub description: String,
    #[serde(rename = "dataScope")]
    pub data_scope: i32,
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    #[serde(rename = "createUserString")]
    pub create_user_string: String,
    #[serde(rename = "createTime")]
    pub create_time: String,
    #[serde(rename = "updateUserString")]
    pub update_user_string: String,
    #[serde(rename = "updateTime")]
    pub update_time: String,
    pub disabled: bool,
}

/// 角色详情响应结构，对齐 RoleDetailResp。
#[derive(Debug, Serialize, Clone)]
pub struct RoleDetailResp {
    #[serde(flatten)]
    pub base: RoleResp,
    #[serde(rename = "menuIds")]
    pub menu_ids: Vec<i64>,
    #[serde(rename = "deptIds")]
    pub dept_ids: Vec<i64>,
    #[serde(rename = "menuCheckStrictly")]
    pub menu_check_strictly: bool,
    #[serde(rename = "deptCheckStrictly")]
    pub dept_check_strictly: bool,
}

/// 角色关联用户响应结构，对齐 RoleUserResp。
#[derive(Debug, Serialize, Clone)]
pub struct RoleUserResp {
    pub id: i64,
    #[serde(rename = "roleId")]
    pub role_id: i64,
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub gender: i16,
    pub status: i16,
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    pub description: String,
    #[serde(rename = "deptId")]
    pub dept_id: i64,
    #[serde(rename = "deptName")]
    pub dept_name: String,
    #[serde(rename = "roleIds")]
    pub role_ids: Vec<i64>,
    #[serde(rename = "roleNames")]
    pub role_names: Vec<String>,
    pub disabled: bool,
}

/// 角色查询参数，仅使用 description。
#[derive(Debug, Deserialize)]
pub struct RoleListQuery {
    pub description: Option<String>,
}

/// 新增/修改角色请求体，对齐 RoleReq。
#[derive(Debug, Deserialize)]
pub struct RoleReq {
    pub name: String,
    pub code: String,
    pub sort: i32,
    pub description: String,
    #[serde(rename = "dataScope")]
    pub data_scope: i32,
    #[serde(rename = "deptIds")]
    pub dept_ids: Vec<i64>,
    #[serde(rename = "deptCheckStrictly")]
    pub dept_check_strictly: bool,
}

/// 更新角色权限请求体，对齐 RolePermissionReq。
#[derive(Debug, Deserialize)]
pub struct RolePermissionReq {
    #[serde(rename = "menuIds")]
    pub menu_ids: Vec<i64>,
    #[serde(rename = "menuCheckStrictly")]
    pub menu_check_strictly: bool,
}

/// 角色关联用户分页查询参数，对齐 RoleUserPageQuery（仅使用 page/size/description）。
#[derive(Debug, Deserialize)]
pub struct RoleUserPageQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub description: Option<String>,
}

/// 批量 ID 请求体。
#[derive(Debug, Deserialize)]
pub struct IdsRequest {
    pub ids: Vec<i64>,
}

/// GET /system/role/list 查询角色列表。
pub async fn list_role(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RoleListQuery>,
) -> ApiJson {
    let desc_filter = query
        .description
        .unwrap_or_default()
        .trim()
        .to_string();

    let rows = match sqlx::query(
        r#"
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999)          AS sort,
       COALESCE(r.description, '')    AS description,
       COALESCE(r.data_scope, 4)      AS data_scope,
       COALESCE(r.is_system, FALSE)   AS is_system,
       r.create_time,
       COALESCE(cu.nickname, '')      AS create_user_string,
       r.update_time,
       COALESCE(uu.nickname, '')      AS update_user_string
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
ORDER BY r.sort ASC, r.id ASC;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询角色失败"),
    };

    let mut list: Vec<RoleResp> = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let name: String = r.get("name");
        let code: String = r.get("code");
        let sort: i32 = r.get("sort");
        let description: String = r.get("description");
        let data_scope: i32 = r.get("data_scope");
        let is_system: bool = r.get("is_system");
        let create_time_raw: NaiveDateTime = r.get("create_time");
        let create_user_string: String = r.get("create_user_string");
        let update_time_raw: Option<NaiveDateTime> =
            r.try_get("update_time").ok();
        let update_user_string: String = r.get("update_user_string");

        if !desc_filter.is_empty()
            && !name.contains(&desc_filter)
            && !description.contains(&desc_filter)
        {
            continue;
        }

        let create_time = format_time(create_time_raw);
        let (update_time, update_user) = if let Some(t) = update_time_raw {
            (format_time(t), update_user_string)
        } else {
            ("".to_string(), "".to_string())
        };

        let disabled = is_system && code == "admin";

        list.push(RoleResp {
            id,
            name,
            code,
            sort,
            description,
            data_scope,
            is_system,
            create_user_string,
            create_time,
            update_user_string: update_user,
            update_time,
            disabled,
        });
    }

    api_ok(list)
}

/// GET /system/role/:id 查询角色详情。
pub async fn get_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiJson {
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let row = match sqlx::query(
        r#"
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999)             AS sort,
       COALESCE(r.description, '')       AS description,
       COALESCE(r.data_scope, 4)         AS data_scope,
       COALESCE(r.is_system, FALSE)      AS is_system,
       COALESCE(r.menu_check_strictly, TRUE) AS menu_check_strictly,
       COALESCE(r.dept_check_strictly, TRUE) AS dept_check_strictly,
       r.create_time,
       COALESCE(cu.nickname, '')         AS create_user_string,
       r.update_time,
       COALESCE(uu.nickname, '')         AS update_user_string
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = $1;
"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_fail("500", "查询角色失败"),
    };

    let row = match row {
        Some(r) => r,
        None => return api_fail("404", "角色不存在"),
    };

    let role_id: i64 = row.get("id");
    let name: String = row.get("name");
    let code: String = row.get("code");
    let sort: i32 = row.get("sort");
    let description: String = row.get("description");
    let data_scope: i32 = row.get("data_scope");
    let is_system: bool = row.get("is_system");
    let menu_check_strictly: bool = row.get("menu_check_strictly");
    let dept_check_strictly: bool = row.get("dept_check_strictly");
    let create_time_raw: NaiveDateTime = row.get("create_time");
    let create_user_string: String = row.get("create_user_string");
    let update_time_raw: Option<NaiveDateTime> =
        row.try_get("update_time").ok();
    let update_user_string: String = row.get("update_user_string");

    let create_time = format_time(create_time_raw);
    let (update_time, update_user) = if let Some(t) = update_time_raw {
        (format_time(t), update_user_string)
    } else {
        ("".to_string(), "".to_string())
    };

    let disabled = is_system && code == "admin";

    let base = RoleResp {
        id: role_id,
        name,
        code,
        sort,
        description,
        data_scope,
        is_system,
        create_user_string,
        create_time,
        update_user_string: update_user,
        update_time,
        disabled,
    };

    // 查询菜单 ID 列表
    let menu_rows = match sqlx::query(
        r#"SELECT menu_id FROM sys_role_menu WHERE role_id = $1 ORDER BY menu_id ASC;"#,
    )
    .bind(role_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询角色菜单失败"),
    };
    let mut menu_ids = Vec::new();
    for r in menu_rows {
        let mid: i64 = r.get("menu_id");
        menu_ids.push(mid);
    }

    // 查询部门 ID 列表
    let dept_rows = match sqlx::query(
        r#"SELECT dept_id FROM sys_role_dept WHERE role_id = $1 ORDER BY dept_id ASC;"#,
    )
    .bind(role_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询角色部门失败"),
    };
    let mut dept_ids = Vec::new();
    for r in dept_rows {
        let did: i64 = r.get("dept_id");
        dept_ids.push(did);
    }

    let resp = RoleDetailResp {
        base,
        menu_ids,
        dept_ids,
        menu_check_strictly,
        dept_check_strictly,
    };

    api_ok(resp)
}

/// POST /system/role 新增角色。
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut body): Json<RoleReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };

    body.name = body.name.trim().to_string();
    body.code = body.code.trim().to_string();
    if body.name.is_empty() || body.code.is_empty() {
        return api_fail("400", "名称和编码不能为空");
    }
    if body.sort <= 0 {
        body.sort = 999;
    }
    if body.data_scope == 0 {
        body.data_scope = 4;
    }

    let now = Local::now().naive_local();
    let new_id = id::next_id();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "新增角色失败"),
    };

    if sqlx::query(
        r#"
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6,
        FALSE, TRUE, $7,
        $8, $9);
"#,
    )
    .bind(new_id)
    .bind(&body.name)
    .bind(&body.code)
    .bind(body.data_scope)
    .bind(body.description.trim())
    .bind(body.sort)
    .bind(body.dept_check_strictly)
    .bind(current_user_id)
    .bind(now)
    .execute(&mut tx)
    .await
    .is_err()
    {
        return api_fail("500", "新增角色失败");
    }

    for did in body.dept_ids.iter() {
        if sqlx::query(
            r#"INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;"#,
        )
        .bind(new_id)
        .bind(*did)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "保存角色部门失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "新增角色失败");
    }

    api_ok(serde_json::json!({ "id": new_id }))
}

/// PUT /system/role/:id 修改角色。
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(mut body): Json<RoleReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    body.name = body.name.trim().to_string();
    if body.name.is_empty() {
        return api_fail("400", "名称不能为空");
    }
    if body.sort <= 0 {
        body.sort = 999;
    }
    if body.data_scope == 0 {
        body.data_scope = 4;
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "修改角色失败"),
    };

    if sqlx::query(
        r#"
UPDATE sys_role
   SET name               = $1,
       description        = $2,
       sort               = $3,
       data_scope         = $4,
       dept_check_strictly= $5,
       update_user        = $6,
       update_time        = $7
 WHERE id                 = $8;
"#,
    )
    .bind(&body.name)
    .bind(body.description.trim())
    .bind(body.sort)
    .bind(body.data_scope)
    .bind(body.dept_check_strictly)
    .bind(current_user_id)
    .bind(Local::now().naive_local())
    .bind(id)
    .execute(&mut tx)
    .await
    .is_err()
    {
        return api_fail("500", "修改角色失败");
    }

    if sqlx::query("DELETE FROM sys_role_dept WHERE role_id = $1")
        .bind(id)
        .execute(&mut tx)
        .await
        .is_err()
    {
        return api_fail("500", "修改角色失败");
    }

    for did in body.dept_ids.iter() {
        if sqlx::query(
            r#"INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;"#,
        )
        .bind(id)
        .bind(*did)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "保存角色部门失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "修改角色失败");
    }

    api_ok(serde_json::json!(true))
}

/// DELETE /system/role 删除角色。
pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<IdsRequest>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    let _ = current_user_id;

    if body.ids.is_empty() {
        return api_fail("400", "ID 列表不能为空");
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "删除角色失败"),
    };

    for role_id in body.ids.iter() {
        if *role_id <= 0 {
            continue;
        }
        // 跳过系统内置角色
        let is_system: bool = match sqlx::query_scalar(
            "SELECT COALESCE(is_system, FALSE) FROM sys_role WHERE id = $1",
        )
        .bind(role_id)
        .fetch_optional(&mut tx)
        .await
        {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(_) => return api_fail("500", "删除角色失败"),
        };
        if is_system {
            continue;
        }

        if sqlx::query("DELETE FROM sys_user_role WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除角色失败");
        }
        if sqlx::query("DELETE FROM sys_role_menu WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除角色失败");
        }
        if sqlx::query("DELETE FROM sys_role_dept WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除角色失败");
        }
        if sqlx::query("DELETE FROM sys_role WHERE id = $1")
            .bind(role_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除角色失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "删除角色失败");
    }

    api_ok(serde_json::json!(true))
}

/// PUT /system/role/:id/permission 修改角色菜单权限。
pub async fn update_role_permission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<RolePermissionReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "保存角色权限失败"),
    };

    if sqlx::query("DELETE FROM sys_role_menu WHERE role_id = $1")
        .bind(id)
        .execute(&mut tx)
        .await
        .is_err()
    {
        return api_fail("500", "保存角色权限失败");
    }

    for mid in body.menu_ids.iter() {
        if sqlx::query(
            r#"INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;"#,
        )
        .bind(id)
        .bind(*mid)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "保存角色权限失败");
        }
    }

    if sqlx::query(
        r#"
UPDATE sys_role
   SET menu_check_strictly = $1,
       update_user         = $2,
       update_time         = $3
 WHERE id                  = $4;
"#,
    )
    .bind(body.menu_check_strictly)
    .bind(current_user_id)
    .bind(Local::now().naive_local())
    .bind(id)
    .execute(&mut tx)
    .await
    .is_err()
    {
        return api_fail("500", "保存角色权限失败");
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "保存角色权限失败");
    }

    api_ok(serde_json::json!(true))
}

/// GET /system/role/:id/user 分页查询关联用户。
pub async fn page_role_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Query(query): Query<RoleUserPageQuery>,
) -> ApiJson {
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let mut page = query.page.unwrap_or(1);
    let mut size = query.size.unwrap_or(10);
    if page <= 0 {
        page = 1;
    }
    if size <= 0 {
        size = 10;
    }
    let desc_filter = query
        .description
        .unwrap_or_default()
        .trim()
        .to_string();

    let rows = match sqlx::query(
        r#"
SELECT ur.id,
       ur.role_id,
       u.id            AS user_id,
       u.username,
       u.nickname,
       u.gender,
       u.status,
       u.is_system,
       COALESCE(u.description, '') AS description,
       u.dept_id,
       COALESCE(d.name, '')        AS dept_name
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE ur.role_id = $1
ORDER BY ur.id DESC;
"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询关联用户失败"),
    };

    let mut all: Vec<RoleUserResp> = Vec::new();
    for r in rows {
        let id_val: i64 = r.get("id");
        let role_id: i64 = r.get("role_id");
        let user_id: i64 = r.get("user_id");
        let username: String = r.get("username");
        let nickname: String = r.get("nickname");
        let gender: i16 = r.get("gender");
        let status: i16 = r.get("status");
        let is_system: bool = r.get("is_system");
        let description: String = r.get("description");
        let dept_id: i64 = r.get("dept_id");
        let dept_name: String = r.get("dept_name");

        if !desc_filter.is_empty()
            && !username.contains(&desc_filter)
            && !nickname.contains(&desc_filter)
            && !description.contains(&desc_filter)
        {
            continue;
        }

        all.push(RoleUserResp {
            id: id_val,
            role_id,
            user_id,
            username,
            nickname,
            gender,
            status,
            is_system,
            description,
            dept_id,
            dept_name,
            role_ids: Vec::new(),
            role_names: Vec::new(),
            disabled: false,
        });
    }

    // 填充每个用户的角色列表
    if !all.is_empty() {
        let mut user_ids: Vec<i64> = Vec::new();
        let mut seen: HashMap<i64, ()> = HashMap::new();
        for item in &all {
            if !seen.contains_key(&item.user_id) {
                seen.insert(item.user_id, ());
                user_ids.push(item.user_id);
            }
        }

        let role_rows = match sqlx::query(
            r#"
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY($1::bigint[]);
"#,
        )
        .bind(&user_ids)
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => rows,
            Err(_) => Vec::new(),
        };

        let mut map: HashMap<i64, (Vec<i64>, Vec<String>)> = HashMap::new();
        for r in role_rows {
            let user_id: i64 = r.get("user_id");
            let role_id: i64 = r.get("role_id");
            let name: String = r.get("name");
            let entry = map
                .entry(user_id)
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(role_id);
            entry.1.push(name);
        }

        for item in &mut all {
            if let Some((ids, names)) = map.get(&item.user_id) {
                item.role_ids = ids.clone();
                item.role_names = names.clone();
            }
            item.disabled = item.is_system && item.role_id == 1;
        }
    }

    let total = all.len() as i64;
    let mut start = (page - 1) * size;
    if start as usize > all.len() {
        start = all.len() as i64;
    }
    let mut end = start + size;
    if end as usize > all.len() {
        end = all.len() as i64;
    }
    let page_list = all[(start as usize)..(end as usize)].to_vec();

    let resp: PageResult<RoleUserResp> = PageResult {
        list: page_list,
        total,
    };
    api_ok(resp)
}

/// POST /system/role/:id/user 分配角色给用户。
pub async fn assign_to_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(user_ids): Json<Vec<i64>>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    let _ = current_user_id;

    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }
    if user_ids.is_empty() {
        return api_fail("400", "用户ID列表不能为空");
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "分配用户失败"),
    };

    for uid in user_ids {
        if uid <= 0 {
            continue;
        }
        if sqlx::query(
            r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
        )
        .bind(id::next_id())
        .bind(uid)
        .bind(id)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "分配用户失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "分配用户失败");
    }

    api_ok(serde_json::json!(true))
}

/// DELETE /system/role/user 取消分配角色（按 userRoleId 删除）。
pub async fn unassign_from_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(ids): Json<Vec<i64>>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    let _ = current_user_id;

    if ids.is_empty() {
        return api_fail("400", "用户角色ID列表不能为空");
    }

    if sqlx::query(
        "DELETE FROM sys_user_role WHERE id = ANY($1::bigint[])",
    )
    .bind(&ids)
    .execute(&state.db)
    .await
    .is_err()
    {
        return api_fail("500", "取消分配失败");
    }

    api_ok(serde_json::json!(true))
}

/// GET /system/role/:id/user/id 查询角色关联用户 ID 列表。
pub async fn list_role_user_ids(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiJson {
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let rows = match sqlx::query(
        r#"SELECT user_id FROM sys_user_role WHERE role_id = $1;"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询关联用户失败"),
    };

    let mut ids = Vec::new();
    for r in rows {
        let uid: i64 = r.get("user_id");
        ids.push(uid);
    }

    api_ok(ids)
}

