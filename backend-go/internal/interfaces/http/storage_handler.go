package http

import (
	"database/sql"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// StorageResp 对应前端 StorageResp 类型，用于存储配置列表与详情展示。
type StorageResp struct {
	ID               int64  `json:"id"`
	Name             string `json:"name"`
	Code             string `json:"code"`
	Type             int16  `json:"type"`
	AccessKey        string `json:"accessKey"`
	SecretKey        string `json:"secretKey"`
	Endpoint         string `json:"endpoint"`
	BucketName       string `json:"bucketName"`
	Domain           string `json:"domain"`
	Description      string `json:"description"`
	IsDefault        bool   `json:"isDefault"`
	Sort             int32  `json:"sort"`
	Status           int16  `json:"status"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
	UpdateUserString string `json:"updateUserString"`
	UpdateTime       string `json:"updateTime"`
}

// storageReq 用于接收新增/修改存储配置请求。
type storageReq struct {
	Name        string  `json:"name"`
	Code        string  `json:"code"`
	Type        int16   `json:"type"`
	AccessKey   string  `json:"accessKey"`
	SecretKey   *string `json:"secretKey"`
	Endpoint    string  `json:"endpoint"`
	BucketName  string  `json:"bucketName"`
	Domain      string  `json:"domain"`
	Description string  `json:"description"`
	IsDefault   *bool   `json:"isDefault"`
	Sort        int32   `json:"sort"`
	Status      int16   `json:"status"`
}

// StorageHandler 提供 /system/storage 相关接口。
type StorageHandler struct {
	db           *sql.DB
	tokenSvc     *security.TokenService
	rsaDecryptor *security.RSADecryptor
}

func NewStorageHandler(db *sql.DB, tokenSvc *security.TokenService, rsa *security.RSADecryptor) *StorageHandler {
	return &StorageHandler{
		db:           db,
		tokenSvc:     tokenSvc,
		rsaDecryptor: rsa,
	}
}

// RegisterStorageRoutes 注册存储配置相关路由。
func (h *StorageHandler) RegisterStorageRoutes(r *gin.Engine) {
	r.GET("/system/storage/list", h.ListStorage)
	r.GET("/system/storage/:id", h.GetStorage)
	r.POST("/system/storage", h.CreateStorage)
	r.PUT("/system/storage/:id", h.UpdateStorage)
	r.DELETE("/system/storage", h.DeleteStorage)
	r.PUT("/system/storage/:id/status", h.UpdateStorageStatus)
	r.PUT("/system/storage/:id/default", h.SetDefaultStorage)
}

func (h *StorageHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// decryptSecretKey 使用后端 RSA 私钥对前端加密的密钥进行解密，并做长度校验。
// 如果 encrypted 为空或 nil，则返回 oldVal（用于更新场景保持原值）。
func (h *StorageHandler) decryptSecretKey(encrypted *string, oldVal string) (string, error) {
	if encrypted == nil {
		return oldVal, nil
	}
	val := strings.TrimSpace(*encrypted)
	if val == "" {
		return "", nil
	}
	if h.rsaDecryptor == nil {
		return "", fmt.Errorf("存储密钥解密器未初始化")
	}
	plain, err := h.rsaDecryptor.DecryptBase64(val)
	if err != nil {
		return "", fmt.Errorf("私有密钥解密失败")
	}
	if len(plain) > 255 {
		return "", fmt.Errorf("私有密钥长度不能超过 255 个字符")
	}
	return plain, nil
}

// ListStorage 处理 GET /system/storage/list，支持按描述与类型筛选。
func (h *StorageHandler) ListStorage(c *gin.Context) {
	description := strings.TrimSpace(c.Query("description"))
	typeStr := strings.TrimSpace(c.Query("type"))

	var storageType int64
	if typeStr != "" {
		storageType, _ = strconv.ParseInt(typeStr, 10, 64)
	}

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if description != "" {
		where += fmt.Sprintf(" AND (s.name ILIKE $%d OR s.code ILIKE $%d OR COALESCE(s.description,'') ILIKE $%d)", argPos, argPos, argPos)
		args = append(args, "%"+description+"%")
		argPos++
	}
	if storageType != 0 {
		where += fmt.Sprintf(" AND s.type = $%d", argPos)
		args = append(args, storageType)
		argPos++
	}

	query := fmt.Sprintf(`
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, ''),
       COALESCE(s.endpoint, ''),
       s.bucket_name,
       COALESCE(s.domain, ''),
       COALESCE(s.description, ''),
       s.is_default,
       COALESCE(s.sort, 999),
       s.status,
       s.create_time,
       COALESCE(cu.nickname, ''),
       s.update_time,
       COALESCE(uu.nickname, '')
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
%s
ORDER BY s.sort ASC, s.id ASC;
`, where)

	rows, err := h.db.QueryContext(c.Request.Context(), query, args...)
	if err != nil {
		Fail(c, "500", "查询存储配置失败")
		return
	}
	defer rows.Close()

	var list []StorageResp
	for rows.Next() {
		var (
			item        StorageResp
			createAt    time.Time
			updateAt    sql.NullTime
			secretDummy string
		)
		if err := rows.Scan(
			&item.ID,
			&item.Name,
			&item.Code,
			&item.Type,
			&item.AccessKey,
			&item.Endpoint,
			&item.BucketName,
			&item.Domain,
			&item.Description,
			&item.IsDefault,
			&item.Sort,
			&item.Status,
			&createAt,
			&item.CreateUserString,
			&updateAt,
			&item.UpdateUserString,
		); err != nil {
			_ = secretDummy
			Fail(c, "500", "解析存储配置失败")
			return
		}
		item.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			item.UpdateTime = formatTime(updateAt.Time)
		}
		// 列表场景不需要返回 SecretKey，保持为空字符串即可。
		item.SecretKey = ""
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询存储配置失败")
		return
	}
	OK(c, list)
}

// GetStorage 处理 GET /system/storage/:id。
func (h *StorageHandler) GetStorage(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, ''),
       COALESCE(s.secret_key, ''),
       COALESCE(s.endpoint, ''),
       s.bucket_name,
       COALESCE(s.domain, ''),
       COALESCE(s.description, ''),
       s.is_default,
       COALESCE(s.sort, 999),
       s.status,
       s.create_time,
       COALESCE(cu.nickname, ''),
       s.update_time,
       COALESCE(uu.nickname, '')
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
WHERE s.id = $1;
`

	var (
		resp     StorageResp
		createAt time.Time
		updateAt sql.NullTime
	)
	if err := h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&resp.ID,
			&resp.Name,
			&resp.Code,
			&resp.Type,
			&resp.AccessKey,
			&resp.SecretKey,
			&resp.Endpoint,
			&resp.BucketName,
			&resp.Domain,
			&resp.Description,
			&resp.IsDefault,
			&resp.Sort,
			&resp.Status,
			&createAt,
			&resp.CreateUserString,
			&updateAt,
			&resp.UpdateUserString,
		); err != nil {
		if err == sql.ErrNoRows {
			Fail(c, "404", "存储配置不存在")
			return
		}
		Fail(c, "500", "查询存储配置失败")
		return
	}
	resp.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		resp.UpdateTime = formatTime(updateAt.Time)
	}
	// 为了与 Java 行为一致，不返回真实密钥，而是掩码，前端据此决定是否更新密钥。
	if resp.SecretKey != "" {
		resp.SecretKey = "******"
	}
	OK(c, resp)
}

// CreateStorage 处理 POST /system/storage。
func (h *StorageHandler) CreateStorage(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req storageReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	req.Code = strings.TrimSpace(req.Code)
	req.BucketName = strings.TrimSpace(req.BucketName)
	req.Domain = strings.TrimSpace(req.Domain)
	req.Endpoint = strings.TrimSpace(req.Endpoint)

	if req.Name == "" || req.Code == "" {
		Fail(c, "400", "名称和编码不能为空")
		return
	}
	if req.Type == 0 {
		req.Type = 1
	}
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.Status == 0 {
		req.Status = 1
	}

	// 编码唯一性校验
	const checkCodeSQL = `SELECT 1 FROM sys_storage WHERE code = $1 LIMIT 1;`
	var dummy int
	if err := h.db.QueryRowContext(c.Request.Context(), checkCodeSQL, req.Code).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验存储编码失败")
		return
	} else if err == nil {
		Fail(c, "400", "存储编码已存在")
		return
	}

	// 解密 SecretKey（仅对象存储需要）
	secretVal := ""
	if req.Type == 2 {
		var err error
		secretVal, err = h.decryptSecretKey(req.SecretKey, "")
		if err != nil {
			Fail(c, "400", err.Error())
			return
		}
	}

	now := time.Now()
	idVal := id.Next()

	const stmt = `
INSERT INTO sys_storage (
    id, name, code, type, access_key, secret_key, endpoint,
    bucket_name, domain, description, is_default, sort, status,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, $6, $7,
    $8, $9, $10, $11, $12, $13,
    $14, $15
);
`
	isDefault := false
	if req.IsDefault != nil {
		isDefault = *req.IsDefault
	}

	if _, err := h.db.ExecContext(
		c.Request.Context(),
		stmt,
		idVal,
		req.Name,
		req.Code,
		req.Type,
		req.AccessKey,
		secretVal,
		req.Endpoint,
		req.BucketName,
		req.Domain,
		req.Description,
		isDefault,
		req.Sort,
		req.Status,
		userID,
		now,
	); err != nil {
		Fail(c, "500", "新增存储配置失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateStorage 处理 PUT /system/storage/:id。
func (h *StorageHandler) UpdateStorage(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req storageReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	req.BucketName = strings.TrimSpace(req.BucketName)
	req.Domain = strings.TrimSpace(req.Domain)
	req.Endpoint = strings.TrimSpace(req.Endpoint)

	if req.Name == "" {
		Fail(c, "400", "名称不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.Status == 0 {
		req.Status = 1
	}

	// 查询旧配置，用于保持未修改时的密钥。
	const selectOld = `
SELECT COALESCE(secret_key, '')
FROM sys_storage
WHERE id = $1;
`
	var oldSecret string
	if err := h.db.QueryRowContext(c.Request.Context(), selectOld, idVal).Scan(&oldSecret); err != nil {
		if err == sql.ErrNoRows {
			Fail(c, "404", "存储配置不存在")
			return
		}
		Fail(c, "500", "查询存储配置失败")
		return
	}

	now := time.Now()

	// 根据是否需要修改密钥构造不同的 SQL。
	if req.SecretKey != nil {
		secretVal, err := h.decryptSecretKey(req.SecretKey, oldSecret)
		if err != nil {
			Fail(c, "400", err.Error())
			return
		}
		const stmtWithSecret = `
UPDATE sys_storage
   SET name = $1,
       type = $2,
       access_key = $3,
       secret_key = $4,
       endpoint = $5,
       bucket_name = $6,
       domain = $7,
       description = $8,
       sort = $9,
       status = $10,
       update_user = $11,
       update_time = $12
 WHERE id = $13;
`
		if _, err := h.db.ExecContext(
			c.Request.Context(),
			stmtWithSecret,
			req.Name,
			req.Type,
			req.AccessKey,
			secretVal,
			req.Endpoint,
			req.BucketName,
			req.Domain,
			req.Description,
			req.Sort,
			req.Status,
			userID,
			now,
			idVal,
		); err != nil {
			Fail(c, "500", "修改存储配置失败")
			return
		}
	} else {
		const stmt = `
UPDATE sys_storage
   SET name = $1,
       type = $2,
       access_key = $3,
       endpoint = $4,
       bucket_name = $5,
       domain = $6,
       description = $7,
       sort = $8,
       status = $9,
       update_user = $10,
       update_time = $11
 WHERE id = $12;
`
		if _, err := h.db.ExecContext(
			c.Request.Context(),
			stmt,
			req.Name,
			req.Type,
			req.AccessKey,
			req.Endpoint,
			req.BucketName,
			req.Domain,
			req.Description,
			req.Sort,
			req.Status,
			userID,
			now,
			idVal,
		); err != nil {
			Fail(c, "500", "修改存储配置失败")
			return
		}
	}
	OK(c, true)
}

// DeleteStorage 处理 DELETE /system/storage。
func (h *StorageHandler) DeleteStorage(c *gin.Context) {
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
		Fail(c, "500", "删除存储配置失败")
		return
	}
	defer tx.Rollback()

	for _, idVal := range req.IDs {
		// 不允许删除默认存储
		var isDefault bool
		if err := tx.QueryRowContext(c.Request.Context(), `SELECT is_default FROM sys_storage WHERE id = $1`, idVal).Scan(&isDefault); err != nil {
			if err == sql.ErrNoRows {
				continue
			}
			Fail(c, "500", "删除存储配置失败")
			return
		}
		if isDefault {
			Fail(c, "400", "不允许删除默认存储")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_storage WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除存储配置失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除存储配置失败")
		return
	}
	OK(c, true)
}

// UpdateStorageStatus 处理 PUT /system/storage/:id/status，仅修改启用状态。
func (h *StorageHandler) UpdateStorageStatus(c *gin.Context) {
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
		Status int16 `json:"status"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	if req.Status != 1 && req.Status != 2 {
		Fail(c, "400", "状态参数不正确")
		return
	}

	// 默认存储不允许被禁用
	var isDefault bool
	if err := h.db.QueryRowContext(c.Request.Context(), `SELECT is_default FROM sys_storage WHERE id = $1`, idVal).Scan(&isDefault); err != nil {
		if err == sql.ErrNoRows {
			Fail(c, "404", "存储配置不存在")
			return
		}
		Fail(c, "500", "更新存储状态失败")
		return
	}
	if isDefault && req.Status != 1 {
		Fail(c, "400", "不允许禁用默认存储")
		return
	}

	const stmt = `
UPDATE sys_storage
   SET status = $1,
       update_user = $2,
       update_time = $3
 WHERE id = $4;
`
	if _, err := h.db.ExecContext(c.Request.Context(), stmt, req.Status, userID, time.Now(), idVal); err != nil {
		Fail(c, "500", "更新存储状态失败")
		return
	}
	OK(c, true)
}

// SetDefaultStorage 处理 PUT /system/storage/:id/default，将指定存储设置为默认。
func (h *StorageHandler) SetDefaultStorage(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "设为默认存储失败")
		return
	}
	defer tx.Rollback()

	// 先将所有存储设为非默认
	if _, err := tx.ExecContext(c.Request.Context(), `UPDATE sys_storage SET is_default = FALSE`); err != nil {
		Fail(c, "500", "设为默认存储失败")
		return
	}
	// 再设置目标存储为默认
	if _, err := tx.ExecContext(
		c.Request.Context(),
		`UPDATE sys_storage SET is_default = TRUE, update_user = $1, update_time = $2 WHERE id = $3`,
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "设为默认存储失败")
		return
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "设为默认存储失败")
		return
	}
	OK(c, true)
}
