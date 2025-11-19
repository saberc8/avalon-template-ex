package http

import (
	"database/sql"
	"database/sql/driver"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// MenuResp matches MenuResp in admin/src/apis/system/type.ts.
type MenuResp struct {
	ID               int64      `json:"id"`
	Title            string     `json:"title"`
	ParentID         int64      `json:"parentId"`
	Type             int16      `json:"type"`
	Path             string     `json:"path"`
	Name             string     `json:"name"`
	Component        string     `json:"component"`
	Redirect         string     `json:"redirect"`
	Icon             string     `json:"icon"`
	IsExternal       bool       `json:"isExternal"`
	IsCache          bool       `json:"isCache"`
	IsHidden         bool       `json:"isHidden"`
	Permission       string     `json:"permission"`
	Sort             int32      `json:"sort"`
	Status           int16      `json:"status"`
	CreateUserString string     `json:"createUserString"`
	CreateTime       string     `json:"createTime"`
	UpdateUserString string     `json:"updateUserString"`
	UpdateTime       string     `json:"updateTime"`
	Children         []MenuResp `json:"children"`
}

type menuReq struct {
	Type       int16  `json:"type"`
	Icon       string `json:"icon"`
	Title      string `json:"title"`
	Sort       int32  `json:"sort"`
	Permission string `json:"permission"`
	Path       string `json:"path"`
	Name       string `json:"name"`
	Component  string `json:"component"`
	Redirect   string `json:"redirect"`
	IsExternal *bool  `json:"isExternal"`
	IsCache    *bool  `json:"isCache"`
	IsHidden   *bool  `json:"isHidden"`
	ParentID   int64  `json:"parentId"`
	Status     int16  `json:"status"`
}

// MenuHandler provides /system/menu endpoints.
type MenuHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewMenuHandler(db *sql.DB, tokenSvc *security.TokenService) *MenuHandler {
	return &MenuHandler{db: db, tokenSvc: tokenSvc}
}

// RegisterMenuRoutes registers menu management routes.
func (h *MenuHandler) RegisterMenuRoutes(r *gin.Engine) {
	r.GET("/system/menu/tree", h.ListMenuTree)
	r.GET("/system/menu/:id", h.GetMenu)
	r.POST("/system/menu", h.CreateMenu)
	r.PUT("/system/menu/:id", h.UpdateMenu)
	r.DELETE("/system/menu", h.DeleteMenu)
	r.DELETE("/system/menu/cache", h.ClearMenuCache)
}

// ListMenuTree handles GET /system/menu/tree.
func (h *MenuHandler) ListMenuTree(c *gin.Context) {
	const query = `
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, ''),
       COALESCE(m.name, ''),
       COALESCE(m.component, ''),
       COALESCE(m.redirect, ''),
       COALESCE(m.icon, ''),
       COALESCE(m.is_external, FALSE),
       COALESCE(m.is_cache, FALSE),
       COALESCE(m.is_hidden, FALSE),
       COALESCE(m.permission, ''),
       COALESCE(m.sort, 0),
       COALESCE(m.status, 1),
       m.create_time,
       COALESCE(cu.nickname, ''),
       m.update_time,
       COALESCE(uu.nickname, '')
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
ORDER BY m.sort ASC, m.id ASC;
`

	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询菜单失败")
		return
	}
	defer rows.Close()

	var flat []MenuResp
	for rows.Next() {
		var (
			item      MenuResp
			createAt  time.Time
			createBy  string
			updateAt  sql.NullTime
			updateBy  string
		)
		if err := rows.Scan(
			&item.ID,
			&item.Title,
			&item.ParentID,
			&item.Type,
			&item.Path,
			&item.Name,
			&item.Component,
			&item.Redirect,
			&item.Icon,
			&item.IsExternal,
			&item.IsCache,
			&item.IsHidden,
			&item.Permission,
			&item.Sort,
			&item.Status,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		); err != nil {
			Fail(c, "500", "解析菜单数据失败")
			return
		}
		item.CreateUserString = createBy
		item.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			item.UpdateUserString = updateBy
			item.UpdateTime = formatTime(updateAt.Time)
		}
		item.Children = []MenuResp{}
		flat = append(flat, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询菜单失败")
		return
	}

	// Build tree by parentId.
	nodeMap := make(map[int64]*MenuResp, len(flat))
	var roots []MenuResp
	for i := range flat {
		nodeMap[flat[i].ID] = &flat[i]
	}
	// First, connect children using pointers only.
	for _, node := range nodeMap {
		if node.ParentID == 0 {
			continue
		}
		if parent, ok := nodeMap[node.ParentID]; ok {
			parent.Children = append(parent.Children, *node)
		}
	}
	// Then, copy roots (and orphans) out to value slice.
	for _, node := range nodeMap {
		if node.ParentID == 0 {
			roots = append(roots, *node)
		} else if _, ok := nodeMap[node.ParentID]; !ok {
			// Orphan node: treat as root.
			roots = append(roots, *node)
		}
	}

	OK(c, roots)
}

// GetMenu handles GET /system/menu/:id.
func (h *MenuHandler) GetMenu(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, ''),
       COALESCE(m.name, ''),
       COALESCE(m.component, ''),
       COALESCE(m.redirect, ''),
       COALESCE(m.icon, ''),
       COALESCE(m.is_external, FALSE),
       COALESCE(m.is_cache, FALSE),
       COALESCE(m.is_hidden, FALSE),
       COALESCE(m.permission, ''),
       COALESCE(m.sort, 0),
       COALESCE(m.status, 1),
       m.create_time,
       COALESCE(cu.nickname, ''),
       m.update_time,
       COALESCE(uu.nickname, '')
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
WHERE m.id = $1;
`

	var (
		item      MenuResp
		createAt  time.Time
		createBy  string
		updateAt  sql.NullTime
		updateBy  string
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&item.ID,
			&item.Title,
			&item.ParentID,
			&item.Type,
			&item.Path,
			&item.Name,
			&item.Component,
			&item.Redirect,
			&item.Icon,
			&item.IsExternal,
			&item.IsCache,
			&item.IsHidden,
			&item.Permission,
			&item.Sort,
			&item.Status,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		)
	if err == sql.ErrNoRows {
		Fail(c, "404", "菜单不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询菜单失败")
		return
	}
	item.CreateUserString = createBy
	item.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		item.UpdateUserString = updateBy
		item.UpdateTime = formatTime(updateAt.Time)
	}
	item.Children = []MenuResp{}
	OK(c, item)
}

// helper to parse auth header and return user id.
func (h *MenuHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// CreateMenu handles POST /system/menu.
func (h *MenuHandler) CreateMenu(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req menuReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	if req.Type == 0 {
		req.Type = 1
	}
	req.Title = strings.TrimSpace(req.Title)
	if req.Title == "" {
		Fail(c, "400", "菜单标题不能为空")
		return
	}
	// 默认布尔值
	isExternal := false
	if req.IsExternal != nil {
		isExternal = *req.IsExternal
	}
	isCache := false
	if req.IsCache != nil {
		isCache = *req.IsCache
	}
	isHidden := false
	if req.IsHidden != nil {
		isHidden = *req.IsHidden
	}

	// 按 Java 逻辑处理外链和路由
	req.Path = strings.TrimSpace(req.Path)
	req.Name = strings.TrimSpace(req.Name)
	req.Component = strings.TrimSpace(req.Component)
	if isExternal {
		if !(strings.HasPrefix(req.Path, "http://") || strings.HasPrefix(req.Path, "https://")) {
			Fail(c, "400", "路由地址格式不正确，请以 http:// 或 https:// 开头")
			return
		}
	} else {
		if strings.HasPrefix(req.Path, "http://") || strings.HasPrefix(req.Path, "https://") {
			Fail(c, "400", "路由地址格式不正确")
			return
		}
		if req.Path != "" && !strings.HasPrefix(req.Path, "/") {
			req.Path = "/" + req.Path
		}
		req.Name = strings.TrimPrefix(req.Name, "/")
		req.Component = strings.TrimPrefix(req.Component, "/")
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
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect,
    icon, is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
)
VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8,
    $9, $10, $11, $12, $13, $14, $15,
    $16, $17
);
`
	_, err := h.db.ExecContext(
		c.Request.Context(),
		stmt,
		idVal,
		req.Title,
		req.ParentID,
		req.Type,
		req.Path,
		req.Name,
		req.Component,
		req.Redirect,
		req.Icon,
		isExternal,
		isCache,
		isHidden,
		req.Permission,
		req.Sort,
		req.Status,
		userID,
		now,
	)
	if err != nil {
		Fail(c, "500", "新增菜单失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateMenu handles PUT /system/menu/:id.
func (h *MenuHandler) UpdateMenu(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req menuReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Title = strings.TrimSpace(req.Title)
	if req.Title == "" {
		Fail(c, "400", "菜单标题不能为空")
		return
	}

	isExternal := false
	if req.IsExternal != nil {
		isExternal = *req.IsExternal
	}
	isCache := false
	if req.IsCache != nil {
		isCache = *req.IsCache
	}
	isHidden := false
	if req.IsHidden != nil {
		isHidden = *req.IsHidden
	}

	req.Path = strings.TrimSpace(req.Path)
	req.Name = strings.TrimSpace(req.Name)
	req.Component = strings.TrimSpace(req.Component)
	if isExternal {
		if !(strings.HasPrefix(req.Path, "http://") || strings.HasPrefix(req.Path, "https://")) {
			Fail(c, "400", "路由地址格式不正确，请以 http:// 或 https:// 开头")
			return
		}
	} else {
		if strings.HasPrefix(req.Path, "http://") || strings.HasPrefix(req.Path, "https://") {
			Fail(c, "400", "路由地址格式不正确")
			return
		}
		if req.Path != "" && !strings.HasPrefix(req.Path, "/") {
			req.Path = "/" + req.Path
		}
		req.Name = strings.TrimPrefix(req.Name, "/")
		req.Component = strings.TrimPrefix(req.Component, "/")
	}

	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.Status == 0 {
		req.Status = 1
	}

	const stmt = `
UPDATE sys_menu
   SET title       = $1,
       parent_id   = $2,
       type        = $3,
       path        = $4,
       name        = $5,
       component   = $6,
       redirect    = $7,
       icon        = $8,
       is_external = $9,
       is_cache    = $10,
       is_hidden   = $11,
       permission  = $12,
       sort        = $13,
       status      = $14,
       update_user = $15,
       update_time = $16
 WHERE id          = $17;
`
	_, err = h.db.ExecContext(
		c.Request.Context(),
		stmt,
		req.Title,
		req.ParentID,
		req.Type,
		req.Path,
		req.Name,
		req.Component,
		req.Redirect,
		req.Icon,
		isExternal,
		isCache,
		isHidden,
		req.Permission,
		req.Sort,
		req.Status,
		userID,
		time.Now(),
		idVal,
	)
	if err != nil {
		Fail(c, "500", "修改菜单失败")
		return
	}
	OK(c, true)
}

// DeleteMenu handles DELETE /system/menu.
func (h *MenuHandler) DeleteMenu(c *gin.Context) {
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

	// Load all menus to compute descendants.
	const query = `SELECT id, parent_id FROM sys_menu;`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}
	defer rows.Close()

	type node struct {
		id       int64
		parentID int64
	}
	var all []node
	for rows.Next() {
		var n node
		if err := rows.Scan(&n.id, &n.parentID); err != nil {
			Fail(c, "500", "删除菜单失败")
			return
		}
		all = append(all, n)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}

	childrenOf := make(map[int64][]int64)
	for _, n := range all {
		childrenOf[n.parentID] = append(childrenOf[n.parentID], n.id)
	}

	seen := make(map[int64]struct{})
	var collect func(id int64)
	collect = func(id int64) {
		if _, ok := seen[id]; ok {
			return
		}
		seen[id] = struct{}{}
		for _, ch := range childrenOf[id] {
			collect(ch)
		}
	}
	for _, idVal := range req.IDs {
		collect(idVal)
	}
	if len(seen) == 0 {
		OK(c, true)
		return
	}
	allIDs := make([]int64, 0, len(seen))
	for idVal := range seen {
		allIDs = append(allIDs, idVal)
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}
	defer tx.Rollback()

	// Remove role-menu relations first.
	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role_menu WHERE menu_id = ANY($1::bigint[])`, pqInt64Array(allIDs)); err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}
	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_menu WHERE id = ANY($1::bigint[])`, pqInt64Array(allIDs)); err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}
	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除菜单失败")
		return
	}
	OK(c, true)
}

// ClearMenuCache handles DELETE /system/menu/cache.
// The Go backend does not use Redis-based menu cache yet, so this is a no-op.
func (h *MenuHandler) ClearMenuCache(c *gin.Context) {
	OK(c, true)
}

// pqInt64Array wraps []int64 for simple ANY($1) usage.
type pqInt64Array []int64

func (a pqInt64Array) Value() (driver.Value, error) {
	if len(a) == 0 {
		return "{}", nil
	}
	// simple text array representation: {1,2,3}
	var sb strings.Builder
	sb.WriteByte('{')
	for i, v := range a {
		if i > 0 {
			sb.WriteByte(',')
		}
		sb.WriteString(strconv.FormatInt(v, 10))
	}
	sb.WriteByte('}')
	return sb.String(), nil
}
