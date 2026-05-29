use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::shared::{error::AppError, id::next_id};

#[derive(Debug, Clone)]
pub struct SystemMiscRepository {
    db: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct DictRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: String,
    pub is_system: bool,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DictItemRecord {
    pub id: i64,
    pub label: String,
    pub value: String,
    pub color: String,
    pub sort: i32,
    pub description: String,
    pub status: i16,
    pub dict_id: i64,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct OptionRecord {
    pub id: i64,
    pub category: String,
    pub name: String,
    pub code: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct StorageRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub storage_type: i16,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub region: String,
    pub bucket_name: String,
    pub domain: String,
    pub description: String,
    pub is_default: bool,
    pub sort: i32,
    pub status: i16,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClientRecord {
    pub id: i64,
    pub client_id: String,
    pub client_type: String,
    pub auth_type: Value,
    pub active_timeout: i64,
    pub timeout: i64,
    pub status: i16,
    pub create_user: i64,
    pub create_time: NaiveDateTime,
    pub update_user: Option<i64>,
    pub update_time: Option<NaiveDateTime>,
    pub create_user_string: String,
    pub update_user_string: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub name: String,
    pub original_name: String,
    pub size: i64,
    pub parent_path: String,
    pub path: String,
    pub sha256: String,
    pub content_type: String,
    pub metadata: String,
    pub thumbnail_size: i64,
    pub thumbnail_name: String,
    pub thumbnail_metadata: String,
    pub extension: String,
    pub file_type: i16,
    pub storage_id: i64,
    pub storage_name: String,
    pub storage_domain: String,
    pub create_time: NaiveDateTime,
    pub create_user_string: String,
    pub update_time: Option<NaiveDateTime>,
    pub update_user_string: String,
}

#[derive(Debug, Clone)]
pub struct DictFilter<'a> {
    pub description: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct DictItemFilter<'a> {
    pub dict_id: Option<i64>,
    pub description: Option<&'a str>,
    pub status: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct StorageFilter<'a> {
    pub description: Option<&'a str>,
    pub storage_type: Option<i16>,
}

#[derive(Debug, Clone)]
pub struct ClientFilter<'a> {
    pub client_type: Option<&'a str>,
    pub status: Option<i16>,
    pub auth_types: &'a [String],
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct FileFilter<'a> {
    pub original_name: Option<&'a str>,
    pub file_type: Option<i16>,
    pub parent_path: Option<&'a str>,
    pub order_by: &'a str,
    pub limit: i64,
    pub offset: i64,
}

impl SystemMiscRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_dicts(&self, filter: &DictFilter<'_>) -> Result<Vec<DictRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dict_select_sql());
        query.push(" WHERE 1 = 1");
        if let Some(description) = non_empty(filter.description) {
            let pattern = format!("%{description}%");
            query
                .push(" AND (d.name ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR d.code ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR COALESCE(d.description, '') ILIKE ")
                .push_bind(pattern)
                .push(")");
        }
        query.push(" ORDER BY d.create_time DESC, d.id DESC");
        Ok(query
            .build_query_as::<DictRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_dict(&self, id: i64) -> Result<Option<DictRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dict_select_sql());
        query.push(" WHERE d.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<DictRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn create_dict(
        &self,
        id: i64,
        name: &str,
        code: &str,
        description: Option<&str>,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
VALUES ($1, $2, $3, $4, FALSE, $5, $6);
"#,
        )
        .bind(id)
        .bind(name)
        .bind(code)
        .bind(description)
        .bind(user_id)
        .bind(now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_dict(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_dict
SET name = $1, description = $2, update_user = $3, update_time = $4
WHERE id = $5;
"#,
        )
        .bind(name)
        .bind(description)
        .bind(user_id)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn delete_dicts(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.db.begin().await?;
        sqlx::query("DELETE FROM sys_dict_item WHERE dict_id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sys_dict WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn first_system_dict_name(&self, ids: &[i64]) -> Result<Option<String>, AppError> {
        if ids.is_empty() {
            return Ok(None);
        }
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT name FROM sys_dict WHERE id = ANY($1) AND is_system = TRUE ORDER BY id ASC LIMIT 1;",
        )
        .bind(ids.to_vec())
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn dict_name_exists(
        &self,
        name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists("sys_dict", "name", name, exclude_id).await
    }

    pub async fn dict_code_exists(
        &self,
        code: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists("sys_dict", "code", code, exclude_id).await
    }

    pub async fn count_dict_items(&self, filter: &DictItemFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_dict_item AS di");
        query.push(" WHERE 1 = 1");
        push_dict_item_filters(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_dict_items(
        &self,
        filter: &DictItemFilter<'_>,
    ) -> Result<Vec<DictItemRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dict_item_select_sql());
        query.push(" WHERE 1 = 1");
        push_dict_item_filters(&mut query, filter);
        query
            .push(" ORDER BY di.sort ASC, di.id ASC LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);
        Ok(query
            .build_query_as::<DictItemRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_dict_item(&self, id: i64) -> Result<Option<DictItemRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(dict_item_select_sql());
        query.push(" WHERE di.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<DictItemRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn create_dict_item(&self, record: &DictItemSaveRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status, dict_id, create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
"#,
        )
        .bind(record.id)
        .bind(record.label)
        .bind(record.value)
        .bind(record.color)
        .bind(record.sort)
        .bind(record.description)
        .bind(record.status)
        .bind(record.dict_id)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_dict_item(&self, record: &DictItemSaveRecord<'_>) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_dict_item
SET label = $1,
    value = $2,
    color = $3,
    sort = $4,
    description = $5,
    status = $6,
    dict_id = $7,
    update_user = $8,
    update_time = $9
WHERE id = $10;
"#,
        )
        .bind(record.label)
        .bind(record.value)
        .bind(record.color)
        .bind(record.sort)
        .bind(record.description)
        .bind(record.status)
        .bind(record.dict_id)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn delete_dict_items(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM sys_dict_item WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn dict_exists(&self, id: i64) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_dict WHERE id = $1);")
                .bind(id)
                .fetch_one(&self.db)
                .await?,
        )
    }

    pub async fn dict_item_value_exists(
        &self,
        dict_id: i64,
        value: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            "SELECT EXISTS(SELECT 1 FROM sys_dict_item WHERE dict_id = ",
        );
        query
            .push_bind(dict_id)
            .push(" AND value = ")
            .push_bind(value);
        if let Some(exclude_id) = exclude_id {
            query.push(" AND id <> ").push_bind(exclude_id);
        }
        query.push(")");
        Ok(query
            .build_query_scalar::<bool>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_options(
        &self,
        codes: &[String],
        category: Option<&str>,
    ) -> Result<Vec<OptionRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(
            r#"
SELECT
    id,
    category,
    name,
    code,
    COALESCE(value, default_value, '') AS value,
    COALESCE(description, '') AS description
FROM sys_option
WHERE 1 = 1
"#,
        );
        if !codes.is_empty() {
            query.push(" AND code IN (");
            let mut separated = query.separated(", ");
            for code in codes {
                separated.push_bind(code);
            }
            separated.push_unseparated(")");
        }
        if let Some(category) = non_empty(category) {
            query.push(" AND category = ").push_bind(category);
        }
        query.push(" ORDER BY id ASC");
        Ok(query
            .build_query_as::<OptionRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn update_options(
        &self,
        values: &[OptionUpdateRecord],
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        for value in values {
            let result = sqlx::query(
                r#"
UPDATE sys_option
SET value = $1, update_user = $2, update_time = $3
WHERE id = $4 AND code = $5;
"#,
            )
            .bind(&value.value)
            .bind(user_id)
            .bind(now)
            .bind(value.id)
            .bind(&value.code)
            .execute(&mut *tx)
            .await?;
            ensure_affected(result.rows_affected())?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn reset_options(
        &self,
        codes: &[String],
        category: Option<&str>,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let mut query =
            QueryBuilder::<Postgres>::new("UPDATE sys_option SET value = NULL, update_user = ");
        query
            .push_bind(user_id)
            .push(", update_time = ")
            .push_bind(now)
            .push(" WHERE 1 = 1");
        if !codes.is_empty() {
            query.push(" AND code IN (");
            let mut separated = query.separated(", ");
            for code in codes {
                separated.push_bind(code);
            }
            separated.push_unseparated(")");
        }
        if let Some(category) = non_empty(category) {
            query.push(" AND category = ").push_bind(category);
        }
        query.build().execute(&self.db).await?;
        Ok(())
    }

    pub async fn list_storages(
        &self,
        filter: &StorageFilter<'_>,
    ) -> Result<Vec<StorageRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(storage_select_sql());
        query.push(" WHERE 1 = 1");
        if let Some(description) = non_empty(filter.description) {
            let pattern = format!("%{description}%");
            query
                .push(" AND (s.name ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR s.code ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR COALESCE(s.description, '') ILIKE ")
                .push_bind(pattern)
                .push(")");
        }
        if let Some(storage_type) = filter.storage_type.filter(|value| *value > 0) {
            query.push(" AND s.type = ").push_bind(storage_type);
        }
        query.push(" ORDER BY s.sort ASC, s.id ASC");
        Ok(query
            .build_query_as::<StorageRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_storage(&self, id: i64) -> Result<Option<StorageRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(storage_select_sql());
        query.push(" WHERE s.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<StorageRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn default_storage(&self) -> Result<Option<StorageRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(storage_select_sql());
        query.push(" WHERE s.is_default = TRUE ORDER BY s.id ASC LIMIT 1");
        Ok(query
            .build_query_as::<StorageRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn create_storage(&self, record: &StorageSaveRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_storage (
    id, name, code, type, access_key, secret_key, endpoint, region, bucket_name,
    domain, description, is_default, sort, status, create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16);
"#,
        )
        .bind(record.id)
        .bind(record.name)
        .bind(record.code)
        .bind(record.storage_type)
        .bind(record.access_key)
        .bind(record.secret_key)
        .bind(record.endpoint)
        .bind(record.region)
        .bind(record.bucket_name)
        .bind(record.domain)
        .bind(record.description)
        .bind(record.is_default)
        .bind(record.sort)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        if record.is_default {
            self.set_default_storage(record.id).await?;
        }
        Ok(())
    }

    pub async fn update_storage(&self, record: &StorageSaveRecord<'_>) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_storage
SET name = $1,
    code = $2,
    type = $3,
    access_key = $4,
    secret_key = $5,
    endpoint = $6,
    region = $7,
    bucket_name = $8,
    domain = $9,
    description = $10,
    is_default = $11,
    sort = $12,
    status = $13,
    update_user = $14,
    update_time = $15
WHERE id = $16;
"#,
        )
        .bind(record.name)
        .bind(record.code)
        .bind(record.storage_type)
        .bind(record.access_key)
        .bind(record.secret_key)
        .bind(record.endpoint)
        .bind(record.region)
        .bind(record.bucket_name)
        .bind(record.domain)
        .bind(record.description)
        .bind(record.is_default)
        .bind(record.sort)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())?;
        if record.is_default {
            self.set_default_storage(record.id).await?;
        }
        Ok(())
    }

    pub async fn storage_code_exists(
        &self,
        code: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        self.exists("sys_storage", "code", code, exclude_id).await
    }

    pub async fn delete_storages(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM sys_storage WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn storage_has_files(&self, ids: &[i64]) -> Result<bool, AppError> {
        if ids.is_empty() {
            return Ok(false);
        }
        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM sys_file WHERE storage_id = ANY($1));",
        )
        .bind(ids.to_vec())
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn update_storage_status(
        &self,
        id: i64,
        status: i16,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE sys_storage SET status = $1, update_user = $2, update_time = $3 WHERE id = $4;",
        )
        .bind(status)
        .bind(user_id)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn set_default_storage(&self, id: i64) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        sqlx::query("UPDATE sys_storage SET is_default = FALSE;")
            .execute(&mut *tx)
            .await?;
        let result = sqlx::query("UPDATE sys_storage SET is_default = TRUE WHERE id = $1;")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn count_clients(&self, filter: &ClientFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_client AS c");
        query.push(" WHERE 1 = 1");
        push_client_filters(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_clients(
        &self,
        filter: &ClientFilter<'_>,
    ) -> Result<Vec<ClientRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(client_select_sql());
        query.push(" WHERE 1 = 1");
        push_client_filters(&mut query, filter);
        query
            .push(" ORDER BY c.id DESC LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);
        Ok(query
            .build_query_as::<ClientRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_client(&self, id: i64) -> Result<Option<ClientRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(client_select_sql());
        query.push(" WHERE c.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<ClientRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn create_client(&self, record: &ClientSaveRecord) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_client (
    id, client_id, client_type, auth_type, active_timeout, timeout, status,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);
"#,
        )
        .bind(record.id)
        .bind(&record.client_id)
        .bind(&record.client_type)
        .bind(&record.auth_type)
        .bind(record.active_timeout)
        .bind(record.timeout)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_client(&self, record: &ClientSaveRecord) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_client
SET client_type = $1,
    auth_type = $2,
    active_timeout = $3,
    timeout = $4,
    status = $5,
    update_user = $6,
    update_time = $7
WHERE id = $8;
"#,
        )
        .bind(&record.client_type)
        .bind(&record.auth_type)
        .bind(record.active_timeout)
        .bind(record.timeout)
        .bind(record.status)
        .bind(record.user_id)
        .bind(record.now)
        .bind(record.id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn delete_clients(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM sys_client WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn count_files(&self, filter: &FileFilter<'_>) -> Result<i64, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM sys_file AS f");
        query.push(" WHERE 1 = 1");
        push_file_filters(&mut query, filter);
        Ok(query
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_files(&self, filter: &FileFilter<'_>) -> Result<Vec<FileRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(file_select_sql());
        query.push(" WHERE 1 = 1");
        push_file_filters(&mut query, filter);
        query
            .push(" ORDER BY ")
            .push(filter.order_by)
            .push(" LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);
        Ok(query
            .build_query_as::<FileRecord>()
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn get_file(&self, id: i64) -> Result<Option<FileRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(file_select_sql());
        query.push(" WHERE f.id = ").push_bind(id).push(" LIMIT 1");
        Ok(query
            .build_query_as::<FileRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn file_by_hash(&self, sha256: &str) -> Result<Option<FileRecord>, AppError> {
        let mut query = QueryBuilder::<Postgres>::new(file_select_sql());
        query
            .push(" WHERE f.sha256 = ")
            .push_bind(sha256)
            .push(" LIMIT 1");
        Ok(query
            .build_query_as::<FileRecord>()
            .fetch_optional(&self.db)
            .await?)
    }

    pub async fn insert_file(&self, record: &FileSaveRecord<'_>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, '', NULL, '', $12, $13, $14);
"#,
        )
        .bind(record.id)
        .bind(record.name)
        .bind(record.original_name)
        .bind(record.size)
        .bind(record.parent_path)
        .bind(record.path)
        .bind(record.extension)
        .bind(record.content_type)
        .bind(record.file_type)
        .bind(record.sha256)
        .bind(record.metadata)
        .bind(record.storage_id)
        .bind(record.user_id)
        .bind(record.now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn update_file_name(
        &self,
        id: i64,
        original_name: &str,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
UPDATE sys_file
SET original_name = $1, update_user = $2, update_time = $3
WHERE id = $4;
"#,
        )
        .bind(original_name)
        .bind(user_id)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await?;
        ensure_affected(result.rows_affected())
    }

    pub async fn delete_files(&self, ids: &[i64]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM sys_file WHERE id = ANY($1);")
            .bind(ids.to_vec())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn dir_size(&self, dir_path: &str) -> Result<i64, AppError> {
        let prefix = format!("{}/%", dir_path.trim_end_matches('/'));
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(size), 0)::BIGINT FROM sys_file WHERE type <> 0 AND path LIKE $1;",
        )
        .bind(prefix)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn file_statistics(&self) -> Result<Vec<FileStatRecord>, AppError> {
        Ok(sqlx::query_as::<_, FileStatRecord>(
            r#"
SELECT type AS file_type, COUNT(*)::BIGINT AS number, COALESCE(SUM(size), 0)::BIGINT AS size
FROM sys_file
WHERE type <> 0
GROUP BY type
ORDER BY type ASC;
"#,
        )
        .fetch_all(&self.db)
        .await?)
    }

    async fn exists(
        &self,
        table: &'static str,
        column: &'static str,
        value: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, AppError> {
        let mut query = QueryBuilder::<Postgres>::new("SELECT EXISTS(SELECT 1 FROM ");
        query
            .push(table)
            .push(" WHERE ")
            .push(column)
            .push(" = ")
            .push_bind(value);
        if let Some(exclude_id) = exclude_id {
            query.push(" AND id <> ").push_bind(exclude_id);
        }
        query.push(")");
        Ok(query
            .build_query_scalar::<bool>()
            .fetch_one(&self.db)
            .await?)
    }
}

#[derive(Debug, Clone)]
pub struct DictItemSaveRecord<'a> {
    pub id: i64,
    pub label: &'a str,
    pub value: &'a str,
    pub color: Option<&'a str>,
    pub sort: i32,
    pub description: Option<&'a str>,
    pub status: i16,
    pub dict_id: i64,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct OptionUpdateRecord {
    pub id: i64,
    pub code: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StorageSaveRecord<'a> {
    pub id: i64,
    pub name: &'a str,
    pub code: &'a str,
    pub storage_type: i16,
    pub access_key: Option<&'a str>,
    pub secret_key: Option<&'a str>,
    pub endpoint: Option<&'a str>,
    pub region: Option<&'a str>,
    pub bucket_name: &'a str,
    pub domain: Option<&'a str>,
    pub description: Option<&'a str>,
    pub is_default: bool,
    pub sort: i32,
    pub status: i16,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct ClientSaveRecord {
    pub id: i64,
    pub client_id: String,
    pub client_type: String,
    pub auth_type: Value,
    pub active_timeout: i64,
    pub timeout: i64,
    pub status: i16,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct FileSaveRecord<'a> {
    pub id: i64,
    pub name: &'a str,
    pub original_name: &'a str,
    pub size: i64,
    pub parent_path: &'a str,
    pub path: &'a str,
    pub extension: &'a str,
    pub content_type: &'a str,
    pub file_type: i16,
    pub sha256: &'a str,
    pub metadata: &'a str,
    pub storage_id: i64,
    pub user_id: i64,
    pub now: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct FileStatRecord {
    pub file_type: i16,
    pub number: i64,
    pub size: i64,
}

fn dict_select_sql() -> &'static str {
    r#"
SELECT
    d.id,
    d.name,
    d.code,
    COALESCE(d.description, '') AS description,
    d.is_system,
    d.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    d.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
"#
}

fn dict_item_select_sql() -> &'static str {
    r#"
SELECT
    di.id,
    di.label,
    di.value,
    COALESCE(di.color, '') AS color,
    di.sort,
    COALESCE(di.description, '') AS description,
    di.status,
    di.dict_id,
    di.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    di.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
"#
}

fn storage_select_sql() -> &'static str {
    r#"
SELECT
    s.id,
    s.name,
    s.code,
    s.type AS storage_type,
    COALESCE(s.access_key, '') AS access_key,
    COALESCE(s.secret_key, '') AS secret_key,
    COALESCE(s.endpoint, '') AS endpoint,
    COALESCE(s.region, '') AS region,
    s.bucket_name,
    COALESCE(s.domain, '') AS domain,
    COALESCE(s.description, '') AS description,
    s.is_default,
    s.sort,
    s.status,
    s.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    s.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
"#
}

fn client_select_sql() -> &'static str {
    r#"
SELECT
    c.id,
    c.client_id,
    c.client_type,
    c.auth_type,
    c.active_timeout,
    c.timeout,
    c.status,
    c.create_user,
    c.create_time,
    c.update_user,
    c.update_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
"#
}

fn file_select_sql() -> &'static str {
    r#"
SELECT
    f.id,
    f.name,
    f.original_name,
    COALESCE(f.size, 0) AS size,
    f.parent_path,
    f.path,
    COALESCE(f.sha256, '') AS sha256,
    COALESCE(f.content_type, '') AS content_type,
    COALESCE(f.metadata, '') AS metadata,
    COALESCE(f.thumbnail_size, 0) AS thumbnail_size,
    COALESCE(f.thumbnail_name, '') AS thumbnail_name,
    COALESCE(f.thumbnail_metadata, '') AS thumbnail_metadata,
    COALESCE(f.extension, '') AS extension,
    f.type AS file_type,
    f.storage_id,
    COALESCE(s.name, '') AS storage_name,
    COALESCE(s.domain, '') AS storage_domain,
    f.create_time,
    COALESCE(cu.nickname, '') AS create_user_string,
    f.update_time,
    COALESCE(uu.nickname, '') AS update_user_string
FROM sys_file AS f
LEFT JOIN sys_storage AS s ON s.id = f.storage_id
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
"#
}

fn push_dict_item_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &DictItemFilter<'_>) {
    if let Some(dict_id) = filter.dict_id.filter(|value| *value > 0) {
        query.push(" AND di.dict_id = ").push_bind(dict_id);
    }
    if let Some(description) = non_empty(filter.description) {
        let pattern = format!("%{description}%");
        query
            .push(" AND (di.label ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR di.value ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR COALESCE(di.description, '') ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(status) = filter.status.filter(|value| *value > 0) {
        query.push(" AND di.status = ").push_bind(status);
    }
}

fn push_client_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &ClientFilter<'_>) {
    if let Some(client_type) = non_empty(filter.client_type) {
        query
            .push(" AND c.client_type = ")
            .push_bind(client_type.to_owned());
    }
    if let Some(status) = filter.status.filter(|value| *value > 0) {
        query.push(" AND c.status = ").push_bind(status);
    }
    if !filter.auth_types.is_empty() {
        query.push(" AND (");
        let mut separated = query.separated(" OR ");
        for auth_type in filter.auth_types {
            separated
                .push("c.auth_type::TEXT ILIKE ")
                .push_bind(format!("%{auth_type}%"));
        }
        separated.push_unseparated(")");
    }
}

fn push_file_filters(query: &mut QueryBuilder<'_, Postgres>, filter: &FileFilter<'_>) {
    if let Some(original_name) = non_empty(filter.original_name) {
        query
            .push(" AND f.original_name ILIKE ")
            .push_bind(format!("%{original_name}%"));
    }
    if let Some(file_type) = filter.file_type.filter(|value| *value >= 0) {
        query.push(" AND f.type = ").push_bind(file_type);
    }
    if let Some(parent_path) = non_empty(filter.parent_path) {
        query
            .push(" AND f.parent_path = ")
            .push_bind(parent_path.to_owned());
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn ensure_affected(rows_affected: u64) -> Result<(), AppError> {
    if rows_affected == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

pub fn normalized_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut ids = ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn new_id() -> i64 {
    next_id()
}
