package http

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"fmt"
	"io"
	"mime/multipart"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/lib/pq"
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// FileItem matches the front-end FileItem type in admin/src/apis/system/type.ts.
type FileItem struct {
	ID               int64   `json:"id"`
	Name             string  `json:"name"`
	OriginalName     string  `json:"originalName"`
	Size             *int64  `json:"size"`
	URL              string  `json:"url"`
	ParentPath       string  `json:"parentPath"`
	Path             string  `json:"path"`
	Sha256           string  `json:"sha256"`
	ContentType      string  `json:"contentType"`
	Metadata         string  `json:"metadata"`
	ThumbnailSize    *int64  `json:"thumbnailSize"`
	ThumbnailName    string  `json:"thumbnailName"`
	ThumbnailMeta    string  `json:"thumbnailMetadata"`
	ThumbnailURL     string  `json:"thumbnailUrl"`
	Extension        string  `json:"extension"`
	Type             int16   `json:"type"`
	StorageID        int64   `json:"storageId"`
	StorageName      string  `json:"storageName"`
	CreateUserString string  `json:"createUserString"`
	CreateTime       string  `json:"createTime"`
	UpdateUserString string  `json:"updateUserString"`
	UpdateTime       string  `json:"updateTime"`
}

// FileStatisticsResp represents aggregated file statistics.
type FileStatisticsResp struct {
	Type   int16                `json:"type"`
	Size   int64                `json:"size"`
	Number int64                `json:"number"`
	Data   []FileStatisticsResp `json:"data,omitempty"`
}

// FileDirCalcSizeResp represents directory size response.
type FileDirCalcSizeResp struct {
	Size int64 `json:"size"`
}

// FileUploadResp matches Java's FileUploadResp.
type FileUploadResp struct {
	ID       string            `json:"id"`
	URL      string            `json:"url"`
	ThumbURL string            `json:"thUrl"`
	Metadata map[string]string `json:"metadata"`
}

// fileHandler implements /system/file and /common/file APIs.
type FileHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewFileHandler(db *sql.DB, tokenSvc *security.TokenService) *FileHandler {
	return &FileHandler{
		db:       db,
		tokenSvc: tokenSvc,
	}
}

// RegisterFileRoutes registers all file-related routes.
func (h *FileHandler) RegisterFileRoutes(r *gin.Engine) {
	// System file management
	r.GET("/system/file", h.ListFile)
	r.POST("/system/file/upload", h.UploadFile)
	r.POST("/system/file/dir", h.CreateDir)
	r.GET("/system/file/dir/:id/size", h.CalcDirSize)
	r.GET("/system/file/statistics", h.Statistics)
	r.GET("/system/file/check", h.CheckFile)
	r.PUT("/system/file/:id", h.UpdateFile)
	r.DELETE("/system/file", h.DeleteFile)

	// Common upload (avatar, editor, etc.)
	r.POST("/common/file", h.UploadFile)
}

func (h *FileHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// StorageConfig 表示 sys_storage 中的一条存储配置。
type StorageConfig struct {
	ID         int64
	Name       string
	Code       string
	Type       int16
	AccessKey  string
	SecretKey  string
	Endpoint   string
	BucketName string
	Domain     string
	IsDefault  bool
	Status     int16
}

// storageType 常量定义，与 Java StorageTypeEnum 一致：1=LOCAL，2=OSS(MinIO等)。
const (
	storageTypeLocal int16 = 1
	storageTypeOSS   int16 = 2
)

// fileBaseURLPrefix returns the URL prefix used for local file URLs, e.g. "/file".
func fileBaseURLPrefix() string {
	prefix := os.Getenv("FILE_BASE_URL")
	if strings.TrimSpace(prefix) == "" {
		prefix = "/file"
	}
	if !strings.HasPrefix(prefix, "/") {
		prefix = "/" + prefix
	}
	return strings.TrimRight(prefix, "/")
}

// buildLocalFileURL 构建本地存储的访问 URL。
func buildLocalFileURL(path string) string {
	if path == "" {
		return ""
	}
	if !strings.HasPrefix(path, "/") {
		path = "/" + path
	}
	return fileBaseURLPrefix() + path
}

// buildStorageFileURL 根据存储配置构建文件访问 URL。
// - 对象存储：使用 storage.Domain 作为前缀（需配置为 http(s) 开头）。
// - 本地存储或未配置域名：退回到本地静态路径 /file。
func buildStorageFileURL(storage *StorageConfig, fullPath string) string {
	if storage == nil {
		return buildLocalFileURL(fullPath)
	}
	switch storage.Type {
	case storageTypeOSS:
		// 对象存储必须配置 Domain，形如 http://minio:9000/bucket/
		domain := strings.TrimSpace(storage.Domain)
		if domain == "" {
			return buildLocalFileURL(fullPath)
		}
		// 规范化 Domain，保证无多余斜杠
		domain = strings.TrimRight(domain, "/")
		key := strings.TrimPrefix(fullPath, "/")
		return domain + "/" + key
	default:
		return buildLocalFileURL(fullPath)
	}
}

// getDefaultStorage 查询默认存储；若未显式指定，则退回到本地存储（使用 ./data/file）。
func (h *FileHandler) getDefaultStorage(ctx context.Context) (*StorageConfig, error) {
	const query = `
SELECT id, name, code, type,
       COALESCE(access_key, ''),
       COALESCE(secret_key, ''),
       COALESCE(endpoint, ''),
       COALESCE(bucket_name, ''),
       COALESCE(domain, ''),
       COALESCE(is_default, FALSE),
       COALESCE(status, 1)
FROM sys_storage
WHERE is_default = TRUE
LIMIT 1;
`
	var cfg StorageConfig
	err := h.db.QueryRowContext(ctx, query).
		Scan(
			&cfg.ID,
			&cfg.Name,
			&cfg.Code,
			&cfg.Type,
			&cfg.AccessKey,
			&cfg.SecretKey,
			&cfg.Endpoint,
			&cfg.BucketName,
			&cfg.Domain,
			&cfg.IsDefault,
			&cfg.Status,
		)
	if err == sql.ErrNoRows {
		// 没有配置默认存储时，按单一本地存储回退，保持兼容原有逻辑。
		return &StorageConfig{
			ID:         1,
			Name:       "本地存储",
			Code:       "local",
			Type:       storageTypeLocal,
			BucketName: "./data/file",
			Domain:     "",
			IsDefault:  true,
			Status:     1,
		}, nil
	}
	if err != nil {
		return nil, err
	}
	// 如果 BucketName 未配置，本地存储仍然使用默认路径。
	if cfg.Type == storageTypeLocal && strings.TrimSpace(cfg.BucketName) == "" {
		cfg.BucketName = "./data/file"
	}
	return &cfg, nil
}

// getStorageByID 根据存储 ID 查询配置。
func (h *FileHandler) getStorageByID(ctx context.Context, id int64) (*StorageConfig, error) {
	const query = `
SELECT id, name, code, type,
       COALESCE(access_key, ''),
       COALESCE(secret_key, ''),
       COALESCE(endpoint, ''),
       COALESCE(bucket_name, ''),
       COALESCE(domain, ''),
       COALESCE(is_default, FALSE),
       COALESCE(status, 1)
FROM sys_storage
WHERE id = $1;
`
	var cfg StorageConfig
	err := h.db.QueryRowContext(ctx, query, id).
		Scan(
			&cfg.ID,
			&cfg.Name,
			&cfg.Code,
			&cfg.Type,
			&cfg.AccessKey,
			&cfg.SecretKey,
			&cfg.Endpoint,
			&cfg.BucketName,
			&cfg.Domain,
			&cfg.IsDefault,
			&cfg.Status,
		)
	if err != nil {
		return nil, err
	}
	if cfg.Type == storageTypeLocal && strings.TrimSpace(cfg.BucketName) == "" {
		cfg.BucketName = "./data/file"
	}
	return &cfg, nil
}

// saveToLocal 将文件保存到本地目录，并返回哈希等信息。
func saveToLocal(header *multipart.FileHeader, bucketPath, parentPath, storedName string) (fullPath, sha string, size int64, contentType string, err error) {
	parentPath = normalizeParentPath(parentPath)

	// Full logical path stored in DB, e.g. /2025/1/1/123.jpg
	if parentPath == "/" {
		fullPath = "/" + storedName
	} else {
		fullPath = parentPath + "/" + storedName
	}

	// Physical path on disk: bucketPath + relative path.
	if strings.TrimSpace(bucketPath) == "" {
		bucketPath = "./data/file"
	}
	relative := strings.TrimPrefix(fullPath, "/")
	dstPath := filepath.Join(bucketPath, filepath.FromSlash(relative))
	if err = os.MkdirAll(filepath.Dir(dstPath), 0o755); err != nil {
		return
	}

	src, err := header.Open()
	if err != nil {
		return
	}
	defer src.Close()

	dst, err := os.Create(dstPath)
	if err != nil {
		return
	}
	defer dst.Close()

	h := sha256.New()
	w := io.MultiWriter(dst, h)
	written, err := io.Copy(w, src)
	if err != nil {
		return
	}
	size = written
	sha = hex.EncodeToString(h.Sum(nil))
	contentType = header.Header.Get("Content-Type")
	return
}

// saveToMinIO 将文件上传到 MinIO/对象存储，并返回哈希等信息。
func saveToMinIO(ctx context.Context, header *multipart.FileHeader, storage *StorageConfig, parentPath, storedName string) (fullPath, sha string, size int64, contentType string, err error) {
	if storage == nil {
		err = fmt.Errorf("storage config is nil")
		return
	}
	if strings.TrimSpace(storage.Endpoint) == "" || strings.TrimSpace(storage.AccessKey) == "" || strings.TrimSpace(storage.SecretKey) == "" || strings.TrimSpace(storage.BucketName) == "" {
		err = fmt.Errorf("对象存储配置不完整")
		return
	}

	parentPath = normalizeParentPath(parentPath)
	if parentPath == "/" {
		fullPath = "/" + storedName
	} else {
		fullPath = parentPath + "/" + storedName
	}
	objectName := strings.TrimPrefix(fullPath, "/")

	contentType = header.Header.Get("Content-Type")
	// 先计算 SHA256
	src, err := header.Open()
	if err != nil {
		return
	}
	h := sha256.New()
	written, err := io.Copy(h, src)
	_ = src.Close()
	if err != nil {
		return
	}
	size = written
	sha = hex.EncodeToString(h.Sum(nil))

	// 创建 MinIO 客户端
	endpoint := storage.Endpoint
	secure := false
	if strings.HasPrefix(endpoint, "http://") || strings.HasPrefix(endpoint, "https://") {
		u, parseErr := url.Parse(endpoint)
		if parseErr != nil {
			err = parseErr
			return
		}
		secure = u.Scheme == "https"
		endpoint = u.Host
	}

	client, err := minio.New(endpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(storage.AccessKey, storage.SecretKey, ""),
		Secure: secure,
	})
	if err != nil {
		return
	}

	// 确保 Bucket 存在
	exists, errBucket := client.BucketExists(ctx, storage.BucketName)
	if errBucket != nil {
		err = errBucket
		return
	}
	if !exists {
		if err = client.MakeBucket(ctx, storage.BucketName, minio.MakeBucketOptions{}); err != nil {
			return
		}
	}

	// 实际上传
	src2, err := header.Open()
	if err != nil {
		return
	}
	defer src2.Close()

	_, err = client.PutObject(ctx, storage.BucketName, objectName, src2, size, minio.PutObjectOptions{
		ContentType: contentType,
	})
	if err != nil {
		return
	}
	return
}

// normalizeParentPath ensures parent path is in the form "/xxx/yyy" (no trailing slash).
func normalizeParentPath(p string) string {
	p = strings.TrimSpace(p)
	if p == "" {
		return "/"
	}
	if !strings.HasPrefix(p, "/") {
		p = "/" + p
	}
	// Drop trailing slash except for root.
	if len(p) > 1 {
		p = strings.TrimRight(p, "/")
	}
	return p
}

func extensionFromFilename(name string) string {
	ext := strings.TrimPrefix(strings.ToLower(filepath.Ext(name)), ".")
	return ext
}

func detectFileType(ext, contentType string) int16 {
	ext = strings.ToLower(ext)
	switch {
	case strings.HasPrefix(contentType, "image/"):
		return 2
	case strings.HasPrefix(contentType, "video/"):
		return 4
	case strings.HasPrefix(contentType, "audio/"):
		return 5
	case ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif":
		return 2
	case ext == "doc" || ext == "docx" || ext == "xls" || ext == "xlsx" || ext == "ppt" || ext == "pptx" || ext == "pdf" || ext == "txt":
		return 3
	default:
		return 1
	}
}

// UploadFile handles POST /system/file/upload and POST /common/file.
func (h *FileHandler) UploadFile(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	header, err := c.FormFile("file")
	if err != nil {
		Fail(c, "400", "文件不能为空")
		return
	}
	parentPath := c.PostForm("parentPath")
	if parentPath == "" {
		parentPath = "/"
	}

	ext := extensionFromFilename(header.Filename)
	newID := id.Next()
	var storedName string
	if ext != "" {
		storedName = fmt.Sprintf("%d.%s", newID, ext)
	} else {
		storedName = fmt.Sprintf("%d", newID)
	}

	// 根据默认存储类型，分别保存到本地或 MinIO。
	ctx := c.Request.Context()
	storageCfg, err := h.getDefaultStorage(ctx)
	if err != nil {
		Fail(c, "500", "获取存储配置失败")
		return
	}

	var (
		fullPath    string
		sha         string
		size        int64
		contentType string
	)
	switch storageCfg.Type {
	case storageTypeOSS:
		fullPath, sha, size, contentType, err = saveToMinIO(ctx, header, storageCfg, parentPath, storedName)
	default:
		fullPath, sha, size, contentType, err = saveToLocal(header, storageCfg.BucketName, parentPath, storedName)
	}
	if err != nil {
		Fail(c, "500", "保存文件失败")
		return
	}

	now := time.Now()
	fileType := detectFileType(ext, contentType)

	const insertSQL = `
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8,
    $9, $10, $11, $12, $13, $14,
    $15, $16, $17
);`

	fileID := id.Next()
	var meta string
	_, err = h.db.ExecContext(
		c.Request.Context(),
		insertSQL,
		fileID,
		storedName,
		header.Filename,
		size,
		normalizeParentPath(parentPath),
		fullPath,
		ext,
		contentType,
		fileType,
		sha,
		meta,
		"",
		nil,
		"",
		storageCfg.ID,
		userID,
		now,
	)
	if err != nil {
		Fail(c, "500", "保存文件记录失败")
		return
	}

	url := buildStorageFileURL(storageCfg, fullPath)
	resp := FileUploadResp{
		ID:       strconv.FormatInt(fileID, 10),
		URL:      url,
		ThumbURL: url,
		Metadata: map[string]string{},
	}
	OK(c, resp)
}

// ListFile handles GET /system/file (paged).
func (h *FileHandler) ListFile(c *gin.Context) {
	originalName := strings.TrimSpace(c.Query("originalName"))
	typeStr := strings.TrimSpace(c.Query("type"))
	parentPath := strings.TrimSpace(c.Query("parentPath"))

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	size, _ := strconv.Atoi(c.DefaultQuery("size", "30"))
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 30
	}

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if originalName != "" {
		where += fmt.Sprintf(" AND f.original_name ILIKE $%d", argPos)
		args = append(args, "%"+originalName+"%")
		argPos++
	}

	if typeStr != "" && typeStr != "0" {
		if t, err := strconv.Atoi(typeStr); err == nil && t > 0 {
			where += fmt.Sprintf(" AND f.type = $%d", argPos)
			args = append(args, t)
			argPos++
		}
	}

	if parentPath != "" {
		where += fmt.Sprintf(" AND f.parent_path = $%d", argPos)
		args = append(args, normalizeParentPath(parentPath))
		argPos++
	}

	countSQL := "SELECT COUNT(*) FROM sys_file AS f " + where
	var total int64
	if err := h.db.QueryRowContext(c.Request.Context(), countSQL, args...).Scan(&total); err != nil {
		Fail(c, "500", "查询文件失败")
		return
	}
	if total == 0 {
		OK(c, PageResult[FileItem]{List: []FileItem{}, Total: 0})
		return
	}

	offset := int64((page - 1) * size)
	args = append(args, int64(size), offset)
	limitPos := argPos
	offsetPos := argPos + 1

	query := fmt.Sprintf(`
SELECT f.id,
       f.name,
       f.original_name,
       f.size,
       f.parent_path,
       f.path,
       COALESCE(f.extension, ''),
       COALESCE(f.content_type, ''),
       f.type,
       COALESCE(f.sha256, ''),
       COALESCE(f.metadata, ''),
       COALESCE(f.thumbnail_name, ''),
       f.thumbnail_size,
       COALESCE(f.thumbnail_metadata, ''),
       f.storage_id,
       f.create_time,
       COALESCE(cu.nickname, ''),
       f.update_time,
       COALESCE(uu.nickname, '')
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
%s
ORDER BY f.type ASC, f.update_time DESC NULLS LAST, f.id DESC
LIMIT $%d OFFSET $%d;
`, where, limitPos, offsetPos)

	rows, err := h.db.QueryContext(c.Request.Context(), query, args...)
	if err != nil {
		Fail(c, "500", "查询文件失败")
		return
	}
	defer rows.Close()

	var list []FileItem
	for rows.Next() {
		var (
			item          FileItem
			sizeVal       sql.NullInt64
			thumbSizeVal  sql.NullInt64
			createTime    time.Time
			updateTimeVal sql.NullTime
		)
		if err := rows.Scan(
			&item.ID,
			&item.Name,
			&item.OriginalName,
			&sizeVal,
			&item.ParentPath,
			&item.Path,
			&item.Extension,
			&item.ContentType,
			&item.Type,
			&item.Sha256,
			&item.Metadata,
			&item.ThumbnailName,
			&thumbSizeVal,
			&item.ThumbnailMeta,
			&item.StorageID,
			&createTime,
			&item.CreateUserString,
			&updateTimeVal,
			&item.UpdateUserString,
		); err != nil {
			Fail(c, "500", "解析文件数据失败")
			return
		}
		if sizeVal.Valid {
			item.Size = &sizeVal.Int64
		}
		if thumbSizeVal.Valid {
			item.ThumbnailSize = &thumbSizeVal.Int64
		}
		item.CreateTime = formatTime(createTime)
		if updateTimeVal.Valid {
			item.UpdateTime = formatTime(updateTimeVal.Time)
		}
		// 填充存储名称和访问 URL
		var storageCfg *StorageConfig
		if item.StorageID > 0 {
			storageCfg, _ = h.getStorageByID(c.Request.Context(), item.StorageID)
		}
		if storageCfg != nil {
			item.StorageName = storageCfg.Name
		} else {
			item.StorageName = "本地存储"
		}
		item.URL = buildStorageFileURL(storageCfg, item.Path)
		if item.ThumbnailName != "" {
			parent := item.ParentPath
			if parent == "/" {
				parent = ""
			}
			thumbPath := parent + "/" + item.ThumbnailName
			item.ThumbnailURL = buildStorageFileURL(storageCfg, thumbPath)
		} else {
			item.ThumbnailURL = item.URL
		}
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询文件失败")
		return
	}

	OK(c, PageResult[FileItem]{List: list, Total: total})
}

// CreateDir handles POST /system/file/dir.
func (h *FileHandler) CreateDir(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req struct {
		ParentPath   string `json:"parentPath"`
		OriginalName string `json:"originalName"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.OriginalName = strings.TrimSpace(req.OriginalName)
	if req.OriginalName == "" {
		Fail(c, "400", "名称不能为空")
		return
	}
	parentPath := normalizeParentPath(req.ParentPath)
	if parentPath == "" {
		parentPath = "/"
	}

	// Check duplicate folder name under same parent.
	const existsSQL = `
SELECT 1 FROM sys_file
WHERE parent_path = $1 AND name = $2 AND type = 0
LIMIT 1;
`
	var dummy int
	if err := h.db.QueryRowContext(c.Request.Context(), existsSQL, parentPath, req.OriginalName).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验文件夹失败")
		return
	} else if err == nil {
		Fail(c, "400", "文件夹已存在")
		return
	}

	now := time.Now()
	dirID := id.Next()
	var path string
	if parentPath == "/" {
		path = "/" + req.OriginalName
	} else {
		path = parentPath + "/" + req.OriginalName
	}

	const insertSQL = `
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    $1, $2, $3, NULL, $4, $5, NULL, NULL,
    0, '', '', '', NULL, '',
    1, $6, $7
);`

	if _, err := h.db.ExecContext(
		c.Request.Context(),
		insertSQL,
		dirID,
		req.OriginalName,
		req.OriginalName,
		parentPath,
		path,
		userID,
		now,
	); err != nil {
		Fail(c, "500", "创建文件夹失败")
		return
	}

	OK(c, true)
}

// CalcDirSize handles GET /system/file/dir/:id/size.
func (h *FileHandler) CalcDirSize(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const selectDir = `
SELECT path, type
FROM sys_file
WHERE id = $1;
`
	var path string
	var t int16
	if err := h.db.QueryRowContext(c.Request.Context(), selectDir, idVal).Scan(&path, &t); err != nil {
		if err == sql.ErrNoRows {
			Fail(c, "404", "文件夹不存在")
			return
		}
		Fail(c, "500", "查询文件夹失败")
		return
	}
	if t != 0 {
		Fail(c, "400", "ID 不是文件夹，无法计算大小")
		return
	}

	const sumSQL = `
SELECT COALESCE(SUM(size), 0)
FROM sys_file
WHERE type <> 0 AND path LIKE $1;
`
	var total int64
	prefix := strings.TrimRight(path, "/") + "/%"
	if err := h.db.QueryRowContext(c.Request.Context(), sumSQL, prefix).Scan(&total); err != nil {
		Fail(c, "500", "计算文件夹大小失败")
		return
	}

	OK(c, FileDirCalcSizeResp{Size: total})
}

// Statistics handles GET /system/file/statistics.
func (h *FileHandler) Statistics(c *gin.Context) {
	const query = `
SELECT type, COUNT(1) AS number, COALESCE(SUM(size), 0) AS size
FROM sys_file
WHERE type <> 0
GROUP BY type;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询文件统计失败")
		return
	}
	defer rows.Close()

	var list []FileStatisticsResp
	var totalSize int64
	var totalNumber int64

	for rows.Next() {
		var item FileStatisticsResp
		if err := rows.Scan(&item.Type, &item.Number, &item.Size); err != nil {
			Fail(c, "500", "解析文件统计失败")
			return
		}
		totalSize += item.Size
		totalNumber += item.Number
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询文件统计失败")
		return
	}

	if len(list) == 0 {
		OK(c, FileStatisticsResp{})
		return
	}

	resp := FileStatisticsResp{
		Size:   totalSize,
		Number: totalNumber,
		Data:   list,
	}
	OK(c, resp)
}

// CheckFile handles GET /system/file/check?fileHash=...
func (h *FileHandler) CheckFile(c *gin.Context) {
	hash := strings.TrimSpace(c.Query("fileHash"))
	if hash == "" {
		OK[any](c, nil)
		return
	}

	const query = `
SELECT id,
       name,
       original_name,
       size,
       parent_path,
       path,
       COALESCE(extension, ''),
       COALESCE(content_type, ''),
       type,
       COALESCE(sha256, ''),
       COALESCE(metadata, ''),
       COALESCE(thumbnail_name, ''),
       thumbnail_size,
       COALESCE(thumbnail_metadata, ''),
       storage_id,
       create_time,
       COALESCE(cu.nickname, ''),
       update_time,
       COALESCE(uu.nickname, '')
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
WHERE sha256 = $1
LIMIT 1;
`

	var (
		item          FileItem
		sizeVal       sql.NullInt64
		thumbSizeVal  sql.NullInt64
		createTime    time.Time
		updateTimeVal sql.NullTime
	)
	err := h.db.QueryRowContext(c.Request.Context(), query, hash).Scan(
		&item.ID,
		&item.Name,
		&item.OriginalName,
		&sizeVal,
		&item.ParentPath,
		&item.Path,
		&item.Extension,
		&item.ContentType,
		&item.Type,
		&item.Sha256,
		&item.Metadata,
		&item.ThumbnailName,
		&thumbSizeVal,
		&item.ThumbnailMeta,
		&item.StorageID,
		&createTime,
		&item.CreateUserString,
		&updateTimeVal,
		&item.UpdateUserString,
	)
	if err != nil {
		if err == sql.ErrNoRows {
			OK[any](c, nil)
			return
		}
		Fail(c, "500", "查询文件失败")
		return
	}

	if sizeVal.Valid {
		item.Size = &sizeVal.Int64
	}
	if thumbSizeVal.Valid {
		item.ThumbnailSize = &thumbSizeVal.Int64
	}
	item.CreateTime = formatTime(createTime)
	if updateTimeVal.Valid {
		item.UpdateTime = formatTime(updateTimeVal.Time)
	}
	var storageCfg *StorageConfig
	if item.StorageID > 0 {
		storageCfg, _ = h.getStorageByID(c.Request.Context(), item.StorageID)
	}
	if storageCfg != nil {
		item.StorageName = storageCfg.Name
	} else {
		item.StorageName = "本地存储"
	}
	item.URL = buildStorageFileURL(storageCfg, item.Path)
	if item.ThumbnailName != "" {
		parent := item.ParentPath
		if parent == "/" {
			parent = ""
		}
		thumbPath := parent + "/" + item.ThumbnailName
		item.ThumbnailURL = buildStorageFileURL(storageCfg, thumbPath)
	} else {
		item.ThumbnailURL = item.URL
	}

	OK(c, item)
}

// UpdateFile handles PUT /system/file/:id (rename).
func (h *FileHandler) UpdateFile(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req struct {
		OriginalName string `json:"originalName"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.OriginalName = strings.TrimSpace(req.OriginalName)
	if req.OriginalName == "" {
		Fail(c, "400", "名称不能为空")
		return
	}

	const updateSQL = `
UPDATE sys_file
   SET original_name = $1,
       update_user   = $2,
       update_time   = $3
 WHERE id            = $4;
`
	if _, err := h.db.ExecContext(c.Request.Context(), updateSQL, req.OriginalName, userID, time.Now(), idVal); err != nil {
		Fail(c, "500", "重命名失败")
		return
	}
	OK(c, true)
}

// DeleteFile handles DELETE /system/file.
func (h *FileHandler) DeleteFile(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	_ = userID

	var req idsRequest
	if err := c.ShouldBindJSON(&req); err != nil || len(req.IDs) == 0 {
		Fail(c, "400", "ID 列表不能为空")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "删除文件失败")
		return
	}
	defer tx.Rollback()

	type fileRow struct {
		id        int64
		name      string
		path      string
		parent    string
		fileType  int16
		storageID int64
	}

	var toDeleteFiles []fileRow

	for _, idVal := range req.IDs {
		var row fileRow
		const selectSQL = `
SELECT id, name, path, parent_path, type, storage_id
FROM sys_file
WHERE id = $1;
`
		if err := tx.QueryRowContext(c.Request.Context(), selectSQL, idVal).Scan(
			&row.id,
			&row.name,
			&row.path,
			&row.parent,
			&row.fileType,
			&row.storageID,
		); err != nil {
			if err == sql.ErrNoRows {
				continue
			}
			Fail(c, "500", "删除文件失败")
			return
		}

		if row.fileType == 0 {
			// Directory: ensure it's empty.
			const childSQL = `
SELECT 1 FROM sys_file
WHERE parent_path = $1
LIMIT 1;
`
			var dummy int
			if err := tx.QueryRowContext(c.Request.Context(), childSQL, row.path).Scan(&dummy); err != nil && err != sql.ErrNoRows {
				Fail(c, "500", "删除文件失败")
				return
			} else if err == nil {
				Fail(c, "400", fmt.Sprintf("文件夹 [%s] 不为空，请先删除文件夹下的内容", row.name))
				return
			}
		} else {
			toDeleteFiles = append(toDeleteFiles, row)
		}
	}

	// Delete DB records.
	const deleteSQL = `DELETE FROM sys_file WHERE id = ANY($1);`
	if _, err := tx.ExecContext(c.Request.Context(), deleteSQL, pq.Int64Array(req.IDs)); err != nil {
		Fail(c, "500", "删除文件失败")
		return
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除文件失败")
		return
	}

	// Best-effort deletion of物理文件（本地）或对象存储文件（MinIO）。
	for _, f := range toDeleteFiles {
		if f.path == "" {
			continue
		}
		storageCfg, err := h.getStorageByID(c.Request.Context(), f.storageID)
		if err != nil || storageCfg == nil {
			continue
		}
		switch storageCfg.Type {
		case storageTypeOSS:
			// 删除 MinIO 对象
			endpoint := storageCfg.Endpoint
			secure := false
			if strings.HasPrefix(endpoint, "http://") || strings.HasPrefix(endpoint, "https://") {
				u, parseErr := url.Parse(endpoint)
				if parseErr != nil {
					continue
				}
				secure = u.Scheme == "https"
				endpoint = u.Host
			}
			client, err := minio.New(endpoint, &minio.Options{
				Creds:  credentials.NewStaticV4(storageCfg.AccessKey, storageCfg.SecretKey, ""),
				Secure: secure,
			})
			if err != nil {
				continue
			}
			objectName := strings.TrimPrefix(f.path, "/")
			_ = client.RemoveObject(c.Request.Context(), storageCfg.BucketName, objectName, minio.RemoveObjectOptions{})
		default:
			// 本地删除
			rel := strings.TrimPrefix(f.path, "/")
			bucket := storageCfg.BucketName
			if strings.TrimSpace(bucket) == "" {
				bucket = "./data/file"
			}
			abs := filepath.Join(bucket, filepath.FromSlash(rel))
			_ = os.Remove(abs)
		}
	}

	OK(c, true)
}
