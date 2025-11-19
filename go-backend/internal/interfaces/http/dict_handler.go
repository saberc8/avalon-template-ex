package http

import (
	"database/sql"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// DictResp matches admin/src/apis/system/type.ts -> DictResp.
type DictResp struct {
	ID               int64  `json:"id"`
	Name             string `json:"name"`
	Code             string `json:"code"`
	IsSystem         bool   `json:"isSystem"`
	Description      string `json:"description"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
	UpdateUserString string `json:"updateUserString"`
	UpdateTime       string `json:"updateTime"`
}

// DictItemResp matches DictItemResp type on the front-end.
type DictItemResp struct {
	ID               int64  `json:"id"`
	Label            string `json:"label"`
	Value            string `json:"value"`
	Color            string `json:"color"`
	Sort             int32  `json:"sort"`
	Description      string `json:"description"`
	Status           int16  `json:"status"`
	DictID           int64  `json:"dictId"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
	UpdateUserString string `json:"updateUserString"`
	UpdateTime       string `json:"updateTime"`
}

type dictReq struct {
	Name        string `json:"name"`
	Code        string `json:"code"`
	Description string `json:"description"`
}

type dictItemReq struct {
	Label       string `json:"label"`
	Value       string `json:"value"`
	Color       string `json:"color"`
	Sort        int32  `json:"sort"`
	Description string `json:"description"`
	Status      int16  `json:"status"`
	DictID      int64  `json:"dictId"` // only used on create
}

// DictHandler provides /system/dict and /system/dict/item endpoints.
// It talks directly to PostgreSQL and uses JWT to obtain the current user.
type DictHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewDictHandler(db *sql.DB, tokenSvc *security.TokenService) *DictHandler {
	return &DictHandler{db: db, tokenSvc: tokenSvc}
}

// RegisterDictRoutes registers dictionary management routes.
func (h *DictHandler) RegisterDictRoutes(r *gin.Engine) {
	// 字典本身
	r.GET("/system/dict/list", h.ListDict)
	r.GET("/system/dict/:id", h.GetDict)
	r.POST("/system/dict", h.CreateDict)
	r.PUT("/system/dict/:id", h.UpdateDict)
	r.DELETE("/system/dict", h.DeleteDict)
	r.DELETE("/system/dict/cache/:code", h.ClearDictCache)

	// 字典项
	r.GET("/system/dict/item", h.ListDictItem)
	r.GET("/system/dict/item/:id", h.GetDictItem)
	r.POST("/system/dict/item", h.CreateDictItem)
	r.PUT("/system/dict/item/:id", h.UpdateDictItem)
	r.DELETE("/system/dict/item", h.DeleteDictItem)
}

func formatTimePtr(t *time.Time) string {
	if t == nil || t.IsZero() {
		return ""
	}
	return t.Format("2006-01-02 15:04:05")
}

func formatTime(t time.Time) string {
	if t.IsZero() {
		return ""
	}
	return t.Format("2006-01-02 15:04:05")
}

// currentUserID parses token and returns userID, or 0 if unauthorized (and writes 401).
func (h *DictHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListDict handles GET /system/dict/list
func (h *DictHandler) ListDict(c *gin.Context) {
	description := strings.TrimSpace(c.Query("description"))

	const query = `
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, ''),
       COALESCE(d.is_system, FALSE),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
ORDER BY d.create_time DESC, d.id DESC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询字典失败")
		return
	}
	defer rows.Close()

	var list []DictResp
	for rows.Next() {
		var (
			id        int64
			name      string
			code      string
			desc      string
			isSystem  bool
			createAt  time.Time
			createBy  string
			updateAt  sql.NullTime
			updateBy  string
		)
		if err := rows.Scan(&id, &name, &code, &desc, &isSystem, &createAt, &createBy, &updateAt, &updateBy); err != nil {
			Fail(c, "500", "解析字典数据失败")
			return
		}
		if description != "" && !strings.Contains(name, description) && !strings.Contains(desc, description) {
			continue
		}
		item := DictResp{
			ID:               id,
			Name:             name,
			Code:             code,
			IsSystem:         isSystem,
			Description:      desc,
			CreateUserString: createBy,
			CreateTime:       formatTime(createAt),
			UpdateUserString: updateBy,
			UpdateTime:       formatTimePtr(&updateAt.Time),
		}
		if !updateAt.Valid {
			item.UpdateTime = ""
		}
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询字典失败")
		return
	}
	OK(c, list)
}

// GetDict handles GET /system/dict/:id
func (h *DictHandler) GetDict(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || id <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, ''),
       COALESCE(d.is_system, FALSE),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = $1;
`
	var (
		resp      DictResp
		isSystem  bool
		createAt  time.Time
		createBy  string
		updateAt  sql.NullTime
		updateBy  string
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, id).
		Scan(&resp.ID, &resp.Name, &resp.Code, &resp.Description, &isSystem, &createAt, &createBy, &updateAt, &updateBy)
	if err == sql.ErrNoRows {
		Fail(c, "404", "字典不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询字典失败")
		return
	}
	resp.IsSystem = isSystem
	resp.CreateUserString = createBy
	resp.CreateTime = formatTime(createAt)
	resp.UpdateUserString = updateBy
	if updateAt.Valid {
		resp.UpdateTime = formatTime(updateAt.Time)
	}
	OK(c, resp)
}

// CreateDict handles POST /system/dict
func (h *DictHandler) CreateDict(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req dictReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	req.Code = strings.TrimSpace(req.Code)
	if req.Name == "" || req.Code == "" {
		Fail(c, "400", "名称和编码不能为空")
		return
	}

	now := time.Now()
	idVal := id.Next()

	const stmt = `
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
VALUES ($1, $2, $3, $4, FALSE, $5, $6);
`
	_, err := h.db.ExecContext(c.Request.Context(), stmt, idVal, req.Name, req.Code, req.Description, userID, now)
	if err != nil {
		Fail(c, "500", "新增字典失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateDict handles PUT /system/dict/:id
func (h *DictHandler) UpdateDict(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req dictReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		Fail(c, "400", "名称不能为空")
		return
	}

	const stmt = `
UPDATE sys_dict
   SET name = $1,
       description = $2,
       update_user = $3,
       update_time = $4
 WHERE id = $5;
`
	_, err = h.db.ExecContext(c.Request.Context(), stmt, req.Name, req.Description, userID, time.Now(), idVal)
	if err != nil {
		Fail(c, "500", "修改字典失败")
		return
	}
	OK(c, true)
}

type idsRequest struct {
	IDs []int64 `json:"ids"`
}

// DeleteDict handles DELETE /system/dict
func (h *DictHandler) DeleteDict(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	_ = userID // reserved for audit; current implementation does not record deleter.

	var req idsRequest
	if err := c.ShouldBindJSON(&req); err != nil || len(req.IDs) == 0 {
		Fail(c, "400", "ID 列表不能为空")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "删除字典失败")
		return
	}
	defer tx.Rollback()

	// Also delete related dict items.
	for _, idVal := range req.IDs {
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_dict_item WHERE dict_id = $1`, idVal); err != nil {
			Fail(c, "500", "删除字典项失败")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_dict WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除字典失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除字典失败")
		return
	}
	OK(c, true)
}

// ClearDictCache handles DELETE /system/dict/cache/:code
// The current Go backend does not use Redis cache yet, so this is a no-op.
func (h *DictHandler) ClearDictCache(c *gin.Context) {
	// code := c.Param("code") // kept for future cache integration
	OK(c, true)
}

// ListDictItem handles GET /system/dict/item (分页查询字典项)
func (h *DictHandler) ListDictItem(c *gin.Context) {
	dictIDStr := c.Query("dictId")
	if dictIDStr == "" {
		Fail(c, "400", "字典 ID 不能为空")
		return
	}
	dictID, err := strconv.ParseInt(dictIDStr, 10, 64)
	if err != nil || dictID <= 0 {
		Fail(c, "400", "字典 ID 不正确")
		return
	}
	page, _ := strconv.Atoi(c.Query("page"))
	size, _ := strconv.Atoi(c.Query("size"))
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}
	description := strings.TrimSpace(c.Query("description"))
	statusStr := strings.TrimSpace(c.Query("status"))
	var statusFilter int64
	if statusStr != "" {
		statusFilter, _ = strconv.ParseInt(statusStr, 10, 64)
	}

	const query = `
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, ''),
       COALESCE(di.sort, 999),
       COALESCE(di.description, ''),
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, ''),
       di.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
WHERE di.dict_id = $1
ORDER BY di.sort ASC, di.id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query, dictID)
	if err != nil {
		Fail(c, "500", "查询字典项失败")
		return
	}
	defer rows.Close()

	var all []DictItemResp
	for rows.Next() {
		var (
			item      DictItemResp
			createAt  time.Time
			createBy  string
			updateAt  sql.NullTime
			updateBy  string
		)
		if err := rows.Scan(
			&item.ID,
			&item.Label,
			&item.Value,
			&item.Color,
			&item.Sort,
			&item.Description,
			&item.Status,
			&item.DictID,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		); err != nil {
			Fail(c, "500", "解析字典项失败")
			return
		}

		if description != "" &&
			!strings.Contains(item.Label, description) &&
			!strings.Contains(item.Description, description) {
			continue
		}
		if statusFilter != 0 && int64(item.Status) != statusFilter {
			continue
		}

		item.CreateUserString = createBy
		item.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			item.UpdateUserString = updateBy
			item.UpdateTime = formatTime(updateAt.Time)
		}
		all = append(all, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询字典项失败")
		return
	}

	total := int64(len(all))
	start := (page - 1) * size
	if start > len(all) {
		start = len(all)
	}
	end := start + size
	if end > len(all) {
		end = len(all)
	}
	pageList := all[start:end]

	OK(c, PageResult[DictItemResp]{List: pageList, Total: total})
}

// GetDictItem handles GET /system/dict/item/:id
func (h *DictHandler) GetDictItem(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, ''),
       COALESCE(di.sort, 999),
       COALESCE(di.description, ''),
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, ''),
       di.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
WHERE di.id = $1;
`
	var (
		item      DictItemResp
		createAt  time.Time
		createBy  string
		updateAt  sql.NullTime
		updateBy  string
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&item.ID,
			&item.Label,
			&item.Value,
			&item.Color,
			&item.Sort,
			&item.Description,
			&item.Status,
			&item.DictID,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		)
	if err == sql.ErrNoRows {
		Fail(c, "404", "字典项不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询字典项失败")
		return
	}
	item.CreateUserString = createBy
	item.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		item.UpdateUserString = updateBy
		item.UpdateTime = formatTime(updateAt.Time)
	}
	OK(c, item)
}

// CreateDictItem handles POST /system/dict/item
func (h *DictHandler) CreateDictItem(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req dictItemReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Label = strings.TrimSpace(req.Label)
	req.Value = strings.TrimSpace(req.Value)
	if req.Label == "" || req.Value == "" || req.DictID == 0 {
		Fail(c, "400", "标签、值和字典 ID 不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.Status == 0 {
		req.Status = 1
	}

	now := time.Now()
	idVal := id.Next()

	const stmt = `
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status, dict_id,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
`
	_, err := h.db.ExecContext(
		c.Request.Context(),
		stmt,
		idVal,
		req.Label,
		req.Value,
		req.Color,
		req.Sort,
		req.Description,
		req.Status,
		req.DictID,
		userID,
		now,
	)
	if err != nil {
		Fail(c, "500", "新增字典项失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateDictItem handles PUT /system/dict/item/:id
func (h *DictHandler) UpdateDictItem(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req dictItemReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Label = strings.TrimSpace(req.Label)
	req.Value = strings.TrimSpace(req.Value)
	if req.Label == "" || req.Value == "" {
		Fail(c, "400", "标签和值不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.Status == 0 {
		req.Status = 1
	}

	const stmt = `
UPDATE sys_dict_item
   SET label       = $1,
       value       = $2,
       color       = $3,
       sort        = $4,
       description = $5,
       status      = $6,
       update_user = $7,
       update_time = $8
 WHERE id          = $9;
`
	_, err = h.db.ExecContext(
		c.Request.Context(),
		stmt,
		req.Label,
		req.Value,
		req.Color,
		req.Sort,
		req.Description,
		req.Status,
		userID,
		time.Now(),
		idVal,
	)
	if err != nil {
		Fail(c, "500", "修改字典项失败")
		return
	}
	OK(c, true)
}

// DeleteDictItem handles DELETE /system/dict/item
func (h *DictHandler) DeleteDictItem(c *gin.Context) {
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
		Fail(c, "500", "删除字典项失败")
		return
	}
	defer tx.Rollback()

	for _, idVal := range req.IDs {
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_dict_item WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除字典项失败")
			return
		}
	}
	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除字典项失败")
		return
	}
	OK(c, true)
}
