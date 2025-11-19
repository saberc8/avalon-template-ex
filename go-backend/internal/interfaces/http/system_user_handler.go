package http

import (
	"database/sql"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/id"
	"voc-go-backend/internal/infrastructure/security"
)

// UserResp matches UserResp in admin/src/apis/system/type.ts.
type UserResp struct {
	ID               int64    `json:"id"`
	Username         string   `json:"username"`
	Nickname         string   `json:"nickname"`
	Avatar           string   `json:"avatar"`
	Gender           int16    `json:"gender"`
	Email            string   `json:"email"`
	Phone            string   `json:"phone"`
	Description      string   `json:"description"`
	Status           int16    `json:"status"`
	IsSystem         bool     `json:"isSystem"`
	CreateUserString string   `json:"createUserString"`
	CreateTime       string   `json:"createTime"`
	UpdateUserString string   `json:"updateUserString"`
	UpdateTime       string   `json:"updateTime"`
	DeptID           int64    `json:"deptId"`
	DeptName         string   `json:"deptName"`
	RoleIDs          []int64  `json:"roleIds"`
	RoleNames        []string `json:"roleNames"`
	Disabled         bool     `json:"disabled"`
}

// UserDetailResp extends UserResp with pwdResetTime.
type UserDetailResp struct {
	UserResp
	PwdResetTime string `json:"pwdResetTime,omitempty"`
}

type userReq struct {
	Username    string  `json:"username"`
	Nickname    string  `json:"nickname"`
	Password    string  `json:"password"` // only for create
	Gender      int16   `json:"gender"`
	Email       string  `json:"email"`
	Phone       string  `json:"phone"`
	Avatar      string  `json:"avatar"`
	Description string  `json:"description"`
	Status      int16   `json:"status"`
	DeptID      int64   `json:"deptId"`
	RoleIDs     []int64 `json:"roleIds"`
}

type userPasswordResetReq struct {
	NewPassword string `json:"newPassword"`
}

type userRoleUpdateReq struct {
	RoleIDs []int64 `json:"roleIds"`
}

// UserImportParseResp is a simplified version of UserImportParseResp in Java.
type UserImportParseResp struct {
	ImportKey          string `json:"importKey"`
	TotalRows          int    `json:"totalRows"`
	ValidRows          int    `json:"validRows"`
	DuplicateUserRows  int    `json:"duplicateUserRows"`
	DuplicateEmailRows int    `json:"duplicateEmailRows"`
	DuplicatePhoneRows int    `json:"duplicatePhoneRows"`
}

// UserImportResultResp matches UserImportResp in Java (used as import result).
type UserImportResultResp struct {
	TotalRows  int `json:"totalRows"`
	InsertRows int `json:"insertRows"`
	UpdateRows int `json:"updateRows"`
}

// SystemUserHandler provides /system/user endpoints.
type SystemUserHandler struct {
	db           *sql.DB
	tokenSvc     *security.TokenService
	rsaDecryptor *security.RSADecryptor
	hasher       security.PasswordHasher
}

func NewSystemUserHandler(db *sql.DB, tokenSvc *security.TokenService, rsa *security.RSADecryptor, hasher security.PasswordHasher) *SystemUserHandler {
	return &SystemUserHandler{
		db:           db,
		tokenSvc:     tokenSvc,
		rsaDecryptor: rsa,
		hasher:       hasher,
	}
}

// RegisterSystemUserRoutes registers /system/user related routes.
func (h *SystemUserHandler) RegisterSystemUserRoutes(r *gin.Engine) {
	r.GET("/system/user", h.ListUserPage)
	r.GET("/system/user/list", h.ListAllUser)
	r.GET("/system/user/:id", h.GetUserDetail)
	r.POST("/system/user", h.CreateUser)
	r.PUT("/system/user/:id", h.UpdateUser)
	r.DELETE("/system/user", h.DeleteUser)
	r.PATCH("/system/user/:id/password", h.ResetPassword)
	r.PATCH("/system/user/:id/role", h.UpdateUserRole)

	// 导出与导入相关接口（简化实现）
	r.GET("/system/user/export", h.ExportUser)
	r.GET("/system/user/import/template", h.DownloadImportTemplate)
	r.POST("/system/user/import/parse", h.ParseImportUser)
	r.POST("/system/user/import", h.ImportUser)
}

func (h *SystemUserHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListUserPage handles GET /system/user (分页查询用户).
func (h *SystemUserHandler) ListUserPage(c *gin.Context) {
	page, _ := strconv.Atoi(c.Query("page"))
	size, _ := strconv.Atoi(c.Query("size"))
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}

	desc := strings.TrimSpace(c.Query("description"))
	statusStr := strings.TrimSpace(c.Query("status"))
	deptStr := strings.TrimSpace(c.Query("deptId"))

	var (
		statusFilter int64
		deptID       int64
	)
	if statusStr != "" {
		statusFilter, _ = strconv.ParseInt(statusStr, 10, 64)
	}
	if deptStr != "" {
		deptID, _ = strconv.ParseInt(deptStr, 10, 64)
	}

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1
	if desc != "" {
		where += fmt.Sprintf(" AND (u.username ILIKE $%d OR u.nickname ILIKE $%d OR COALESCE(u.description,'') ILIKE $%d)", argPos, argPos, argPos)
		args = append(args, "%"+desc+"%")
		argPos++
	}
	if statusFilter != 0 {
		where += fmt.Sprintf(" AND u.status = $%d", argPos)
		args = append(args, statusFilter)
		argPos++
	}
	if deptID != 0 {
		where += fmt.Sprintf(" AND u.dept_id = $%d", argPos)
		args = append(args, deptID)
		argPos++
	}

	countSQL := "SELECT COUNT(*) FROM sys_user AS u " + where
	var total int64
	if err := h.db.QueryRowContext(c.Request.Context(), countSQL, args...).Scan(&total); err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	if total == 0 {
		OK(c, PageResult[UserResp]{List: []UserResp{}, Total: 0})
		return
	}

	offset := int64((page - 1) * size)
	limitPos := argPos
	offsetPos := argPos + 1
	argsWithPage := append(args, int64(size), offset)

	query := fmt.Sprintf(`
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
%s
ORDER BY u.id DESC
LIMIT $%d OFFSET $%d;
`, where, limitPos, offsetPos)

	rows, err := h.db.QueryContext(c.Request.Context(), query, argsWithPage...)
	if err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	defer rows.Close()

	var users []UserResp
	for rows.Next() {
		var (
			u        UserResp
			createAt time.Time
			createBy string
			updateAt sql.NullTime
			updateBy string
		)
		if err := rows.Scan(
			&u.ID,
			&u.Username,
			&u.Nickname,
			&u.Avatar,
			&u.Gender,
			&u.Email,
			&u.Phone,
			&u.Description,
			&u.Status,
			&u.IsSystem,
			&u.DeptID,
			&u.DeptName,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		); err != nil {
			Fail(c, "500", "解析用户数据失败")
			return
		}
		u.CreateUserString = createBy
		u.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			u.UpdateUserString = updateBy
			u.UpdateTime = formatTime(updateAt.Time)
		}
		u.Disabled = u.IsSystem
		users = append(users, u)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}

	// 填充角色信息
	h.fillUserRoles(c, users)

	OK(c, PageResult[UserResp]{List: users, Total: total})
}

// ListAllUser handles GET /system/user/list.
func (h *SystemUserHandler) ListAllUser(c *gin.Context) {
	idStrs := c.QueryArray("userIds")
	var ids []int64
	for _, s := range idStrs {
		if v, err := strconv.ParseInt(s, 10, 64); err == nil && v > 0 {
			ids = append(ids, v)
		}
	}

	var rows *sql.Rows
	var err error
	if len(ids) > 0 {
		rows, err = h.db.QueryContext(
			c.Request.Context(),
			`SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = ANY($1::bigint[])`,
			pqInt64Array(ids),
		)
	} else {
		rows, err = h.db.QueryContext(
			c.Request.Context(),
			`SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
ORDER BY u.id DESC`,
		)
	}
	if err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	defer rows.Close()

	var users []UserResp
	for rows.Next() {
		var (
			u        UserResp
			createAt time.Time
			createBy string
			updateAt sql.NullTime
			updateBy string
		)
		if err := rows.Scan(
			&u.ID,
			&u.Username,
			&u.Nickname,
			&u.Avatar,
			&u.Gender,
			&u.Email,
			&u.Phone,
			&u.Description,
			&u.Status,
			&u.IsSystem,
			&u.DeptID,
			&u.DeptName,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		); err != nil {
			Fail(c, "500", "解析用户数据失败")
			return
		}
		u.CreateUserString = createBy
		u.CreateTime = formatTime(createAt)
		if updateAt.Valid {
			u.UpdateUserString = updateBy
			u.UpdateTime = formatTime(updateAt.Time)
		}
		u.Disabled = u.IsSystem
		users = append(users, u)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}

	h.fillUserRoles(c, users)

	OK(c, users)
}

// fillUserRoles loads roleIds and roleNames for the given users.
func (h *SystemUserHandler) fillUserRoles(c *gin.Context, users []UserResp) {
	if len(users) == 0 {
		return
	}
	userIDs := make([]int64, 0, len(users))
	seen := make(map[int64]struct{})
	for _, u := range users {
		if _, ok := seen[u.ID]; !ok {
			seen[u.ID] = struct{}{}
			userIDs = append(userIDs, u.ID)
		}
	}
	const query = `
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY($1::bigint[]);
`
	rows, err := h.db.QueryContext(c.Request.Context(), query, pqInt64Array(userIDs))
	if err != nil {
		return
	}
	defer rows.Close()

	type info struct {
		ids   []int64
		names []string
	}
	roleMap := make(map[int64]*info)
	for rows.Next() {
		var uid, rid int64
		var name string
		if err := rows.Scan(&uid, &rid, &name); err != nil {
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
	for i := range users {
		if entry := roleMap[users[i].ID]; entry != nil {
			users[i].RoleIDs = entry.ids
			users[i].RoleNames = entry.names
		}
	}
}

// GetUserDetail handles GET /system/user/:id.
func (h *SystemUserHandler) GetUserDetail(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.pwd_reset_time,
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = $1;
`
	var (
		u        UserDetailResp
		pwdReset sql.NullTime
		createAt time.Time
		createBy string
		updateAt sql.NullTime
		updateBy string
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).
		Scan(
			&u.ID,
			&u.Username,
			&u.Nickname,
			&u.Avatar,
			&u.Gender,
			&u.Email,
			&u.Phone,
			&u.Description,
			&u.Status,
			&u.IsSystem,
			&u.DeptID,
			&u.DeptName,
			&pwdReset,
			&createAt,
			&createBy,
			&updateAt,
			&updateBy,
		)
	if err == sql.ErrNoRows {
		Fail(c, "404", "用户不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	u.CreateUserString = createBy
	u.CreateTime = formatTime(createAt)
	if updateAt.Valid {
		u.UpdateUserString = updateBy
		u.UpdateTime = formatTime(updateAt.Time)
	}
	u.Disabled = u.IsSystem
	if pwdReset.Valid {
		u.PwdResetTime = formatTime(pwdReset.Time)
	}

	// 角色信息
	h.fillUserRoles(c, []UserResp{u.UserResp})
	if len(u.UserResp.RoleIDs) > 0 || len(u.UserResp.RoleNames) > 0 {
		u.RoleIDs = u.UserResp.RoleIDs
		u.RoleNames = u.UserResp.RoleNames
	}

	OK(c, u)
}

// CreateUser handles POST /system/user.
func (h *SystemUserHandler) CreateUser(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	var req userReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Username = strings.TrimSpace(req.Username)
	req.Nickname = strings.TrimSpace(req.Nickname)
	if req.Username == "" || req.Nickname == "" {
		Fail(c, "400", "用户名和昵称不能为空")
		return
	}
	if req.DeptID == 0 {
		Fail(c, "400", "所属部门不能为空")
		return
	}
	if req.Status == 0 {
		req.Status = 1
	}
	if strings.TrimSpace(req.Password) == "" {
		Fail(c, "400", "密码不能为空")
		return
	}

	rawPwd, err := h.rsaDecryptor.DecryptBase64(req.Password)
	if err != nil {
		Fail(c, "400", "密码解密失败")
		return
	}
	if len(rawPwd) < 8 || len(rawPwd) > 32 {
		Fail(c, "400", "密码长度为 8-32 个字符，至少包含字母和数字")
		return
	}
	var hasLetter, hasDigit bool
	for _, ch := range rawPwd {
		switch {
		case ch >= '0' && ch <= '9':
			hasDigit = true
		case (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z'):
			hasLetter = true
		}
	}
	if !hasLetter || !hasDigit {
		Fail(c, "400", "密码长度为 8-32 个字符，至少包含字母和数字")
		return
	}

	encodedPwd, err := h.hasher.Hash(rawPwd)
	if err != nil {
		Fail(c, "500", "密码加密失败")
		return
	}

	now := time.Now()
	idVal := id.Next()

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "新增用户失败")
		return
	}
	defer tx.Rollback()

	const insertUser = `
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
        $9, $10, FALSE, $11, $12,
        $13, $14);
`
	if _, err := tx.ExecContext(
		c.Request.Context(),
		insertUser,
		idVal,
		req.Username,
		req.Nickname,
		encodedPwd,
		req.Gender,
		req.Email,
		req.Phone,
		req.Avatar,
		req.Description,
		req.Status,
		now,
		req.DeptID,
		userID,
		now,
	); err != nil {
		Fail(c, "500", "新增用户失败")
		return
	}

	if len(req.RoleIDs) > 0 {
		const insertUserRole = `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`
		for _, rid := range req.RoleIDs {
			if _, err := tx.ExecContext(c.Request.Context(), insertUserRole, id.Next(), idVal, rid); err != nil {
				Fail(c, "500", "保存用户角色失败")
				return
			}
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "新增用户失败")
		return
	}
	OK(c, gin.H{"id": idVal})
}

// UpdateUser handles PUT /system/user/:id.
func (h *SystemUserHandler) UpdateUser(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req userReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	req.Username = strings.TrimSpace(req.Username)
	req.Nickname = strings.TrimSpace(req.Nickname)
	if req.Username == "" || req.Nickname == "" {
		Fail(c, "400", "用户名和昵称不能为空")
		return
	}
	if req.DeptID == 0 {
		Fail(c, "400", "所属部门不能为空")
		return
	}
	if req.Status == 0 {
		req.Status = 1
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "修改用户失败")
		return
	}
	defer tx.Rollback()

	const updateUser = `
UPDATE sys_user
   SET username    = $1,
       nickname    = $2,
       gender      = $3,
       email       = $4,
       phone       = $5,
       avatar      = $6,
       description = $7,
       status      = $8,
       dept_id     = $9,
       update_user = $10,
       update_time = $11
 WHERE id          = $12;
`
	if _, err := tx.ExecContext(
		c.Request.Context(),
		updateUser,
		req.Username,
		req.Nickname,
		req.Gender,
		req.Email,
		req.Phone,
		req.Avatar,
		req.Description,
		req.Status,
		req.DeptID,
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "修改用户失败")
		return
	}

	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_user_role WHERE user_id = $1`, idVal); err != nil {
		Fail(c, "500", "修改用户失败")
		return
	}
	if len(req.RoleIDs) > 0 {
		const insertUserRole = `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`
		for _, rid := range req.RoleIDs {
			if _, err := tx.ExecContext(c.Request.Context(), insertUserRole, id.Next(), idVal, rid); err != nil {
				Fail(c, "500", "保存用户角色失败")
				return
			}
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "修改用户失败")
		return
	}
	OK(c, true)
}

// DeleteUser handles DELETE /system/user.
func (h *SystemUserHandler) DeleteUser(c *gin.Context) {
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
		Fail(c, "500", "删除用户失败")
		return
	}
	defer tx.Rollback()

	for _, idVal := range req.IDs {
		// 不允许删除系统内置用户
		var isSystem bool
		if err := tx.QueryRowContext(c.Request.Context(), `SELECT is_system FROM sys_user WHERE id = $1`, idVal).Scan(&isSystem); err != nil {
			if err == sql.ErrNoRows {
				continue
			}
			Fail(c, "500", "删除用户失败")
			return
		}
		if isSystem {
			continue
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_user_role WHERE user_id = $1`, idVal); err != nil {
			Fail(c, "500", "删除用户失败")
			return
		}
		if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_user WHERE id = $1`, idVal); err != nil {
			Fail(c, "500", "删除用户失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "删除用户失败")
		return
	}
	OK(c, true)
}

// ResetPassword handles PATCH /system/user/:id/password.
func (h *SystemUserHandler) ResetPassword(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req userPasswordResetReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	if strings.TrimSpace(req.NewPassword) == "" {
		Fail(c, "400", "密码不能为空")
		return
	}

	rawPwd, err := h.rsaDecryptor.DecryptBase64(req.NewPassword)
	if err != nil {
		Fail(c, "400", "密码解密失败")
		return
	}
	if len(rawPwd) < 8 || len(rawPwd) > 32 {
		Fail(c, "400", "密码长度为 8-32 个字符，至少包含字母和数字")
		return
	}
	var hasLetter, hasDigit bool
	for _, ch := range rawPwd {
		switch {
		case ch >= '0' && ch <= '9':
			hasDigit = true
		case (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z'):
			hasLetter = true
		}
	}
	if !hasLetter || !hasDigit {
		Fail(c, "400", "密码长度为 8-32 个字符，至少包含字母和数字")
		return
	}

	encodedPwd, err := h.hasher.Hash(rawPwd)
	if err != nil {
		Fail(c, "500", "密码加密失败")
		return
	}

	if _, err := h.db.ExecContext(
		c.Request.Context(),
		`UPDATE sys_user SET password = $1, pwd_reset_time = $2, update_user = $3, update_time = $4 WHERE id = $5`,
		encodedPwd,
		time.Now(),
		userID,
		time.Now(),
		idVal,
	); err != nil {
		Fail(c, "500", "重置密码失败")
		return
	}
	OK(c, true)
}

// UpdateUserRole handles PATCH /system/user/:id/role.
func (h *SystemUserHandler) UpdateUserRole(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	_ = userID

	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	var req userRoleUpdateReq
	if err := c.ShouldBindJSON(&req); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "分配角色失败")
		return
	}
	defer tx.Rollback()

	if _, err := tx.ExecContext(c.Request.Context(), `DELETE FROM sys_user_role WHERE user_id = $1`, idVal); err != nil {
		Fail(c, "500", "分配角色失败")
		return
	}
	if len(req.RoleIDs) > 0 {
		const insertUserRole = `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`
		for _, rid := range req.RoleIDs {
			if _, err := tx.ExecContext(c.Request.Context(), insertUserRole, id.Next(), idVal, rid); err != nil {
				Fail(c, "500", "分配角色失败")
				return
			}
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "分配角色失败")
		return
	}
	OK(c, true)
}

// ExportUser handles GET /system/user/export.
// It returns a very simple CSV file with basic user information.
func (h *SystemUserHandler) ExportUser(c *gin.Context) {
	rows, err := h.db.QueryContext(
		c.Request.Context(),
		`SELECT username, nickname, gender, COALESCE(email,''), COALESCE(phone,'') FROM sys_user ORDER BY id`,
	)
	if err != nil {
		Fail(c, "500", "导出用户失败")
		return
	}
	defer rows.Close()

	var b strings.Builder
	b.WriteString("username,nickname,gender,email,phone\n")
	for rows.Next() {
		var username, nickname, email, phone string
		var gender int16
		if err := rows.Scan(&username, &nickname, &gender, &email, &phone); err != nil {
			Fail(c, "500", "导出用户失败")
			return
		}
		line := fmt.Sprintf("%s,%s,%d,%s,%s\n", username, nickname, gender, email, phone)
		b.WriteString(line)
	}

	c.Header("Content-Type", "text/csv; charset=utf-8")
	c.Header("Content-Disposition", "attachment; filename=\"users.csv\"")
	c.String(http.StatusOK, b.String())
}

// DownloadImportTemplate handles GET /system/user/import/template.
func (h *SystemUserHandler) DownloadImportTemplate(c *gin.Context) {
	content := "username,nickname,gender,email,phone\n"
	c.Header("Content-Type", "text/csv; charset=utf-8")
	c.Header("Content-Disposition", "attachment; filename=\"user_import_template.csv\"")
	c.String(http.StatusOK, content)
}

// ParseImportUser handles POST /system/user/import/parse.
// 当前实现不真正解析 Excel，只返回一个空的解析结果，保证前端流程可用。
func (h *SystemUserHandler) ParseImportUser(c *gin.Context) {
	// 接收上传文件，但不做实际解析
	if _, err := c.FormFile("file"); err != nil {
		Fail(c, "400", "文件不能为空")
		return
	}
	resp := UserImportParseResp{
		ImportKey:          strconv.FormatInt(time.Now().UnixNano(), 10),
		TotalRows:          0,
		ValidRows:          0,
		DuplicateUserRows:  0,
		DuplicateEmailRows: 0,
		DuplicatePhoneRows: 0,
	}
	OK(c, resp)
}

// ImportUser handles POST /system/user/import.
// 当前实现为占位实现，不执行真正的导入逻辑。
func (h *SystemUserHandler) ImportUser(c *gin.Context) {
	var body map[string]any
	if err := c.ShouldBindJSON(&body); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}
	_ = body // 预留给未来扩展

	resp := UserImportResultResp{
		TotalRows:  0,
		InsertRows: 0,
		UpdateRows: 0,
	}
	OK(c, resp)
}

