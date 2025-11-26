use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::application::auth::format_time;
use crate::db::DbPool;
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

/// 用户列表/详情响应结构，对齐前端 UserResp。
#[derive(Debug, Serialize, Clone)]
pub struct SystemUserResp {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub gender: i16,
    pub email: String,
    pub phone: String,
    pub description: String,
    pub status: i16,
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

/// 用户详情响应结构，附加 pwdResetTime。
#[derive(Debug, Serialize, Clone)]
pub struct SystemUserDetailResp {
    #[serde(flatten)]
    pub base: SystemUserResp,
    #[serde(rename = "pwdResetTime", skip_serializing_if = "Option::is_none")]
    pub pwd_reset_time: Option<String>,
}

/// 分页查询参数。
#[derive(Debug, Deserialize)]
pub struct SystemUserListQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub description: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "deptId")]
    pub dept_id: Option<String>,
}

/// 新增/修改用户请求体。
#[derive(Debug, Deserialize)]
pub struct SystemUserReq {
    pub username: String,
    pub nickname: String,
    pub password: Option<String>,
    pub gender: i16,
    pub email: String,
    pub phone: String,
    pub avatar: String,
    pub description: String,
    pub status: i16,
    #[serde(rename = "deptId")]
    pub dept_id: i64,
    #[serde(rename = "roleIds")]
    pub role_ids: Vec<i64>,
}

/// 重置密码请求体。
#[derive(Debug, Deserialize)]
pub struct SystemUserPasswordResetReq {
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

/// 分配角色请求体。
#[derive(Debug, Deserialize)]
pub struct SystemUserRoleUpdateReq {
    #[serde(rename = "roleIds")]
    pub role_ids: Vec<i64>,
}

/// 批量 ID 请求体。
#[derive(Debug, Deserialize)]
pub struct IdsRequest {
    pub ids: Vec<i64>,
}

/// 导入解析结果。
#[derive(Debug, Serialize)]
pub struct UserImportParseResp {
    #[serde(rename = "importKey")]
    pub import_key: String,
    #[serde(rename = "totalRows")]
    pub total_rows: i32,
    #[serde(rename = "validRows")]
    pub valid_rows: i32,
    #[serde(rename = "duplicateUserRows")]
    pub duplicate_user_rows: i32,
    #[serde(rename = "duplicateEmailRows")]
    pub duplicate_email_rows: i32,
    #[serde(rename = "duplicatePhoneRows")]
    pub duplicate_phone_rows: i32,
}

/// 导入结果。
#[derive(Debug, Serialize)]
pub struct UserImportResultResp {
    #[serde(rename = "totalRows")]
    pub total_rows: i32,
    #[serde(rename = "insertRows")]
    pub insert_rows: i32,
    #[serde(rename = "updateRows")]
    pub update_rows: i32,
}

/// GET /system/user 分页查询用户列表。
pub async fn list_user_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SystemUserListQuery>,
) -> ApiJson {
    let mut page = query.page.unwrap_or(1);
    let mut size = query.size.unwrap_or(10);
    if page <= 0 {
        page = 1;
    }
    if size <= 0 {
        size = 10;
    }

    let desc = query
        .description
        .unwrap_or_default()
        .trim()
        .to_string();
    let status_filter: i16 = query
        .status
        .as_deref()
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0);
    let dept_id: i64 = query
        .dept_id
        .as_deref()
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0);

    let desc_pattern = if desc.is_empty() {
        String::new()
    } else {
        format!("%{}%", desc)
    };

    // 计算总数
    let total: i64 = match sqlx::query_scalar(
        r#"
SELECT COUNT(*)::bigint
FROM sys_user AS u
WHERE ($1 = '' OR u.username ILIKE $1 OR u.nickname ILIKE $1 OR COALESCE(u.description,'') ILIKE $1)
  AND ($2 = 0 OR u.status = $2)
  AND ($3 = 0 OR u.dept_id = $3);
"#,
    )
    .bind(desc_pattern.clone())
    .bind(status_filter)
    .bind(dept_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(cnt) => cnt,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    if total == 0 {
        let empty: PageResult<SystemUserResp> = PageResult {
            list: Vec::new(),
            total: 0,
        };
        return api_ok(empty);
    }

    let offset: i64 = (page - 1) * size;

    // 查询分页数据
    let rows = match sqlx::query(
        r#"
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE ($1 = '' OR u.username ILIKE $1 OR u.nickname ILIKE $1 OR COALESCE(u.description,'') ILIKE $1)
  AND ($2 = 0 OR u.status = $2)
  AND ($3 = 0 OR u.dept_id = $3)
ORDER BY u.id DESC
LIMIT $4 OFFSET $5;
"#,
    )
    .bind(desc_pattern)
    .bind(status_filter)
    .bind(dept_id)
    .bind(size)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    let mut users: Vec<SystemUserResp> = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let username: String = r.get("username");
        let nickname: String = r.get("nickname");
        let avatar: String = r.get("avatar");
        let gender: i16 = r.get("gender");
        let email: String = r.get("email");
        let phone: String = r.get("phone");
        let description: String = r.get("description");
        let status: i16 = r.get("status");
        let is_system: bool = r.get("is_system");
        let dept_id_val: i64 = r.get("dept_id");
        let dept_name: String = r.get("dept_name");
        let create_time_raw: NaiveDateTime = r.get("create_time");
        let create_user: String = r.get("create_user_string");
        let update_time_raw: Option<NaiveDateTime> =
            r.try_get("update_time").ok();
        let update_user: String = r.get("update_user_string");

        let create_time = format_time(create_time_raw);
        let (update_time, update_user_string) = if let Some(t) = update_time_raw
        {
            (format_time(t), update_user)
        } else {
            ("".to_string(), "".to_string())
        };

        let user = SystemUserResp {
            id,
            username,
            nickname,
            avatar,
            gender,
            email,
            phone,
            description,
            status,
            is_system,
            create_user_string: create_user,
            create_time,
            update_user_string,
            update_time,
            dept_id: dept_id_val,
            dept_name,
            role_ids: Vec::new(),
            role_names: Vec::new(),
            disabled: is_system,
        };
        users.push(user);
    }

    if fill_user_roles(&state.db, &mut users).await.is_err() {
        // 角色填充失败不影响主流程，忽略错误
    }

    let resp: PageResult<SystemUserResp> = PageResult {
        list: users,
        total,
    };
    api_ok(resp)
}

/// GET /system/user/list 查询所有用户列表。
pub async fn list_all_user(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiJson {
    let mut id_list: Vec<i64> = Vec::new();
    if let Some(ids_param) = params.get("userIds") {
        for part in ids_param.split(',') {
            if let Ok(v) = part.parse::<i64>() {
                if v > 0 {
                    id_list.push(v);
                }
            }
        }
    }

    let rows = if !id_list.is_empty() {
        sqlx::query(
            r#"
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = ANY($1::bigint[])
ORDER BY u.id DESC;
"#,
        )
        .bind(&id_list)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query(
            r#"
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
ORDER BY u.id DESC;
"#,
        )
        .fetch_all(&state.db)
        .await
    };

    let rows = match rows {
        Ok(r) => r,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    let mut users: Vec<SystemUserResp> = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let username: String = r.get("username");
        let nickname: String = r.get("nickname");
        let avatar: String = r.get("avatar");
        let gender: i16 = r.get("gender");
        let email: String = r.get("email");
        let phone: String = r.get("phone");
        let description: String = r.get("description");
        let status: i16 = r.get("status");
        let is_system: bool = r.get("is_system");
        let dept_id_val: i64 = r.get("dept_id");
        let dept_name: String = r.get("dept_name");
        let create_time_raw: NaiveDateTime = r.get("create_time");
        let create_user: String = r.get("create_user_string");
        let update_time_raw: Option<NaiveDateTime> =
            r.try_get("update_time").ok();
        let update_user: String = r.get("update_user_string");

        let create_time = format_time(create_time_raw);
        let (update_time, update_user_string) = if let Some(t) = update_time_raw
        {
            (format_time(t), update_user)
        } else {
            ("".to_string(), "".to_string())
        };

        let user = SystemUserResp {
            id,
            username,
            nickname,
            avatar,
            gender,
            email,
            phone,
            description,
            status,
            is_system,
            create_user_string: create_user,
            create_time,
            update_user_string,
            update_time,
            dept_id: dept_id_val,
            dept_name,
            role_ids: Vec::new(),
            role_names: Vec::new(),
            disabled: is_system,
        };
        users.push(user);
    }

    if fill_user_roles(&state.db, &mut users).await.is_err() {
        // 忽略角色填充错误
    }

    api_ok(users)
}

/// 填充用户的角色 ID/名称信息。
async fn fill_user_roles(db: &DbPool, users: &mut [SystemUserResp]) -> anyhow::Result<()> {
    if users.is_empty() {
        return Ok(());
    }

    let mut user_ids: Vec<i64> = Vec::new();
    for u in users.iter() {
        if u.id > 0 && !user_ids.contains(&u.id) {
            user_ids.push(u.id);
        }
    }
    if user_ids.is_empty() {
        return Ok(());
    }

    let rows = sqlx::query(
        r#"
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY($1::bigint[]);
"#,
    )
    .bind(&user_ids)
    .fetch_all(db)
    .await?;

    let mut map: HashMap<i64, (Vec<i64>, Vec<String>)> = HashMap::new();
    for r in rows {
        let user_id: i64 = r.get("user_id");
        let role_id: i64 = r.get("role_id");
        let name: String = r.get("name");
        let entry = map
            .entry(user_id)
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.0.push(role_id);
        entry.1.push(name);
    }

    for u in users.iter_mut() {
        if let Some((ids, names)) = map.get(&u.id) {
            u.role_ids = ids.clone();
            u.role_names = names.clone();
        }
    }

    Ok(())
}

/// GET /system/user/:id 查询用户详情。
pub async fn get_user_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiJson {
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let row = match sqlx::query(
        r#"
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.pwd_reset_time,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = $1;
"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    let row = match row {
        Some(r) => r,
        None => return api_fail("404", "用户不存在"),
    };

    let user_id: i64 = row.get("id");
    let username: String = row.get("username");
    let nickname: String = row.get("nickname");
    let avatar: String = row.get("avatar");
    let gender: i16 = row.get("gender");
    let email: String = row.get("email");
    let phone: String = row.get("phone");
    let description: String = row.get("description");
    let status: i16 = row.get("status");
    let is_system: bool = row.get("is_system");
    let dept_id_val: i64 = row.get("dept_id");
    let dept_name: String = row.get("dept_name");
    let pwd_reset_time_raw: Option<NaiveDateTime> =
        row.try_get("pwd_reset_time").ok();
    let create_time_raw: NaiveDateTime = row.get("create_time");
    let create_user_string: String = row.get("create_user_string");
    let update_time_raw: Option<NaiveDateTime> = row.try_get("update_time").ok();
    let update_user_string: String = row.get("update_user_string");

    let create_time = format_time(create_time_raw);
    let (update_time, update_user) = if let Some(t) = update_time_raw {
        (format_time(t), update_user_string)
    } else {
        ("".to_string(), "".to_string())
    };
    let pwd_reset_time = pwd_reset_time_raw.map(format_time);

    let mut base = SystemUserResp {
        id: user_id,
        username,
        nickname,
        avatar,
        gender,
        email,
        phone,
        description,
        status,
        is_system,
        create_user_string,
        create_time,
        update_user_string: update_user,
        update_time,
        dept_id: dept_id_val,
        dept_name,
        role_ids: Vec::new(),
        role_names: Vec::new(),
        disabled: is_system,
    };

    if fill_user_roles(&state.db, &mut [base.clone()])
        .await
        .is_err()
    {
        // 忽略角色填充错误
    }

    let resp = SystemUserDetailResp {
        base,
        pwd_reset_time,
    };

    api_ok(resp)
}

/// POST /system/user 新增用户。
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SystemUserReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };

    let username = body.username.trim().to_string();
    let nickname = body.nickname.trim().to_string();
    if username.is_empty() || nickname.is_empty() {
        return api_fail("400", "用户名和昵称不能为空");
    }
    if body.dept_id == 0 {
        return api_fail("400", "所属部门不能为空");
    }
    let status = if body.status == 0 { 1 } else { body.status };

    let encrypted_pwd = body.password.unwrap_or_default().trim().to_string();
    if encrypted_pwd.is_empty() {
        return api_fail("400", "密码不能为空");
    }

    let raw_pwd = match state.rsa_decryptor.decrypt_base64(&encrypted_pwd) {
        Ok(p) => p,
        Err(_) => return api_fail("400", "密码解密失败"),
    };
    if raw_pwd.len() < 8 || raw_pwd.len() > 32 {
        return api_fail("400", "密码长度为 8-32 个字符，至少包含字母和数字");
    }
    let mut has_letter = false;
    let mut has_digit = false;
    for ch in raw_pwd.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
        }
        if ch.is_ascii_alphabetic() {
            has_letter = true;
        }
    }
    if !has_letter || !has_digit {
        return api_fail("400", "密码长度为 8-32 个字符，至少包含字母和数字");
    }

    let encoded_pwd = match crate::security::bcrypt_hash(&raw_pwd) {
        Ok(h) => h,
        Err(_) => return api_fail("500", "密码加密失败"),
    };

    let now = Local::now().naive_local();
    let new_id = id::next_id();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "新增用户失败"),
    };

    if sqlx::query(
        r#"
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
        $9, $10, FALSE, $11, $12,
        $13, $14);
"#,
    )
    .bind(new_id)
    .bind(&username)
    .bind(&nickname)
    .bind(&encoded_pwd)
    .bind(body.gender)
    .bind(body.email.trim())
    .bind(body.phone.trim())
    .bind(body.avatar.trim())
    .bind(body.description.trim())
    .bind(status)
    .bind(now)
    .bind(body.dept_id)
    .bind(current_user_id)
    .bind(now)
    .execute(&mut tx)
    .await
    .is_err()
    {
        return api_fail("500", "新增用户失败");
    }

    for rid in body.role_ids.iter() {
        if sqlx::query(
            r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
        )
        .bind(id::next_id())
        .bind(new_id)
        .bind(*rid)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "保存用户角色失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "新增用户失败");
    }

    api_ok(serde_json::json!({ "id": new_id }))
}

/// PUT /system/user/:id 修改用户。
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SystemUserReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let username = body.username.trim().to_string();
    let nickname = body.nickname.trim().to_string();
    if username.is_empty() || nickname.is_empty() {
        return api_fail("400", "用户名和昵称不能为空");
    }
    if body.dept_id == 0 {
        return api_fail("400", "所属部门不能为空");
    }
    let status = if body.status == 0 { 1 } else { body.status };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "修改用户失败"),
    };

    if sqlx::query(
        r#"
UPDATE sys_user
   SET username    = $1,
       nickname    = $2,
       gender      = $3,
       email       = $4,
       phone       = $5,
       avatar      = $6,
       description = $7,
       status      = $8,
       dept_id     = $9,
       update_user = $10,
       update_time = $11
 WHERE id          = $12;
"#,
    )
    .bind(&username)
    .bind(&nickname)
    .bind(body.gender)
    .bind(body.email.trim())
    .bind(body.phone.trim())
    .bind(body.avatar.trim())
    .bind(body.description.trim())
    .bind(status)
    .bind(body.dept_id)
    .bind(current_user_id)
    .bind(Local::now().naive_local())
    .bind(id)
    .execute(&mut tx)
    .await
    .is_err()
    {
        return api_fail("500", "修改用户失败");
    }

    if sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1")
        .bind(id)
        .execute(&mut tx)
        .await
        .is_err()
    {
        return api_fail("500", "修改用户失败");
    }

    for rid in body.role_ids.iter() {
        if sqlx::query(
            r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
        )
        .bind(id::next_id())
        .bind(id)
        .bind(*rid)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "保存用户角色失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "修改用户失败");
    }

    api_ok(serde_json::json!(true))
}

/// DELETE /system/user 删除用户。
pub async fn delete_user(
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
        Err(_) => return api_fail("500", "删除用户失败"),
    };

    for user_id in body.ids.iter() {
        // 不允许删除系统内置用户
        let is_system: bool = match sqlx::query_scalar(
            "SELECT is_system FROM sys_user WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&mut tx)
        .await
        {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(_) => return api_fail("500", "删除用户失败"),
        };

        if is_system {
            continue;
        }

        if sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除用户失败");
        }
        if sqlx::query("DELETE FROM sys_user WHERE id = $1")
            .bind(user_id)
            .execute(&mut tx)
            .await
            .is_err()
        {
            return api_fail("500", "删除用户失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "删除用户失败");
    }

    api_ok(serde_json::json!(true))
}

/// PATCH /system/user/:id/password 重置密码。
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SystemUserPasswordResetReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let enc = body.new_password.trim().to_string();
    if enc.is_empty() {
        return api_fail("400", "密码不能为空");
    }

    let raw_pwd = match state.rsa_decryptor.decrypt_base64(&enc) {
        Ok(p) => p,
        Err(_) => return api_fail("400", "密码解密失败"),
    };
    if raw_pwd.len() < 8 || raw_pwd.len() > 32 {
        return api_fail("400", "密码长度为 8-32 个字符，至少包含字母和数字");
    }
    let mut has_letter = false;
    let mut has_digit = false;
    for ch in raw_pwd.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
        }
        if ch.is_ascii_alphabetic() {
            has_letter = true;
        }
    }
    if !has_letter || !has_digit {
        return api_fail("400", "密码长度为 8-32 个字符，至少包含字母和数字");
    }

    let encoded_pwd = match crate::security::bcrypt_hash(&raw_pwd) {
        Ok(h) => h,
        Err(_) => return api_fail("500", "密码加密失败"),
    };

    if sqlx::query(
        r#"
UPDATE sys_user
   SET password = $1,
       pwd_reset_time = $2,
       update_user = $3,
       update_time = $4
 WHERE id = $5;
"#,
    )
    .bind(encoded_pwd)
    .bind(Local::now().naive_local())
    .bind(current_user_id)
    .bind(Local::now().naive_local())
    .bind(id)
    .execute(&state.db)
    .await
    .is_err()
    {
        return api_fail("500", "重置密码失败");
    }

    api_ok(serde_json::json!(true))
}

/// PATCH /system/user/:id/role 分配角色。
pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SystemUserRoleUpdateReq>,
) -> ApiJson {
    let current_user_id = match current_user_id(&headers, &state.token_svc) {
        Ok(id) => id,
        Err(_) => return api_fail("401", "未授权，请重新登录"),
    };
    let _ = current_user_id;
    if id <= 0 {
        return api_fail("400", "ID 参数不正确");
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return api_fail("500", "分配角色失败"),
    };

    if sqlx::query("DELETE FROM sys_user_role WHERE user_id = $1")
        .bind(id)
        .execute(&mut tx)
        .await
        .is_err()
    {
        return api_fail("500", "分配角色失败");
    }

    for rid in body.role_ids.iter() {
        if sqlx::query(
            r#"
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
"#,
        )
        .bind(id::next_id())
        .bind(id)
        .bind(*rid)
        .execute(&mut tx)
        .await
        .is_err()
        {
            return api_fail("500", "分配角色失败");
        }
    }

    if tx.commit().await.is_err() {
        return api_fail("500", "分配角色失败");
    }

    api_ok(serde_json::json!(true))
}

/// GET /system/user/export 导出用户（CSV）。
pub async fn export_user(State(state): State<Arc<AppState>>) -> Response {
    let rows = match sqlx::query(
        r#"
SELECT username,
       nickname,
       gender,
       COALESCE(email,'') AS email,
       COALESCE(phone,'') AS phone
FROM sys_user
ORDER BY id;
"#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            let json = api_fail("500", "导出用户失败");
            return json.into_response();
        }
    };

    let mut content = String::from("username,nickname,gender,email,phone\n");
    for r in rows {
        let username: String = r.get("username");
        let nickname: String = r.get("nickname");
        let gender: i16 = r.get("gender");
        let email: String = r.get("email");
        let phone: String = r.get("phone");
        let line =
            format!("{},{},{},{},{}\n", username, nickname, gender, email, phone);
        content.push_str(&line);
    }

    let mut resp = content.into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"users.csv\""
            .parse()
            .unwrap(),
    );
    resp
}

/// GET /system/user/import/template 下载导入模板。
pub async fn download_import_template() -> Response {
    let content = "username,nickname,gender,email,phone\n".to_string();
    let mut resp = content.into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"user_import_template.csv\""
            .parse()
            .unwrap(),
    );
    resp
}

/// POST /system/user/import/parse 解析导入（占位）。
pub async fn parse_import_user(body: Bytes) -> ApiJson {
    if body.is_empty() {
        return api_fail("400", "文件不能为空");
    }
    let resp = UserImportParseResp {
        import_key: format!(
            "{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        total_rows: 0,
        valid_rows: 0,
        duplicate_user_rows: 0,
        duplicate_email_rows: 0,
        duplicate_phone_rows: 0,
    };
    api_ok(resp)
}

/// POST /system/user/import 导入用户（占位实现）。
pub async fn import_user(Json(_body): Json<serde_json::Value>) -> ApiJson {
    let resp = UserImportResultResp {
        total_rows: 0,
        insert_rows: 0,
        update_rows: 0,
    };
    api_ok(resp)
}

