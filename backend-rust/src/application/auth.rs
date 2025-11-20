use chrono::{DateTime, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// 登录请求体结构，对齐前端与 Go 版 LoginRequest。
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "authType")]
    pub auth_type: String,
    pub username: String,
    /// 使用 RSA+Base64 加密后的密码
    pub password: String,
    pub captcha: String,
    pub uuid: String,
}

/// 登录响应结构，对齐 LoginResponse。
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// /auth/user/info 返回的用户信息结构，对齐 UserInfo。
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub gender: i16,
    pub email: String,
    pub phone: String,
    pub avatar: String,
    pub description: String,
    #[serde(rename = "pwdResetTime")]
    pub pwd_reset_time: String,
    #[serde(rename = "pwdExpired")]
    pub pwd_expired: bool,
    #[serde(rename = "registrationDate")]
    pub registration_date: String,
    #[serde(rename = "deptName")]
    pub dept_name: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// 路由树节点结构，对齐前端 RouteItem。
#[derive(Debug, Serialize)]
pub struct RouteItem {
    pub id: i64,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "parentId")]
    pub parent_id: i64,
    #[serde(rename = "type")]
    pub menu_type: i16,
    pub path: String,
    pub name: String,
    pub component: String,
    pub redirect: String,
    pub icon: String,
    #[serde(rename = "isExternal")]
    pub is_external: bool,
    #[serde(rename = "isHidden")]
    pub is_hidden: bool,
    #[serde(rename = "isCache")]
    pub is_cache: bool,
    pub permission: String,
    pub roles: Vec<String>,
    pub sort: i32,
    pub status: i16,
    pub children: Vec<RouteItem>,
    #[serde(rename = "activeMenu")]
    pub active_menu: String,
    #[serde(rename = "alwaysShow")]
    pub always_show: bool,
    pub breadcrumb: bool,
    #[serde(rename = "showInTabs")]
    pub show_in_tabs: bool,
    pub affix: bool,
}

/// 根据 Claim 构造当前用户 ID。
pub fn user_id_from_claims(claims: &Claims) -> i64 {
    claims.user_id
}

/// 将数据库中的时间字段格式化为前端需要的字符串格式。
pub fn format_time(ts: NaiveDateTime) -> String {
    let dt: DateTime<Local> = DateTime::from_utc(ts, *Local::now().offset());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
