package http

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// ClientResp 对应前端 ClientResp，用于列表展示。
type ClientResp struct {
	ID               int64    `json:"id"`
	ClientID         string   `json:"clientId"`
	ClientType       string   `json:"clientType"`
	AuthType         []string `json:"authType"`
	ActiveTimeout    int64    `json:"activeTimeout"`
	Timeout          int64    `json:"timeout"`
	Status           int16    `json:"status"`
	CreateUser       string   `json:"createUser"`
	CreateTime       string   `json:"createTime"`
	UpdateUser       string   `json:"updateUser"`
	UpdateTime       string   `json:"updateTime"`
	CreateUserString string   `json:"createUserString"`
	UpdateUserString string   `json:"updateUserString"`
}

// ClientDetailResp 与前端 ClientDetailResp 对应。
type ClientDetailResp = ClientResp

// clientReq 用于新增/修改客户端配置。
type clientReq struct {
	ClientType    string   `json:"clientType"`
	AuthType      []string `json:"authType"`
	ActiveTimeout int64    `json:"activeTimeout"`
	Timeout       int64    `json:"timeout"`
	Status        int16    `json:"status"`
}

// ClientHandler 提供 /system/client 相关接口。
type ClientHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewClientHandler(db *sql.DB, tokenSvc *security.TokenService) *ClientHandler {
	return &ClientHandler{
		db:       db,
		tokenSvc: tokenSvc,
	}
}

// RegisterClientRoutes 注册客户端配置路由。
func (h *ClientHandler) RegisterClientRoutes(r *gin.Engine) {
	r.GET("/system/client", h.ListClientPage)
	r.GET("/system/client/:id", h.GetClient)
	r.POST("/system/client", h.CreateClient)
	r.PUT("/system/client/:id", h.UpdateClient)
	r.DELETE("/system/client", h.DeleteClient)
}

func (h *ClientHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListClientPage 处理 GET /system/client（分页查询客户端）。
func (h *ClientHandler) ListClientPage(c *gin.Context) {
	page, _ := strconv.Atoi(c.Query("page"))
	size, _ := strconv.Atoi(c.Query("size"))
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}

	clientType := strings.TrimSpace(c.Query("clientType"))
	statusStr := strings.TrimSpace(c.Query("status"))
	authTypes := c.QueryArray("authType")

	var statusFilter int64
	if statusStr != "" {
		statusFilter, _ = strconv.ParseInt(statusStr, 10, 64)
	}

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if clientType != "" {
		where += fmt.Sprintf(" AND c.client_type = $%d", argPos)
		args = append(args, clientType)
		argPos++
	}
	if statusFilter != 0 {
		where += fmt.Sprintf(" AND c.status = $%d", argPos)
		args = append(args, statusFilter)
		argPos++
	}
	if len(authTypes) > 0 {
		// 简单实现：JSON 文本模糊匹配任意一个认证类型。
		where += fmt.Sprintf(" AND (")
		conds := make([]string, 0, len(authTypes))
		for _, t := range authTypes {
			conds = append(conds, fmt.Sprintf("c.auth_type::text ILIKE $%d", argPos))
			args = append(args, `%`+t+`%`)
			argPos++
		}
		where += strings.Join(conds, " OR ") + ")"
	}

	countSQL := "SELECT COUNT(*) FROM sys_client AS c " + where
	var total int64
	if err := h.db.QueryRowContext(c.Request.Context(), countSQL, args...).Scan(&total); err != nil {
		Fail(c, "500", "查询客户端失败")
		return
	}
	if total == 0 {
		OK(c, PageResult[ClientResp]{List: []ClientResp{}, Total: 0})
		return
	}

	offset := int64((page - 1) * size)
	limitPos := argPos
	offsetPos := argPos + 1
	argsWithPage := append(args, int64(size), offset)

	query := fmt.Sprintf(`
SELECT c.id,
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
       COALESCE(cu.nickname, ''),
       COALESCE(uu.nickname, '')
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
%s
ORDER BY c.id DESC
LIMIT $%d OFFSET $%d;
`, where, limitPos, offsetPos)

	rows, err := h.db.QueryContext(c.Request.Context(), query, argsWithPage...)
	if err != nil {
		Fail(c, "500", "查询客户端失败")
		return
	}
	defer rows.Close()

	var list []ClientResp
	for rows.Next() {
		var (
			item          ClientResp
			authRaw       []byte
			createUserID  sql.NullInt64
			updateUserID  sql.NullInt64
			createAt      time.Time
			updateAt      sql.NullTime
			createUserStr string
			updateUserStr string
		)
		if err := rows.Scan(
			&item.ID,
			&item.ClientID,
			&item.ClientType,
			&authRaw,
			&item.ActiveTimeout,
			&item.Timeout,
			&item.Status,
			&createUserID,
			&createAt,
			&updateUserID,
			&updateAt,
			&createUserStr,
			&updateUserStr,
		); err != nil {
			Fail(c, "500", "解析客户端数据失败")
			return
		}
		item.CreateUser = createUserStr
		item.UpdateUser = updateUserStr
		item.CreateUserString = createUserStr
		item.UpdateUserString = updateUserStr
		item.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			item.UpdateTime = formatTime(updateAt.Time)
		}
		if len(authRaw) > 0 {
			var authSlice []string
			if err := json.Unmarshal(authRaw, &authSlice); err == nil {
				item.AuthType = authSlice
			}
		}
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询客户端失败")
		return
	}
	OK(c, PageResult[ClientResp]{List: list, Total: total})
}

// GetClient 处理 GET /system/client/:id。
func (h *ClientHandler) GetClient(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT c.id,
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
       COALESCE(cu.nickname, ''),
       COALESCE(uu.nickname, '')
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
WHERE c.id = $1;
`

	var (
		resp          ClientDetailResp
		authRaw       []byte
		createUserID  sql.NullInt64
		updateUserID  sql.NullInt64
		createAt      time.Time
		updateAt      sql.NullTime
		createUserStr string
		updateUserStr string
	)
	if err := h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&resp.ID,
			&resp.ClientID,
			&resp.ClientType,
			&authRaw,
			&resp.ActiveTimeout,
			&resp.Timeout,
			&resp.Status,
			&createUserID,
			&createAt,
			&updateUserID,
			&updateAt,
			&createUserStr,
			&updateUserStr,
		); err != nil {
		if err == sql.ErrNoRows {
			Fail(c, "404", "客户端不存在")
			return
		}
		Fail(c, "500", "查询客户端失败")
		return
	}
	resp.CreateUser = createUserStr
	resp.UpdateUser = updateUserStr
	resp.CreateUserString = createUserStr
	resp.UpdateUserString = updateUserStr
	resp.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		resp.UpdateTime = formatTime(updateAt.Time)
	}
	if len(authRaw) > 0 {
		var authSlice []string
		if err := json.Unmarshal(authRaw, &authSlice); err == nil {
			resp.AuthType = authSlice
		}
	}
	OK(c, resp)
}

// CreateClient 处理 POST /system/client。
func (h *ClientHandler) CreateClient(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req clientReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.ClientType = strings.TrimSpace(req.ClientType)
	if req.ClientType == "" || len(req.AuthType) == 0 {
		Fail(c, "400", "客户端类型和认证类型不能为空")
		return
	}
	if req.ActiveTimeout == 0 {
		req.ActiveTimeout = 1800
	}
	if req.Timeout == 0 {
		req.Timeout = 86400
	}
	if req.Status == 0 {
		req.Status = 1
	}

	// 生成客户端 ID，使用随机雪花 ID 的 hex 形式，保证唯一且长度适中。
	clientID := fmt.Sprintf("%x", id.Next())
	authJSON, err := json.Marshal(req.AuthType)
	if err != nil {
		Fail(c, "500", "保存客户端失败")
		return
	}

	now := time.Now()
	idVal := id.Next()

	const stmt = `
INSERT INTO sys_client (
    id, client_id, client_type, auth_type,
    active_timeout, timeout, status,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4,
    $5, $6, $7,
    $8, $9
);
`
	if _, err := h.db.ExecContext(
		c.Request.Context(),
		stmt,
		idVal,
		clientID,
		req.ClientType,
		string(authJSON),
		req.ActiveTimeout,
		req.Timeout,
		req.Status,
		userID,
		now,
	); err != nil {
		Fail(c, "500", "新增客户端失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateClient 处理 PUT /system/client/:id。
func (h *ClientHandler) UpdateClient(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req clientReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.ClientType = strings.TrimSpace(req.ClientType)
	if req.ClientType == "" || len(req.AuthType) == 0 {
		Fail(c, "400", "客户端类型和认证类型不能为空")
		return
	}
	if req.Status == 0 {
		req.Status = 1
	}

	authJSON, err := json.Marshal(req.AuthType)
	if err != nil {
		Fail(c, "500", "保存客户端失败")
		return
	}

	const stmt = `
UPDATE sys_client
   SET client_type = $1,
       auth_type = $2,
       active_timeout = $3,
       timeout = $4,
       status = $5,
       update_user = $6,
       update_time = $7
 WHERE id = $8;
`
	if _, err := h.db.ExecContext(
		c.Request.Context(),
		stmt,
		req.ClientType,
		string(authJSON),
		req.ActiveTimeout,
		req.Timeout,
		req.Status,
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "修改客户端失败")
		return
	}
	OK(c, true)
}

// DeleteClient 处理 DELETE /system/client。
func (h *ClientHandler) DeleteClient(c *gin.Context) {
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
		Fail(c, "500", "删除客户端失败")
		return
	}
	defer tx.Rollback()

	for _, idVal := range req.IDs {
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_client WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除客户端失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除客户端失败")
		return
	}
	OK(c, true)
}

