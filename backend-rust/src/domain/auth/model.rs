use chrono::NaiveDateTime;

pub const ADMIN_ROLE_CODE: &str = "admin";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleContext {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub data_scope: i16,
}

impl RoleContext {
    pub fn is_admin(&self) -> bool {
        self.code == ADMIN_ROLE_CODE
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub dept_id: i64,
    pub roles: Vec<RoleContext>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserAccount {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub password_hash: Option<String>,
    pub gender: i16,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub status: i16,
    pub pwd_reset_time: Option<NaiveDateTime>,
    pub dept_id: i64,
    pub dept_name: String,
    pub create_time: NaiveDateTime,
}
