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

// RoleResp matches RoleResp in admin/src/apis/system/type.ts.
type RoleResp struct {
	ID               int64  `json:"id"`
	Name             string `json:"name"`
	Code             string `json:"code"`
	Sort             int32  `json:"sort"`
	Description      string `json:"description"`
	DataScope        int32  `json:"dataScope"`
	IsSystem         bool   `json:"isSystem"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
	UpdateUserString string `json:"updateUserString"`
	UpdateTime       string `json:"updateTime"`
	Disabled         bool   `json:"disabled"`
}

// RoleDetailResp extends RoleResp with menu/dept information.
type RoleDetailResp struct {
	RoleResp
	MenuIDs          []int64 `json:"menuIds"`
	DeptIDs          []int64 `json:"deptIds"`
	MenuCheckStrict  bool    `json:"menuCheckStrictly"`
	DeptCheckStrict  bool    `json:"deptCheckStrictly"`
}

// RoleUserResp matches RoleUserResp at front-end.
type RoleUserResp struct {
	ID               int64    `json:"id"` // sys_user_role.id
	RoleID           int64    `json:"roleId"`
	UserID           int64    `json:"userId"`
	Username         string   `json:"username"`
	Nickname         string   `json:"nickname"`
	Gender           int16    `json:"gender"`
	Status           int16    `json:"status"`
	IsSystem         bool     `json:"isSystem"`
	Description      string   `json:"description"`
	DeptID           int64    `json:"deptId"`
	DeptName         string   `json:"deptName"`
	RoleIDs          []int64  `json:"roleIds"`
	RoleNames        []string `json:"roleNames"`
	Disabled         bool     `json:"disabled"`
}

type roleReq struct {
	Name             string   `json:"name"`
	Code             string   `json:"code"`
	Sort             int32    `json:"sort"`
	Description      string   `json:"description"`
	DataScope        int32    `json:"dataScope"`
	DeptIDs          []int64  `json:"deptIds"`
	DeptCheckStrict  bool     `json:"deptCheckStrictly"`
}

type rolePermissionReq struct {
	MenuIDs         []int64 `json:"menuIds"`
	MenuCheckStrict bool    `json:"menuCheckStrictly"`
}

// RoleHandler provides /system/role endpoints.
type RoleHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewRoleHandler(db *sql.DB, tokenSvc *security.TokenService) *RoleHandler {
	return &RoleHandler{db: db, tokenSvc: tokenSvc}
}

// RegisterRoleRoutes registers role management routes.
func (h *RoleHandler) RegisterRoleRoutes(r *gin.Engine) {
	r.GET("/system/role/list", h.ListRole)
	r.GET("/system/role/:id", h.GetRole)
	r.POST("/system/role", h.CreateRole)
	r.PUT("/system/role/:id", h.UpdateRole)
	r.DELETE("/system/role", h.DeleteRole)

	r.PUT("/system/role/:id/permission", h.UpdateRolePermission)
	r.GET("/system/role/:id/user", h.PageRoleUser)
	r.POST("/system/role/:id/user", h.AssignToUsers)
	r.DELETE("/system/role/user", h.UnassignFromUsers)
	r.GET("/system/role/:id/user/id", h.ListRoleUserIDs)
}

func (h *RoleHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListRole handles GET /system/role/list.
func (h *RoleHandler) ListRole(c *gin.Context) {
	const query = `
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       r.create_time,
       COALESCE(cu.nickname, ''),
       r.update_time,
       COALESCE(uu.nickname, '')
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
ORDER BY r.sort ASC, r.id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询角色失败")
		return
	}
	defer rows.Close()

	descFilter := strings.TrimSpace(c.Query("description"))

	var list []RoleResp
	for rows.Next() {
		var (
			item      RoleResp
			createAt  time.Time
			createBy  string
			updateAt  sql.NullTime
			updateBy  string
		)
		if err := rows.Scan(
			&item.ID,
			&item.Name,
			&item.Code,
			&item.Sort,
			&item.Description,
			&item.DataScope,
			&item.IsSystem,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		); err != nil {
			Fail(c, "500", "解析角色数据失败")
			return
		}
		if descFilter != "" && !strings.Contains(item.Name, descFilter) && !strings.Contains(item.Description, descFilter) {
			continue
		}
		item.CreateUserString = createBy
		item.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			item.UpdateUserString = updateBy
			item.UpdateTime = formatTime(updateAt.Time)
		}
		item.Disabled = item.IsSystem && item.Code == "admin"
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询角色失败")
		return
	}
	OK(c, list)
}

// GetRole handles GET /system/role/:id.
func (h *RoleHandler) GetRole(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       COALESCE(r.menu_check_strictly, TRUE),
       COALESCE(r.dept_check_strictly, TRUE),
       r.create_time,
       COALESCE(cu.nickname, ''),
       r.update_time,
       COALESCE(uu.nickname, '')
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = $1;
`

	var (
		base       RoleResp
		menuStrict bool
		deptStrict bool
		createAt   time.Time
		createBy   string
		updateAt   sql.NullTime
		updateBy   string
		isSystem   bool
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&base.ID,
			&base.Name,
			&base.Code,
			&base.Sort,
			&base.Description,
			&base.DataScope,
			&isSystem,
			&menuStrict,
			&deptStrict,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		)
	if err == sql.ErrNoRows {
		Fail(c, "404", "角色不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询角色失败")
		return
	}
	base.IsSystem = isSystem
	base.CreateUserString = createBy
	base.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		base.UpdateUserString = updateBy
		base.UpdateTime = formatTime(updateAt.Time)
	}
	base.Disabled = base.IsSystem && base.Code == "admin"

	// Load menuIds.
	menuRows, err := h.db.QueryContext(c.Request.Context(), `SELECT menu_id FROM sys_role_menu WHERE role_id = $1`, idVal)
	if err != nil {
		Fail(c, "500", "查询角色菜单失败")
		return
	}
	defer menuRows.Close()
	var menuIDs []int64
	for menuRows.Next() {
		var mid int64
		if err := menuRows.Scan(&mid); err != nil {
			Fail(c, "500", "查询角色菜单失败")
			return
		}
		menuIDs = append(menuIDs, mid)
	}
	if err := menuRows.Err(); err != nil {
		Fail(c, "500", "查询角色菜单失败")
		return
	}

	// Load deptIds.
	deptRows, err := h.db.QueryContext(c.Request.Context(), `SELECT dept_id FROM sys_role_dept WHERE role_id = $1`, idVal)
	if err != nil {
		Fail(c, "500", "查询角色部门失败")
		return
	}
	defer deptRows.Close()
	var deptIDs []int64
	for deptRows.Next() {
		var did int64
		if err := deptRows.Scan(&did); err != nil {
			Fail(c, "500", "查询角色部门失败")
			return
		}
		deptIDs = append(deptIDs, did)
	}
	if err := deptRows.Err(); err != nil {
		Fail(c, "500", "查询角色部门失败")
		return
	}

	resp := RoleDetailResp{
		RoleResp:        base,
		MenuIDs:         menuIDs,
		DeptIDs:         deptIDs,
		MenuCheckStrict: menuStrict,
		DeptCheckStrict: deptStrict,
	}
	OK(c, resp)
}

// CreateRole handles POST /system/role.
func (h *RoleHandler) CreateRole(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req roleReq
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
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.DataScope == 0 {
		req.DataScope = 4
	}

	now := time.Now()
	idVal := id.Next()

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "新增角色失败")
		return
	}
	defer tx.Rollback()

	const insertRole = `
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6,
        FALSE, TRUE, $7,
        $8, $9);
`
	if _, err := tx.ExecContext(
		c.Request.Context(),
		insertRole,
		idVal,
		req.Name,
		req.Code,
		req.DataScope,
		req.Description,
		req.Sort,
		req.DeptCheckStrict,
		userID,
		now,
	); err != nil {
		Fail(c, "500", "新增角色失败")
		return
	}

	if len(req.DeptIDs) > 0 {
		const insertDept = `INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`
		for _, did := range req.DeptIDs {
			if _, err := tx.ExecContext(c.Request.Context(), insertDept, idVal, did); err != nil {
				Fail(c, "500", "保存角色部门失败")
				return
			}
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "新增角色失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateRole handles PUT /system/role/:id.
func (h *RoleHandler) UpdateRole(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req roleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		Fail(c, "400", "名称不能为空")
		return
	}
	if req.Sort <= 0 {
		req.Sort = 999
	}
	if req.DataScope == 0 {
		req.DataScope = 4
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "修改角色失败")
		return
	}
	defer tx.Rollback()

	const updateRole = `
UPDATE sys_role
   SET name               = $1,
       description        = $2,
       sort               = $3,
       data_scope         = $4,
       dept_check_strictly= $5,
       update_user        = $6,
       update_time        = $7
 WHERE id                 = $8;
`
	if _, err := tx.ExecContext(
		c.Request.Context(),
		updateRole,
		req.Name,
		req.Description,
		req.Sort,
		req.DataScope,
		req.DeptCheckStrict,
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "修改角色失败")
		return
	}

	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role_dept WHERE role_id = $1`, idVal); err != nil {
		Fail(c, "500", "修改角色失败")
		return
	}
	if len(req.DeptIDs) > 0 {
		const insertDept = `INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`
		for _, did := range req.DeptIDs {
			if _, err := tx.ExecContext(c.Request.Context(), insertDept, idVal, did); err != nil {
				Fail(c, "500", "保存角色部门失败")
				return
			}
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "修改角色失败")
		return
	}
	OK(c, true)
}

// DeleteRole handles DELETE /system/role.
func (h *RoleHandler) DeleteRole(c *gin.Context) {
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
		Fail(c, "500", "删除角色失败")
		return
	}
	defer tx.Rollback()

	for _, idVal := range req.IDs {
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_user_role WHERE role_id = $1`, idVal); err != nil {
			Fail(c, "500", "删除角色失败")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role_menu WHERE role_id = $1`, idVal); err != nil {
			Fail(c, "500", "删除角色失败")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role_dept WHERE role_id = $1`, idVal); err != nil {
			Fail(c, "500", "删除角色失败")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除角色失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除角色失败")
		return
	}
	OK(c, true)
}

// UpdateRolePermission handles PUT /system/role/:id/permission.
func (h *RoleHandler) UpdateRolePermission(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req rolePermissionReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "保存角色权限失败")
		return
	}
	defer tx.Rollback()

	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_role_menu WHERE role_id = $1`, idVal); err != nil {
		Fail(c, "500", "保存角色权限失败")
		return
	}
	const insertMenu = `INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`
	for _, mid := range req.MenuIDs {
		if _, err := tx.ExecContext(c.Request.Context(), insertMenu, idVal, mid); err != nil {
			Fail(c, "500", "保存角色权限失败")
			return
		}
	}

	if _, err := tx.ExecContext(
		c.Request.Context(),
		`UPDATE sys_role SET menu_check_strictly = $1, update_user = $2, update_time = $3 WHERE id = $4`,
		req.MenuCheckStrict,
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "保存角色权限失败")
		return
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "保存角色权限失败")
		return
	}
	OK(c, true)
}

// PageRoleUser handles GET /system/role/:id/user (分页查询关联用户).
func (h *RoleHandler) PageRoleUser(c *gin.Context) {
	roleID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || roleID <= 0 {
		Fail(c, "400", "ID 参数不正确")
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
	descFilter := strings.TrimSpace(c.Query("description"))

	const query = `
SELECT ur.id,
       ur.role_id,
       u.id,
       u.username,
       u.nickname,
       u.gender,
       u.status,
       u.is_system,
       COALESCE(u.description, ''),
       u.dept_id,
       COALESCE(d.name, '')
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE ur.role_id = $1
ORDER BY ur.id DESC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query, roleID)
	if err != nil {
		Fail(c, "500", "查询关联用户失败")
		return
	}
	defer rows.Close()

	var all []RoleUserResp
	for rows.Next() {
		var item RoleUserResp
		if err := rows.Scan(
			&item.ID,
			&item.RoleID,
			&item.UserID,
			&item.Username,
			&item.Nickname,
			&item.Gender,
			&item.Status,
			&item.IsSystem,
			&item.Description,
			&item.DeptID,
			&item.DeptName,
		); err != nil {
			Fail(c, "500", "解析关联用户失败")
			return
		}
		if descFilter != "" &&
			!strings.Contains(item.Username, descFilter) &&
			!strings.Contains(item.Nickname, descFilter) &&
			!strings.Contains(item.Description, descFilter) {
			continue
		}
		all = append(all, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询关联用户失败")
		return
	}

	// Build roleIds/roleNames for each user.
	if len(all) > 0 {
		userIDs := make([]int64, 0, len(all))
		seen := make(map[int64]struct{})
		for _, item := range all {
			if _, ok := seen[item.UserID]; !ok {
				seen[item.UserID] = struct{}{}
				userIDs = append(userIDs, item.UserID)
			}
		}
		const roleQuery = `
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY($1::bigint[]);
`
		roleRows, err := h.db.QueryContext(c.Request.Context(), roleQuery, pqInt64Array(userIDs))
		if err != nil {
			Fail(c, "500", "查询用户角色失败")
			return
		}
		defer roleRows.Close()

		type info struct {
			ids   []int64
			names []string
		}
		roleMap := make(map[int64]*info)
		for roleRows.Next() {
			var uid, rid int64
			var name string
			if err := roleRows.Scan(&uid, &rid, &name); err != nil {
				Fail(c, "500", "查询用户角色失败")
				return
			}
			entry := roleMap[uid]
			if entry == nil {
				entry = &info{}
				roleMap[uid] = entry
			}
			entry.ids = append(entry.ids, rid)
			entry.names = append(entry.names, name)
		}
		if err := roleRows.Err(); err != nil {
			Fail(c, "500", "查询用户角色失败")
			return
		}

		for i := range all {
			if entry := roleMap[all[i].UserID]; entry != nil {
				all[i].RoleIDs = entry.ids
				all[i].RoleNames = entry.names
			}
			all[i].Disabled = all[i].IsSystem && all[i].RoleID == 1
		}
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

	OK(c, PageResult[RoleUserResp]{List: pageList, Total: total})
}

// AssignToUsers handles POST /system/role/:id/user.
func (h *RoleHandler) AssignToUsers(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	roleID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || roleID <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var userIDs []int64
	if err := c.ShouldBindJSON(&userIDs); err != nil || len(userIDs) == 0 {
		Fail(c, "400", "用户ID列表不能为空")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "分配用户失败")
		return
	}
	defer tx.Rollback()

	const insertUserRole = `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`
	for _, uid := range userIDs {
		if uid <= 0 {
			continue
		}
		if _, err := tx.ExecContext(c.Request.Context(), insertUserRole, id.Next(), uid, roleID); err != nil {
			Fail(c, "500", "分配用户失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "分配用户失败")
		return
	}
	OK(c, true)
}

// UnassignFromUsers handles DELETE /system/role/user.
// Body is an array of userRoleIds.
func (h *RoleHandler) UnassignFromUsers(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	_ = userID

	var ids []int64
	if err := c.ShouldBindJSON(&ids); err != nil || len(ids) == 0 {
		Fail(c, "400", "用户角色ID列表不能为空")
		return
	}

	if _, err := h.db.ExecContext(c.Request.Context(), `DELETE FROM sys_user_role WHERE id = ANY($1::bigint[])`, pqInt64Array(ids)); err != nil {
		Fail(c, "500", "取消分配失败")
		return
	}
	OK(c, true)
}

// ListRoleUserIDs handles GET /system/role/:id/user/id.
func (h *RoleHandler) ListRoleUserIDs(c *gin.Context) {
	roleID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || roleID <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	rows, err := h.db.QueryContext(c.Request.Context(), `SELECT user_id FROM sys_user_role WHERE role_id = $1`, roleID)
	if err != nil {
		Fail(c, "500", "查询关联用户失败")
		return
	}
	defer rows.Close()

	var ids []int64
	for rows.Next() {
		var uid int64
		if err := rows.Scan(&uid); err != nil {
			Fail(c, "500", "查询关联用户失败")
			return
		}
		ids = append(ids, uid)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询关联用户失败")
		return
	}
	OK(c, ids)
}

