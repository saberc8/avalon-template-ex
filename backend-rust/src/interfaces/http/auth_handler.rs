use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use sqlx::Row;

use crate::application::auth::{LoginRequest, LoginResponse};
use crate::security::PasswordVerifier;
use crate::AppState;

use super::response::{api_fail, api_ok, ApiJson};

/// POST /auth/login
/// 复用 Java/Go 的认证逻辑：RSA 解密 + BCrypt 校验 + JWT 生成。
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> ApiJson {
    let auth_type = req.auth_type.trim().to_uppercase();
    if !auth_type.is_empty() && auth_type != "ACCOUNT" {
        return api_fail("400", "暂不支持该认证方式");
    }
    if req.client_id.trim().is_empty() {
        return api_fail("400", "客户端ID不能为空");
    }
    if req.username.trim().is_empty() {
        return api_fail("400", "用户名不能为空");
    }
    if req.password.trim().is_empty() {
        return api_fail("400", "密码不能为空");
    }

    // 1. RSA 解密前端传入的密码
    let raw_password = match state.rsa_decryptor.decrypt_base64(&req.password) {
        Ok(p) => p,
        Err(_) => return api_fail("400", "密码解密失败"),
    };

    // 2. 从 sys_user 查询用户信息
    let row = match sqlx::query(
        r#"
SELECT id, username, nickname, password, status
FROM sys_user
WHERE username = $1
LIMIT 1;
"#,
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => return api_fail("500", "查询用户失败"),
    };

    let row = match row {
        Some(r) => r,
        None => return api_fail("400", "用户名或密码不正确"),
    };

    let user_id: i64 = row.get("id");
    let stored_pwd: String = row.get("password");
    let status: i16 = row.get("status");

    // 3. BCrypt 校验密码
    let ok = match state.pwd_verifier.verify(&raw_password, &stored_pwd) {
        Ok(ok) => ok,
        Err(_) => return api_fail("500", "密码校验失败"),
    };
    if !ok {
        return api_fail("400", "用户名或密码不正确");
    }

    // 4. 账号状态校验（1：启用）
    if status != 1 {
        return api_fail("400", "此账号已被禁用，如有疑问，请联系管理员");
    }

    // 5. 生成 JWT Token
    let token = match state.token_svc.generate(user_id) {
        Ok(t) => t,
        Err(_) => return api_fail("500", "生成令牌失败"),
    };

    let resp = LoginResponse { token };
    api_ok(resp)
}

