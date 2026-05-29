use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{
    application::system::{ensure_max_chars, format_datetime, format_optional_datetime},
    infrastructure::{
        persistence::system_misc_repositories::{
            new_id, normalized_ids, FileFilter, FileRecord, FileSaveRecord, FileStatRecord,
            StorageRecord, SystemMiscRepository,
        },
        storage::local,
    },
    shared::{
        error::AppError,
        pagination::{PageQuery, PageResult, DEFAULT_PAGE, DEFAULT_PAGE_SIZE},
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default, rename = "type")]
    pub file_type: Option<String>,
    #[serde(default)]
    pub parent_path: Option<String>,
    #[serde(
        default,
        alias = "sort[]",
        deserialize_with = "crate::shared::query::deserialize_string_vec"
    )]
    pub sort: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileUploadCommand {
    pub original_name: String,
    pub content_type: String,
    pub parent_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpdateCommand {
    #[serde(default)]
    pub original_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirCommand {
    #[serde(default)]
    pub parent_path: String,
    #[serde(default)]
    pub original_name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCheckQuery {
    #[serde(default, alias = "hash", alias = "fileHash")]
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResp {
    pub id: i64,
    pub name: String,
    pub original_name: String,
    pub size: i64,
    pub url: String,
    pub parent_path: String,
    pub path: String,
    pub sha256: String,
    pub content_type: String,
    pub metadata: String,
    pub thumbnail_size: i64,
    pub thumbnail_name: String,
    pub thumbnail_metadata: String,
    pub thumbnail_url: String,
    pub extension: String,
    #[serde(rename = "type")]
    pub file_type: i16,
    pub storage_id: i64,
    pub storage_name: String,
    pub create_user_string: String,
    pub create_time: String,
    pub update_user_string: String,
    pub update_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatisticsResp {
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: i64,
    pub number: i64,
    pub unit: String,
    pub data: Vec<FileStatisticsResp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDirCalcSizeResp {
    pub size: i64,
}

#[derive(Debug, Clone)]
pub struct FileService {
    repo: SystemMiscRepository,
}

impl FileService {
    pub fn new(db: PgPool) -> Self {
        Self {
            repo: SystemMiscRepository::new(db),
        }
    }

    pub async fn page(&self, query: FileQuery) -> Result<PageResult<FileResp>, AppError> {
        let page = PageQuery {
            page: query.page,
            size: query.size,
        }
        .normalized();
        let file_type = parse_file_type_filter(query.file_type.as_deref())?;
        let parent_path = query
            .parent_path
            .as_deref()
            .map(local::normalized_parent_path);
        let order_by = file_order_sql(&query.sort);
        let filter = FileFilter {
            original_name: query.original_name.as_deref(),
            file_type,
            parent_path: parent_path.as_deref(),
            order_by: &order_by,
            limit: page.limit(),
            offset: page.offset(),
        };
        let total = self.repo.count_files(&filter).await?;
        let list = self
            .repo
            .list_files(&filter)
            .await?
            .into_iter()
            .map(FileResp::from)
            .collect();
        Ok(PageResult::new(list, total))
    }

    pub async fn upload(
        &self,
        user_id: i64,
        command: FileUploadCommand,
    ) -> Result<FileResp, AppError> {
        let command = normalize_upload_command(command)?;
        let storage = self.enabled_default_storage().await?;
        if storage.storage_type != 1 {
            return Err(AppError::bad_request("当前仅支持本地存储上传"));
        }

        let id = new_id();
        let extension = file_extension(&command.original_name);
        let name = stored_file_name(id, &extension);
        let root = storage_root(&storage);
        let sha256 = sha256_hex(&command.bytes);
        let path = local::save_bytes(&root, &command.parent_path, &name, &command.bytes).await?;
        let metadata = file_metadata(&command.content_type, storage.storage_type);
        let file_type = detect_file_type(&extension, &command.content_type);
        let record = FileSaveRecord {
            id,
            name: &name,
            original_name: &command.original_name,
            size: command.bytes.len() as i64,
            parent_path: &command.parent_path,
            path: &path,
            extension: &extension,
            content_type: &command.content_type,
            file_type,
            sha256: &sha256,
            metadata: &metadata,
            storage_id: storage.id,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.insert_file(&record).await?;
        self.get(id).await
    }

    pub async fn get(&self, id: i64) -> Result<FileResp, AppError> {
        self.repo
            .get_file(id)
            .await?
            .map(FileResp::from)
            .ok_or(AppError::NotFound)
    }

    pub async fn update(
        &self,
        user_id: i64,
        id: i64,
        command: FileUpdateCommand,
    ) -> Result<(), AppError> {
        let original_name = normalize_file_name(command.original_name)?;
        self.repo
            .update_file_name(id, &original_name, user_id, Utc::now().naive_utc())
            .await
    }

    pub async fn delete(&self, ids: Vec<i64>) -> Result<(), AppError> {
        let ids = normalized_ids(ids);
        if ids.is_empty() {
            return Err(AppError::bad_request("ID 列表不能为空"));
        }
        self.repo.delete_files(&ids).await
    }

    pub async fn check(&self, sha256: String) -> Result<Option<FileResp>, AppError> {
        let sha256 = sha256.trim();
        if sha256.is_empty() {
            return Ok(None);
        }
        Ok(self.repo.file_by_hash(sha256).await?.map(FileResp::from))
    }

    pub async fn create_dir(
        &self,
        user_id: i64,
        command: CreateDirCommand,
    ) -> Result<FileResp, AppError> {
        let storage = self.enabled_default_storage().await?;
        let original_name = normalize_file_name(command.original_name)?;
        let parent_path = local::normalized_parent_path(&command.parent_path);
        let path = local::join_logical_path(&parent_path, &original_name);
        let id = new_id();
        let metadata = file_metadata("", storage.storage_type);
        let record = FileSaveRecord {
            id,
            name: &original_name,
            original_name: &original_name,
            size: 0,
            parent_path: &parent_path,
            path: &path,
            extension: "",
            content_type: "",
            file_type: 0,
            sha256: "",
            metadata: &metadata,
            storage_id: storage.id,
            user_id,
            now: Utc::now().naive_utc(),
        };
        self.repo.insert_file(&record).await?;
        self.get(id).await
    }

    pub async fn dir_size(&self, id: i64) -> Result<FileDirCalcSizeResp, AppError> {
        let file = self.repo.get_file(id).await?.ok_or(AppError::NotFound)?;
        if file.file_type != 0 {
            return Err(AppError::bad_request("请选择文件夹"));
        }
        Ok(FileDirCalcSizeResp {
            size: self.repo.dir_size(&file.path).await?,
        })
    }

    pub async fn statistics(&self) -> Result<FileStatisticsResp, AppError> {
        let data = self
            .repo
            .file_statistics()
            .await?
            .into_iter()
            .map(file_stat_response)
            .collect::<Vec<_>>();
        let size = data.iter().map(|item| item.size).sum();
        let number = data.iter().map(|item| item.number).sum();
        Ok(FileStatisticsResp {
            file_type: String::new(),
            size,
            number,
            unit: String::new(),
            data,
        })
    }

    async fn enabled_default_storage(&self) -> Result<StorageRecord, AppError> {
        let storage = self
            .repo
            .default_storage()
            .await?
            .ok_or_else(|| AppError::bad_request("默认存储未配置"))?;
        if storage.status != 1 {
            return Err(AppError::bad_request("默认存储未启用"));
        }
        Ok(storage)
    }
}

impl From<FileRecord> for FileResp {
    fn from(record: FileRecord) -> Self {
        let thumbnail_url = if record.thumbnail_name.is_empty() {
            String::new()
        } else {
            file_url(
                &record.storage_domain,
                &local::join_logical_path(&record.parent_path, &record.thumbnail_name),
            )
        };
        Self {
            id: record.id,
            name: record.name,
            original_name: record.original_name,
            size: record.size,
            url: file_url(&record.storage_domain, &record.path),
            parent_path: record.parent_path,
            path: record.path,
            sha256: record.sha256,
            content_type: record.content_type,
            metadata: record.metadata,
            thumbnail_size: record.thumbnail_size,
            thumbnail_name: record.thumbnail_name,
            thumbnail_metadata: record.thumbnail_metadata,
            thumbnail_url,
            extension: record.extension,
            file_type: record.file_type,
            storage_id: record.storage_id,
            storage_name: record.storage_name,
            create_user_string: record.create_user_string,
            create_time: format_datetime(record.create_time),
            update_user_string: record.update_user_string,
            update_time: format_optional_datetime(record.update_time),
        }
    }
}

pub fn file_order_sql(sort: &[String]) -> String {
    let mut clauses = Vec::new();
    for item in sort {
        let parts = item.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 2 {
            continue;
        }
        let field = parts[0]
            .trim_start_matches("t1.")
            .trim_start_matches("f.")
            .trim();
        let column = match field {
            "id" => "f.id",
            "originalName" | "original_name" => "f.original_name",
            "size" => "f.size",
            "type" => "f.type",
            "createTime" | "create_time" => "f.create_time",
            _ => continue,
        };
        let direction = match parts[1].to_ascii_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => continue,
        };
        clauses.push(format!("{column} {direction}"));
    }
    if clauses.is_empty() {
        "f.create_time DESC, f.id DESC".to_owned()
    } else {
        if !clauses.iter().any(|clause| clause.starts_with("f.id ")) {
            clauses.push("f.id DESC".to_owned());
        }
        clauses.join(", ")
    }
}

pub fn file_url(domain: &str, path: &str) -> String {
    let path = if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    };
    let domain = domain.trim();
    if domain.is_empty() || domain == "/" {
        return path;
    }
    format!(
        "{}/{}",
        domain.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn detect_file_type(extension: &str, content_type: &str) -> i16 {
    let ext = extension.trim_start_matches('.').to_ascii_lowercase();
    let mime = content_type.to_ascii_lowercase();
    if mime.starts_with("image/")
        || matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico"
        )
    {
        1
    } else if mime.starts_with("video/")
        || matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm")
    {
        2
    } else if mime.starts_with("audio/")
        || matches!(ext.as_str(), "mp3" | "wav" | "flac" | "aac" | "ogg")
    {
        3
    } else if matches!(
        ext.as_str(),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" | "md"
    ) {
        4
    } else {
        5
    }
}

fn normalize_upload_command(mut command: FileUploadCommand) -> Result<FileUploadCommand, AppError> {
    command.original_name = normalize_file_name(command.original_name)?;
    command.content_type = command.content_type.trim().to_owned();
    command.parent_path = local::normalized_parent_path(&command.parent_path);
    if command.bytes.is_empty() {
        return Err(AppError::bad_request("上传文件不能为空"));
    }
    if command.content_type.is_empty() {
        command.content_type = mime_guess::from_path(&command.original_name)
            .first_or_octet_stream()
            .essence_str()
            .to_owned();
    }
    Ok(command)
}

fn normalize_file_name(value: String) -> Result<String, AppError> {
    let value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return Err(AppError::bad_request("文件名称不能为空"));
    }
    if value.contains('/') || value == "." || value == ".." {
        return Err(AppError::bad_request("文件名称不正确"));
    }
    ensure_max_chars("文件名称", &value, 255)?;
    Ok(value)
}

fn parse_file_type_filter(value: Option<&str>) -> Result<Option<i16>, AppError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    value
        .parse::<i16>()
        .map(Some)
        .map_err(|_| AppError::bad_request("文件类型不正确"))
}

fn file_extension(original_name: &str) -> String {
    original_name
        .rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_default()
}

fn stored_file_name(id: i64, extension: &str) -> String {
    if extension.is_empty() {
        id.to_string()
    } else {
        format!("{id}.{extension}")
    }
}

fn storage_root(storage: &StorageRecord) -> PathBuf {
    if storage.bucket_name.trim().is_empty() {
        local::default_root()
    } else {
        PathBuf::from(storage.bucket_name.trim())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn file_metadata(content_type: &str, storage_type: i16) -> String {
    serde_json::json!({
        "contentType": content_type,
        "storageType": storage_type,
    })
    .to_string()
}

fn file_stat_response(record: FileStatRecord) -> FileStatisticsResp {
    FileStatisticsResp {
        file_type: record.file_type.to_string(),
        size: record.size,
        number: record.number,
        unit: String::new(),
        data: Vec::new(),
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
    use super::*;

    #[test]
    fn file_response_uses_vue_field_names() {
        let value = serde_json::to_value(FileResp {
            id: 1,
            name: "1.png".to_owned(),
            original_name: "logo.png".to_owned(),
            size: 10,
            url: "/file/logo.png".to_owned(),
            parent_path: "/".to_owned(),
            path: "/logo.png".to_owned(),
            sha256: "abc".to_owned(),
            content_type: "image/png".to_owned(),
            metadata: "{}".to_owned(),
            thumbnail_size: 0,
            thumbnail_name: String::new(),
            thumbnail_metadata: String::new(),
            thumbnail_url: String::new(),
            extension: "png".to_owned(),
            file_type: 1,
            storage_id: 1,
            storage_name: "本地".to_owned(),
            create_user_string: "admin".to_owned(),
            create_time: "2026-05-29 10:00:00".to_owned(),
            update_user_string: String::new(),
            update_time: String::new(),
        })
        .unwrap();

        assert_eq!(value["originalName"], "logo.png");
        assert_eq!(value["thumbnailSize"], 0);
        assert_eq!(value["thumbnailUrl"], "");
        assert_eq!(value["type"], 1);
        assert_eq!(value["storageId"], 1);
        assert_eq!(value["createUserString"], "admin");
    }

    #[test]
    fn file_order_sql_uses_whitelist_and_default_id_tiebreaker() {
        assert_eq!(
            file_order_sql(&["createTime,desc".to_owned(), "originalName,asc".to_owned()]),
            "f.create_time DESC, f.original_name ASC, f.id DESC"
        );
        assert_eq!(
            file_order_sql(&["originalName;drop table sys_file,desc".to_owned()]),
            "f.create_time DESC, f.id DESC"
        );
    }

    #[test]
    fn file_type_detection_matches_upload_categories() {
        assert_eq!(detect_file_type("png", ""), 1);
        assert_eq!(detect_file_type("", "video/mp4"), 2);
        assert_eq!(detect_file_type("mp3", ""), 3);
        assert_eq!(detect_file_type("pdf", ""), 4);
        assert_eq!(detect_file_type("zip", ""), 5);
    }

    #[test]
    fn file_statistics_response_uses_type_field() {
        let value = serde_json::to_value(FileStatisticsResp {
            file_type: String::new(),
            size: 10,
            number: 2,
            unit: String::new(),
            data: vec![FileStatisticsResp {
                file_type: "1".to_owned(),
                size: 10,
                number: 2,
                unit: String::new(),
                data: Vec::new(),
            }],
        })
        .unwrap();

        assert_eq!(value["type"], "");
        assert_eq!(value["data"][0]["type"], "1");
        assert_eq!(value["data"][0]["number"], 2);
    }

    #[test]
    fn file_url_joins_storage_domain_and_path() {
        assert_eq!(file_url("/file/", "/a.png"), "/file/a.png");
        assert_eq!(file_url("", "/a.png"), "/a.png");
    }
}
