use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use sqlx::PgPool;

use crate::{
    application::{
        data_scope::resolver::{resolve_data_scope, DataScopeContext},
        system::{ensure_max_chars, format_datetime, format_optional_datetime, trim_to_none},
    },
    domain::{
        auth::model::CurrentUser,
        data_scope::model::{DataPermissionTarget, DataScopeFilter},
    },
    infrastructure::{
        persistence::{
            dept_repository::DeptRepository,
            system_user_repository::{
                SystemUserRepository, UserCreateRecord, UserListFilter, UserRecord,
                UserUpdateRecord,
            },
        },
        security::password::hash_password,
    },
    shared::{
        error::AppError,
        id::next_id,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<i16>,
    #[serde(
        default,
        alias = "createTime[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub create_time: Vec<String>,
    #[serde(default)]
    pub dept_id: Option<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
    #[serde(
        default,
        alias = "userIds[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub role_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCommand {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub gender: i16,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: i16,
    #[serde(default)]
    pub dept_id: i64,
    #[serde(default, deserialize_with = "deserialize_id_vec")]
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetCommand {
    #[serde(default)]
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleCommand {
    #[serde(default, deserialize_with = "deserialize_id_vec")]
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResp {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub gender: i16,
    pub email: String,
    pub phone: String,
    pub description: String,
    pub status: i16,
    pub is_system: bool,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
    pub dept_id: i64,
    pub dept_name: String,
    pub role_ids: Vec<i64>,
    pub role_names: Vec<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailResp {
    #[serde(flatten)]
    pub user: UserResp,
    pub pwd_reset_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserImportResp {
    pub import_key: String,
    pub total_rows: usize,
    pub valid_rows: usize,
    pub duplicate_user_rows: usize,
    pub duplicate_email_rows: usize,
    pub duplicate_phone_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserImportResultResp {
    pub total_rows: usize,
    pub insert_rows: usize,
    pub update_rows: usize,
}

#[derive(Debug, Clone)]
pub struct UserService {
    users: SystemUserRepository,
    depts: DeptRepository,
}

#[derive(Debug, Clone)]
struct NormalizedUserQuery {
    page: PageQuery,
    description: Option<String>,
    status: Option<i16>,
    create_time_start: Option<NaiveDateTime>,
    create_time_end: Option<NaiveDateTime>,
    dept_id: Option<i64>,
    user_ids: Vec<i64>,
    role_id: Option<i64>,
    order_by: String,
}

impl UserService {
    pub fn new(db: PgPool) -> Self {
        Self {
            users: SystemUserRepository::new(db.clone()),
            depts: DeptRepository::new(db),
        }
    }

    pub async fn page(
        &self,
        current_user: &CurrentUser,
        query: UserQuery,
    ) -> Result<PageResult<UserResp>, AppError> {
        let query = normalize_user_query(query)?;
        let data_scope = self.resolve_data_scope(current_user).await?;
        let filter = query.to_filter(
            &data_scope,
            Some(query.page.limit()),
            Some(query.page.offset()),
        );
        let total = self.users.count(&filter).await?;
        let list = self
            .users
            .list(&filter)
            .await?
            .into_iter()
            .map(UserResp::from)
            .collect();

        Ok(PageResult::new(list, total))
    }

    pub async fn list(
        &self,
        current_user: &CurrentUser,
        query: UserQuery,
    ) -> Result<Vec<UserResp>, AppError> {
        let query = normalize_user_query(query)?;
        let data_scope = self.resolve_data_scope(current_user).await?;
        let filter = query.to_filter(&data_scope, None, None);
        Ok(self
            .users
            .list(&filter)
            .await?
            .into_iter()
            .map(UserResp::from)
            .collect())
    }

    pub async fn list_for_export(
        &self,
        current_user: &CurrentUser,
        query: UserQuery,
    ) -> Result<Vec<UserResp>, AppError> {
        self.list(current_user, query).await
    }

    pub async fn get(&self, id: i64) -> Result<UserDetailResp, AppError> {
        self.users
            .get(id)
            .await?
            .map(UserDetailResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn create(
        &self,
        current_user: &CurrentUser,
        command: UserCommand,
    ) -> Result<i64, AppError> {
        let command = normalize_user_command(command)?;
        ensure_password_present(&command.password)?;
        ensure_unique_user_fields(&self.users, &command, None).await?;
        ensure_dept_exists(&self.users, command.dept_id).await?;
        ensure_role_ids_assignable(&self.users, &command.role_ids).await?;

        let id = next_id();
        self.users
            .create(&UserCreateRecord {
                id,
                username: command.username,
                nickname: command.nickname,
                password_hash: hash_password(&command.password)?,
                gender: command.gender,
                email: trim_to_none(command.email),
                phone: trim_to_none(command.phone),
                avatar: trim_to_none(command.avatar),
                description: trim_to_none(command.description),
                status: command.status,
                dept_id: command.dept_id,
                role_ids: command.role_ids,
                user_id: current_user.id,
                now: Utc::now().naive_utc(),
            })
            .await?;

        Ok(id)
    }

    pub async fn update(
        &self,
        current_user: &CurrentUser,
        id: i64,
        command: UserCommand,
    ) -> Result<(), AppError> {
        let command = normalize_user_command(command)?;
        let existing = self.users.get(id).await?.ok_or(AppError::NotFound)?;
        if existing.is_system && existing.username != command.username {
            return Err(AppError::bad_request("系统内置用户用户名不允许修改"));
        }
        if existing.is_system && existing.role_ids.contains(&1) {
            return Err(AppError::bad_request("系统管理员用户角色不允许修改"));
        }
        ensure_unique_user_fields(&self.users, &command, Some(id)).await?;
        ensure_dept_exists(&self.users, command.dept_id).await?;
        ensure_role_ids_assignable(&self.users, &command.role_ids).await?;

        self.users
            .update(&UserUpdateRecord {
                id,
                username: command.username,
                nickname: command.nickname,
                gender: command.gender,
                email: trim_to_none(command.email),
                phone: trim_to_none(command.phone),
                avatar: trim_to_none(command.avatar),
                description: trim_to_none(command.description),
                status: command.status,
                dept_id: command.dept_id,
                role_ids: command.role_ids,
                user_id: current_user.id,
                now: Utc::now().naive_utc(),
            })
            .await
    }

    pub async fn delete(&self, current_user: &CurrentUser, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalize_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        if ids.contains(&current_user.id) {
            return Err(AppError::bad_request("当前登录用户不允许删除"));
        }

        let mut system_usernames = Vec::new();
        for id in &ids {
            if let Some(user) = self.users.get(*id).await? {
                if user.is_system {
                    system_usernames.push(user.username);
                }
            }
        }
        if let Some(username) = system_usernames.first() {
            return Err(AppError::bad_request(format!(
                "所选用户 [{username}] 是系统内置用户，不允许删除"
            )));
        }

        self.users.delete_many(&ids).await
    }

    pub async fn reset_password(
        &self,
        current_user: &CurrentUser,
        id: i64,
        command: PasswordResetCommand,
    ) -> Result<(), AppError> {
        let password = command.new_password.trim();
        ensure_password_present(password)?;
        let hash = hash_password(password)?;
        self.users
            .update_password(id, &hash, current_user.id, Utc::now().naive_utc())
            .await
    }

    pub async fn update_role(&self, id: i64, command: UserRoleCommand) -> Result<(), AppError> {
        let role_ids = normalize_ids(command.role_ids);
        let existing = self.users.get(id).await?.ok_or(AppError::NotFound)?;
        if existing.is_system && existing.role_ids.contains(&1) {
            return Err(AppError::bad_request("系统管理员用户角色不允许修改"));
        }
        ensure_role_ids_assignable(&self.users, &role_ids).await?;
        self.users.replace_roles(id, &role_ids).await
    }

    pub fn parse_import(&self, content: &[u8]) -> UserImportResp {
        let text = String::from_utf8_lossy(content);
        let total_rows = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
            .saturating_sub(1);

        UserImportResp {
            import_key: uuid::Uuid::new_v4().to_string(),
            total_rows,
            valid_rows: total_rows,
            duplicate_user_rows: 0,
            duplicate_email_rows: 0,
            duplicate_phone_rows: 0,
        }
    }

    pub fn import_users(&self) -> UserImportResultResp {
        UserImportResultResp {
            total_rows: 0,
            insert_rows: 0,
            update_rows: 0,
        }
    }

    async fn resolve_data_scope(
        &self,
        current_user: &CurrentUser,
    ) -> Result<DataScopeFilter, AppError> {
        let role_ids = current_user
            .roles
            .iter()
            .map(|role| role.id)
            .collect::<Vec<_>>();
        let context = DataScopeContext {
            dept_tree: self.depts.enabled_dept_tree().await?,
            role_dept_scope: self.depts.role_dept_scope(&role_ids).await?,
        };

        resolve_data_scope(
            current_user,
            &DataPermissionTarget {
                dept_column: Some("u.dept_id"),
                user_column: Some("u.create_user"),
            },
            &context,
        )
    }
}

impl NormalizedUserQuery {
    fn to_filter<'a>(
        &'a self,
        data_scope: &'a DataScopeFilter,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> UserListFilter<'a> {
        UserListFilter {
            description: self.description.as_deref(),
            status: self.status,
            create_time_start: self.create_time_start,
            create_time_end: self.create_time_end,
            dept_id: self.dept_id,
            user_ids: &self.user_ids,
            role_id: self.role_id,
            data_scope,
            order_by: &self.order_by,
            limit,
            offset,
        }
    }
}

impl From<UserRecord> for UserResp {
    fn from(record: UserRecord) -> Self {
        Self {
            id: record.id,
            username: record.username,
            nickname: record.nickname,
            avatar: record.avatar,
            gender: record.gender,
            email: record.email,
            phone: record.phone,
            description: record.description,
            status: record.status,
            is_system: record.is_system,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
            dept_id: record.dept_id,
            dept_name: record.dept_name,
            role_ids: record.role_ids,
            role_names: record.role_names,
            disabled: record.is_system,
        }
    }
}

impl From<UserRecord> for UserDetailResp {
    fn from(record: UserRecord) -> Self {
        let pwd_reset_time = format_optional_datetime(record.pwd_reset_time);
        Self {
            user: UserResp::from(record),
            pwd_reset_time,
        }
    }
}

pub fn user_sort_sql(sort: &[String]) -> String {
    let mut clauses = sort
        .iter()
        .filter_map(|value| parse_sort_clause(value))
        .collect::<Vec<_>>();

    if clauses.is_empty() {
        clauses.push("u.create_time DESC".to_owned());
    }
    if !clauses.iter().any(|clause| clause.starts_with("u.id ")) {
        clauses.push("u.id DESC".to_owned());
    }
    clauses.join(", ")
}

pub fn normalize_user_command(mut command: UserCommand) -> Result<UserCommand, AppError> {
    command.username = command.username.trim().to_owned();
    command.nickname = command.nickname.trim().to_owned();
    command.password = command.password.trim().to_owned();
    command.email = command.email.trim().to_owned();
    command.phone = command.phone.trim().to_owned();
    command.avatar = command.avatar.trim().to_owned();
    command.description = command.description.trim().to_owned();
    command.role_ids = normalize_ids(command.role_ids);

    if command.username.is_empty() {
        return Err(AppError::bad_request("用户名不能为空"));
    }
    if command.nickname.is_empty() {
        return Err(AppError::bad_request("昵称不能为空"));
    }
    if command.dept_id <= 0 {
        return Err(AppError::bad_request("所属部门不能为空"));
    }
    if command.status == 0 {
        command.status = 1;
    }
    if command.status != 1 && command.status != 2 {
        return Err(AppError::bad_request("用户状态不正确"));
    }
    if !(0..=2).contains(&command.gender) {
        return Err(AppError::bad_request("性别不正确"));
    }

    ensure_max_chars("用户名", &command.username, 64)?;
    ensure_max_chars("昵称", &command.nickname, 30)?;
    ensure_max_chars("邮箱", &command.email, 255)?;
    ensure_max_chars("手机号", &command.phone, 255)?;
    ensure_max_chars("头像", &command.avatar, 512)?;
    ensure_max_chars("描述", &command.description, 200)?;

    Ok(command)
}

pub fn ensure_user_role_ids_can_be_assigned(role_ids: &[i64]) -> Result<(), AppError> {
    if role_ids.iter().any(|role_id| *role_id <= 0) {
        return Err(AppError::bad_request("角色ID参数不正确"));
    }
    if role_ids.contains(&1) {
        return Err(AppError::bad_request("系统管理员角色不允许分配"));
    }
    Ok(())
}

async fn ensure_unique_user_fields(
    users: &SystemUserRepository,
    command: &UserCommand,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    if users.username_exists(&command.username, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.username
        )));
    }
    if !command.email.is_empty() && users.email_exists(&command.email, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.email
        )));
    }
    if !command.phone.is_empty() && users.phone_exists(&command.phone, exclude_id).await? {
        return Err(AppError::bad_request(format!(
            "保存失败，[{}] 已存在",
            command.phone
        )));
    }
    Ok(())
}

async fn ensure_dept_exists(users: &SystemUserRepository, dept_id: i64) -> Result<(), AppError> {
    if !users.dept_exists(dept_id).await? {
        return Err(AppError::bad_request("所属部门不存在"));
    }
    Ok(())
}

async fn ensure_role_ids_assignable(
    users: &SystemUserRepository,
    role_ids: &[i64],
) -> Result<(), AppError> {
    ensure_user_role_ids_can_be_assigned(role_ids)?;
    if users.role_ids_contain_admin(role_ids).await? {
        return Err(AppError::bad_request("系统管理员角色不允许分配"));
    }
    let missing = users.missing_role_ids(role_ids).await?;
    if let Some(role_id) = missing.first() {
        return Err(AppError::bad_request(format!("角色 [{role_id}] 不存在")));
    }
    Ok(())
}

fn ensure_password_present(password: &str) -> Result<(), AppError> {
    if password.trim().is_empty() {
        return Err(AppError::bad_request("密码不能为空"));
    }
    ensure_max_chars("密码", password, 128)
}

fn normalize_user_query(query: UserQuery) -> Result<NormalizedUserQuery, AppError> {
    let create_time_start = query
        .create_time
        .first()
        .map(|value| parse_start_datetime(value))
        .transpose()?;
    let create_time_end = query
        .create_time
        .get(1)
        .map(|value| parse_end_datetime(value))
        .transpose()?;
    let dept_id = query
        .dept_id
        .as_deref()
        .map(parse_optional_positive_i64)
        .transpose()?;
    let role_id = query
        .role_id
        .as_deref()
        .map(parse_optional_positive_i64)
        .transpose()?;

    Ok(NormalizedUserQuery {
        page: PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized(),
        description: query
            .description
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty()),
        status: query.status.filter(|status| *status > 0),
        create_time_start,
        create_time_end,
        dept_id,
        user_ids: parse_query_ids(&query.user_ids)?,
        role_id,
        order_by: user_sort_sql(&query.sort),
    })
}

fn parse_sort_clause(value: &str) -> Option<String> {
    let mut parts = value.split(',');
    let field = parts.next()?.trim();
    let direction = parts.next().unwrap_or("asc").trim();
    if parts.next().is_some() {
        return None;
    }

    let field = field.strip_prefix("t1.").unwrap_or(field);
    let column = match field {
        "id" => "u.id",
        "username" => "u.username",
        "nickname" => "u.nickname",
        "status" => "u.status",
        "deptId" => "u.dept_id",
        "createTime" => "u.create_time",
        "updateTime" => "u.update_time",
        _ => return None,
    };
    let direction = if direction.eq_ignore_ascii_case("desc") {
        "DESC"
    } else if direction.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        return None;
    };

    Some(format!("{column} {direction}"))
}

fn parse_query_ids(values: &[String]) -> Result<Vec<i64>, AppError> {
    let mut ids = Vec::new();
    for value in values {
        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            ids.push(
                part.parse::<i64>()
                    .map_err(|_| AppError::bad_request("ID 参数不正确"))?,
            );
        }
    }
    Ok(normalize_ids(ids))
}

fn parse_optional_positive_i64(value: &str) -> Result<i64, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(0);
    }
    value
        .parse::<i64>()
        .map_err(|_| AppError::bad_request("ID 参数不正确"))
}

fn parse_start_datetime(value: &str) -> Result<NaiveDateTime, AppError> {
    parse_datetime(value, false)
}

fn parse_end_datetime(value: &str) -> Result<NaiveDateTime, AppError> {
    parse_datetime(value, true)
}

fn parse_datetime(value: &str, end_of_day: bool) -> Result<NaiveDateTime, AppError> {
    let value = value.trim();
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(datetime);
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::bad_request("创建时间格式不正确"))?;
    if end_of_day {
        date.and_hms_opt(23, 59, 59)
    } else {
        date.and_hms_opt(0, 0, 0)
    }
    .ok_or_else(|| AppError::bad_request("创建时间格式不正确"))
}

fn normalize_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut ids = ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn deserialize_id_vec<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<IdValue>::deserialize(deserializer)?;
    values
        .into_iter()
        .map(IdValue::into_i64)
        .collect::<Result<Vec<_>, D::Error>>()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IdValue {
    Number(i64),
    String(String),
}

impl IdValue {
    fn into_i64<E>(self) -> Result<i64, E>
    where
        E: de::Error,
    {
        match self {
            Self::Number(value) => Ok(value),
            Self::String(value) => value
                .trim()
                .parse::<i64>()
                .map_err(|_| E::custom("ID must be an integer string or number")),
        }
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_size() -> u64 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn role_ids_accept_string_and_number_values() {
        let command: UserRoleCommand =
            serde_json::from_value(json!({"roleIds": ["2", 3]})).unwrap();

        assert_eq!(command.role_ids, vec![2, 3]);
    }

    #[test]
    fn sort_sql_ignores_unknown_fields() {
        assert_eq!(
            user_sort_sql(&["bad,desc".to_owned(), "id,asc".to_owned()]),
            "u.id ASC"
        );
    }

    #[test]
    fn sort_sql_accepts_vue_table_alias_prefix() {
        assert_eq!(
            user_sort_sql(&["t1.createTime,desc".to_owned(), "t1.id,desc".to_owned()]),
            "u.create_time DESC, u.id DESC"
        );
    }

    #[test]
    fn query_ids_accept_repeated_and_comma_separated_values() {
        assert_eq!(
            parse_query_ids(&["1,2".to_owned(), "3".to_owned()]).unwrap(),
            vec![1, 2, 3]
        );
    }
}
