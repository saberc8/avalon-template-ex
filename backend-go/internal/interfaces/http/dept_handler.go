package http

import (
	"database/sql"
	"encoding/csv"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/lib/pq"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// DeptResp matches DeptResp in admin/src/apis/system/type.ts.
type DeptResp struct {
	ID               int64      `json:"id"`
	Name             string     `json:"name"`
	Sort             int32      `json:"sort"`
	Status           int16      `json:"status"`
	IsSystem         bool       `json:"isSystem"`
	Description      string     `json:"description"`
	CreateUserString string     `json:"createUserString"`
	CreateTime       string     `json:"createTime"`
	UpdateUserString string     `json:"updateUserString"`
	UpdateTime       string     `json:"updateTime"`
	ParentID         int64      `json:"parentId"`
	Children         []DeptResp `json:"children"`
}

// DeptQuery represents /system/dept/tree query parameters.
type DeptQuery struct {
	Description string
	Status      int64
}

// deptReq represents create/update request body for department.
type deptReq struct {
	Name        string `json:"name"`
	ParentID    int64  `json:"parentId"`
	Sort        int32  `json:"sort"`
	Status      int16  `json:"status"`
	Description string `json:"description"`
}

// DeptHandler provides /system/dept endpoints.
type DeptHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewDeptHandler(db *sql.DB, tokenSvc *security.TokenService) *DeptHandler {
	return &DeptHandler{
		db:       db,
		tokenSvc: tokenSvc,
	}
}

// RegisterDeptRoutes registers /system/dept related routes.
func (h *DeptHandler) RegisterDeptRoutes(r *gin.Engine) {
	r.GET("/system/dept/tree", h.ListDeptTree)
	r.GET("/system/dept/:id", h.GetDept)
	r.POST("/system/dept", h.CreateDept)
	r.PUT("/system/dept/:id", h.UpdateDept)
	r.DELETE("/system/dept", h.DeleteDept)
	r.GET("/system/dept/export", h.ExportDept)
}

// currentUserID extracts user id from JWT, similar to SystemUserHandler.currentUserID.
func (h *DeptHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListDeptTree handles GET /system/dept/tree and returns a department tree list
// with the full DeptResp structure, keeping response compatible with the front-end.
func (h *DeptHandler) ListDeptTree(c *gin.Context) {
	// Parse filters from query
	desc := strings.TrimSpace(c.Query("description"))
	statusStr := strings.TrimSpace(c.Query("status"))

	var status int64
	if statusStr != "" {
		if v, err := strconv.ParseInt(statusStr, 10, 64); err == nil && v > 0 {
			status = v
		}
	}

	// Build basic query; we join creator/updater nickname to match *UserString fields.
	where := "WHERE 1=1"
	args := []any{}
	argPos := 1
	if desc != "" {
		where += " AND (d.name ILIKE $" + strconv.Itoa(argPos) + " OR COALESCE(d.description,'') ILIKE $" + strconv.Itoa(argPos) + ")"
		args = append(args, "%"+desc+"%")
		argPos++
	}
	if status != 0 {
		where += " AND d.status = $" + strconv.Itoa(argPos)
		args = append(args, status)
		argPos++
	}

	query := `
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
` + where + `
ORDER BY d.sort ASC, d.id ASC;
`

	rows, err := h.db.QueryContext(c.Request.Context(), query, args...)
	if err != nil {
		Fail(c, "500", "查询部门失败")
		return
	}
	defer rows.Close()

	type deptRow struct {
		id          int64
		name        string
		parentID    int64
		sort        int32
		status      int16
		isSystem    bool
		description string
		createTime  time.Time
		createUser  string
		updateTime  sql.NullTime
		updateUser  string
	}

	var flat []deptRow
	for rows.Next() {
		var d deptRow
		if err := rows.Scan(
			&d.id,
			&d.name,
			&d.parentID,
			&d.sort,
			&d.status,
			&d.isSystem,
			&d.description,
			&d.createTime,
			&d.createUser,
			&d.updateTime,
			&d.updateUser,
		); err != nil {
			Fail(c, "500", "解析部门数据失败")
			return
		}
		flat = append(flat, d)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询部门失败")
		return
	}

	if len(flat) == 0 {
		OK(c, []DeptResp{})
		return
	}

	// Build id -> node map first
	nodeMap := make(map[int64]*DeptResp, len(flat))
	for _, d := range flat {
		resp := &DeptResp{
			ID:               d.id,
			Name:             d.name,
			Sort:             d.sort,
			Status:           d.status,
			IsSystem:         d.isSystem,
			Description:      d.description,
			CreateUserString: d.createUser,
			CreateTime:       d.createTime.Format(time.RFC3339),
			UpdateUserString: d.updateUser,
			ParentID:         d.parentID,
		}
		if d.updateTime.Valid {
			resp.UpdateTime = d.updateTime.Time.Format(time.RFC3339)
		}
		nodeMap[d.id] = resp
	}

	// Assemble tree structure
	var roots []*DeptResp
	for _, d := range flat {
		node := nodeMap[d.id]
		if d.parentID == 0 {
			roots = append(roots, node)
			continue
		}
		parent, ok := nodeMap[d.parentID]
		if !ok {
			roots = append(roots, node)
			continue
		}
		parent.Children = append(parent.Children, *node)
	}

	result := make([]DeptResp, 0, len(roots))
	for _, n := range roots {
		result = append(result, *n)
	}
	OK(c, result)
}

// GetDept handles GET /system/dept/:id and returns single department detail.
func (h *DeptHandler) GetDept(c *gin.Context) {
	idStr := c.Param("id")
	idVal, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "无效的部门 ID")
		return
	}

	const query = `
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = $1;
`

	var row struct {
		id          int64
		name        string
		parentID    int64
		sort        int32
		status      int16
		isSystem    bool
		description string
		createTime  time.Time
		createUser  string
		updateTime  sql.NullTime
		updateUser  string
	}

	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).Scan(
		&row.id,
		&row.name,
		&row.parentID,
		&row.sort,
		&row.status,
		&row.isSystem,
		&row.description,
		&row.createTime,
		&row.createUser,
		&row.updateTime,
		&row.updateUser,
	)
	if err == sql.ErrNoRows {
		Fail(c, "404", "部门不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询部门失败")
		return
	}

	resp := DeptResp{
		ID:               row.id,
		Name:             row.name,
		Sort:             row.sort,
		Status:           row.status,
		IsSystem:         row.isSystem,
		Description:      row.description,
		CreateUserString: row.createUser,
		CreateTime:       row.createTime.Format(time.RFC3339),
		UpdateUserString: row.updateUser,
		ParentID:         row.parentID,
	}
	if row.updateTime.Valid {
		resp.UpdateTime = row.updateTime.Time.Format(time.RFC3339)
	}

	OK(c, resp)
}

// CreateDept handles POST /system/dept.
func (h *DeptHandler) CreateDept(c *gin.Context) {
	var req deptReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "参数错误")
		return
	}
	if strings.TrimSpace(req.Name) == "" {
		Fail(c, "400", "名称不能为空")
		return
	}
	if req.ParentID == 0 {
		Fail(c, "400", "上级部门不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 1
	}
	if req.Status == 0 {
		req.Status = 1
	}

	// Check name uniqueness under same parent.
	const checkSQL = `
SELECT 1
FROM sys_dept
WHERE name = $1 AND parent_id = $2
LIMIT 1;
`
	var dummy int
	if err := h.db.QueryRowContext(c.Request.Context(), checkSQL, req.Name, req.ParentID).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验部门名称失败")
		return
	} else if err == nil {
		Fail(c, "400", "新增失败，该名称在当前上级下已存在")
		return
	}

	// Ensure parent exists.
	const parentCheck = `SELECT 1 FROM sys_dept WHERE id = $1;`
	if err := h.db.QueryRowContext(c.Request.Context(), parentCheck, req.ParentID).Scan(&dummy); err == sql.ErrNoRows {
		Fail(c, "400", "上级部门不存在")
		return
	} else if err != nil {
		Fail(c, "500", "校验上级部门失败")
		return
	}

	now := time.Now()
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	newID := id.Next()
	const insertSQL = `
INSERT INTO sys_dept (
    id, name, parent_id, sort, status, is_system, description,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, FALSE, $6,
    $7, $8
);
`
	if _, err := h.db.ExecContext(c.Request.Context(), insertSQL,
		newID,
		req.Name,
		req.ParentID,
		req.Sort,
		req.Status,
		strings.TrimSpace(req.Description),
		userID,
		now,
	); err != nil {
		Fail(c, "500", "新增部门失败")
		return
	}

	OK(c, true)
}

// UpdateDept handles PUT /system/dept/:id.
func (h *DeptHandler) UpdateDept(c *gin.Context) {
	idStr := c.Param("id")
	idVal, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "无效的部门 ID")
		return
	}

	var req deptReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "参数错误")
		return
	}
	if strings.TrimSpace(req.Name) == "" {
		Fail(c, "400", "名称不能为空")
		return
	}
	if req.ParentID == 0 {
		Fail(c, "400", "上级部门不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 1
	}
	if req.Status == 0 {
		req.Status = 1
	}

	// Load existing dept.
	const selectSQL = `
SELECT id, name, parent_id, status, is_system
FROM sys_dept
WHERE id = $1;
`
	var oldID, oldParentID int64
	var oldName string
	var oldStatus int16
	var isSystem bool
	if err := h.db.QueryRowContext(c.Request.Context(), selectSQL, idVal).Scan(
		&oldID, &oldName, &oldParentID, &oldStatus, &isSystem,
	); err == sql.ErrNoRows {
		Fail(c, "404", "部门不存在")
		return
	} else if err != nil {
		Fail(c, "500", "查询部门失败")
		return
	}

	// System dept rules (simplified from Java).
	if isSystem {
		if req.Status == 2 {
			Fail(c, "400", "["+oldName+"] 是系统内置部门，不允许禁用")
			return
		}
		if req.ParentID != oldParentID {
			Fail(c, "400", "["+oldName+"] 是系统内置部门，不允许变更上级部门")
			return
		}
	}

	// Check name uniqueness under parent (exclude self).
	const checkSQL = `
SELECT 1
FROM sys_dept
WHERE name = $1 AND parent_id = $2 AND id <> $3
LIMIT 1;
`
	var dummy int
	if err := h.db.QueryRowContext(c.Request.Context(), checkSQL, req.Name, req.ParentID, idVal).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验部门名称失败")
		return
	} else if err == nil {
		Fail(c, "400", "修改失败，该名称在当前上级下已存在")
		return
	}

	now := time.Now()
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	const updateSQL = `
UPDATE sys_dept
SET name = $1,
    parent_id = $2,
    sort = $3,
    status = $4,
    description = $5,
    update_user = $6,
    update_time = $7
WHERE id = $8;
`
	if _, err := h.db.ExecContext(c.Request.Context(), updateSQL,
		req.Name,
		req.ParentID,
		req.Sort,
		req.Status,
		strings.TrimSpace(req.Description),
		userID,
		now,
		idVal,
	); err != nil {
		Fail(c, "500", "修改部门失败")
		return
	}

	OK(c, true)
}

// DeleteDept handles DELETE /system/dept with JSON body: { "ids": [1,2,3] }.
func (h *DeptHandler) DeleteDept(c *gin.Context) {
	var body struct {
		IDs []int64 `json:"ids"`
	}
	if err := c.ShouldBindJSON(&body); err != nil || len(body.IDs) == 0 {
		Fail(c, "400", "参数错误")
		return
	}

	// 1. Prevent deleting system departments.
	const sysCheckSQL = `
SELECT name
FROM sys_dept
WHERE id = ANY($1) AND is_system = TRUE
LIMIT 1;
`
	var sysName string
	if err := h.db.QueryRowContext(c.Request.Context(), sysCheckSQL, pq.Array(body.IDs)).Scan(&sysName); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验系统内置部门失败")
		return
	} else if err == nil {
		Fail(c, "400", "所选部门 ["+sysName+"] 是系统内置部门，不允许删除")
		return
	}

	// 2. Prevent deleting departments that still have children.
	const childCheckSQL = `
SELECT 1
FROM sys_dept
WHERE parent_id = ANY($1)
LIMIT 1;
`
	var dummy int
	if err := h.db.QueryRowContext(c.Request.Context(), childCheckSQL, pq.Array(body.IDs)).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验子部门失败")
		return
	} else if err == nil {
		Fail(c, "400", "所选部门存在下级部门，不允许删除")
		return
	}

	// 3. Prevent deleting departments referenced by users.
	const userCheckSQL = `
SELECT 1
FROM sys_user
WHERE dept_id = ANY($1)
LIMIT 1;
`
	if err := h.db.QueryRowContext(c.Request.Context(), userCheckSQL, pq.Array(body.IDs)).Scan(&dummy); err != nil && err != sql.ErrNoRows {
		Fail(c, "500", "校验用户关联失败")
		return
	} else if err == nil {
		Fail(c, "400", "所选部门存在用户关联，请解除关联后重试")
		return
	}

	// 4. Delete role-dept relations.
	const deleteRoleDeptSQL = `DELETE FROM sys_role_dept WHERE dept_id = ANY($1);`
	if _, err := h.db.ExecContext(c.Request.Context(), deleteRoleDeptSQL, pq.Array(body.IDs)); err != nil {
		Fail(c, "500", "删除角色和部门关联失败")
		return
	}

	// 5. Delete departments.
	const deleteSQL = `DELETE FROM sys_dept WHERE id = ANY($1);`
	if _, err := h.db.ExecContext(c.Request.Context(), deleteSQL, pq.Array(body.IDs)); err != nil {
		Fail(c, "500", "删除部门失败")
		return
	}

	OK(c, true)
}

// ExportDept handles GET /system/dept/export and streams a simple CSV file.
// 前端只需要一个可下载的文件，这里用 CSV 简化实现。
func (h *DeptHandler) ExportDept(c *gin.Context) {
	desc := strings.TrimSpace(c.Query("description"))
	statusStr := strings.TrimSpace(c.Query("status"))

	var status int64
	if statusStr != "" {
		if v, err := strconv.ParseInt(statusStr, 10, 64); err == nil && v > 0 {
			status = v
		}
	}

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1
	if desc != "" {
		where += " AND (d.name ILIKE $" + strconv.Itoa(argPos) + " OR COALESCE(d.description,'') ILIKE $" + strconv.Itoa(argPos) + ")"
		args = append(args, "%"+desc+"%")
		argPos++
	}
	if status != 0 {
		where += " AND d.status = $" + strconv.Itoa(argPos)
		args = append(args, status)
		argPos++
	}

	query := `
SELECT d.id,
       d.name,
       d.parent_id,
       d.status,
       d.sort,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
` + where + `
ORDER BY d.sort ASC, d.id ASC;
`

	rows, err := h.db.QueryContext(c.Request.Context(), query, args...)
	if err != nil {
		Fail(c, "500", "导出部门失败")
		return
	}
	defer rows.Close()

	c.Header("Content-Type", "text/csv; charset=utf-8")
	c.Header("Content-Disposition", "attachment; filename=\"dept_export.csv\"")

	writer := csv.NewWriter(c.Writer)
	_ = writer.Write([]string{"ID", "名称", "上级部门ID", "状态", "排序", "系统内置", "描述", "创建时间", "创建人", "修改时间", "修改人"})

	for rows.Next() {
		var (
			id, parentID          int64
			name, description     string
			statusVal             int16
			sortVal               int32
			isSystem              bool
			createTime            time.Time
			createUser, updateUser string
			updateTime            sql.NullTime
		)
		if err := rows.Scan(
			&id,
			&name,
			&parentID,
			&statusVal,
			&sortVal,
			&isSystem,
			&description,
			&createTime,
			&createUser,
			&updateTime,
			&updateUser,
		); err != nil {
			continue
		}

		ut := ""
		if updateTime.Valid {
			ut = updateTime.Time.Format(time.RFC3339)
		}

		record := []string{
			strconv.FormatInt(id, 10),
			name,
			strconv.FormatInt(parentID, 10),
			strconv.FormatInt(int64(statusVal), 10),
			strconv.FormatInt(int64(sortVal), 10),
			strconv.FormatBool(isSystem),
			description,
			createTime.Format(time.RFC3339),
			createUser,
			ut,
			updateUser,
		}
		_ = writer.Write(record)
	}
	writer.Flush()
}
