package http

import (
	"database/sql"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
)

// LabelValue represents a simple label/value pair for dictionaries.
type LabelValue struct {
	Label string `json:"label"`
	Value any    `json:"value"`
	Extra string `json:"extra,omitempty"`
}

// DeptTreeNode matches the TreeNodeData structure used by Arco Tree/TreeSelect.
type DeptTreeNode struct {
	Key      int64          `json:"key"`
	Title    string         `json:"title"`
	Disabled bool           `json:"disabled"`
	Children []DeptTreeNode `json:"children,omitempty"`
}

// MenuTreeNode is a simplified menu tree node for common menu trees.
type MenuTreeNode struct {
	Key      int64          `json:"key"`
	Title    string         `json:"title"`
	Disabled bool           `json:"disabled"`
	Children []MenuTreeNode `json:"children,omitempty"`
}

// CommonHandler exposes /common related endpoints.
type CommonHandler struct {
	db *sql.DB
}

func NewCommonHandler(db *sql.DB) *CommonHandler {
	return &CommonHandler{db: db}
}

// RegisterCommonRoutes registers /common endpoints.
func (h *CommonHandler) RegisterCommonRoutes(r *gin.Engine) {
	r.GET("/common/dict/option/site", h.ListSiteOptions)
	r.GET("/common/tree/menu", h.ListMenuTree)
	r.GET("/common/tree/dept", h.ListDeptTree)
	r.GET("/common/dict/user", h.ListUserDict)
	r.GET("/common/dict/role", h.ListRoleDict)
	r.GET("/common/dict/:code", h.ListDictByCode)
}

// ListSiteOptions 返回基础网站配置字典数据（用于前端初始化站点标题、图标等）。
// 数据来源于 sys_option 表的 SITE 类别，优先使用当前 value，其次 default_value。
func (h *CommonHandler) ListSiteOptions(c *gin.Context) {
	const query = `
SELECT code,
       COALESCE(value, default_value, '') AS value
FROM sys_option
WHERE category = 'SITE'
ORDER BY id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询网站配置失败")
		return
	}
	defer rows.Close()

	var list []LabelValue
	for rows.Next() {
		var (
			code  string
			value string
		)
		if err := rows.Scan(&code, &value); err != nil {
			Fail(c, "500", "解析网站配置失败")
			return
		}
		list = append(list, LabelValue{
			Label: code,
			Value: value,
		})
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询网站配置失败")
		return
	}
	OK(c, list)
}

// ListMenuTree handles GET /common/tree/menu and returns a menu tree
// compatible with the front-end TreeNodeData definition.
func (h *CommonHandler) ListMenuTree(c *gin.Context) {
	// 这里只返回目录/菜单（type IN (1,2)），按钮不参与菜单树。
	const query = `
SELECT id, title, parent_id, sort, status
FROM sys_menu
WHERE type IN (1, 2)
ORDER BY sort ASC, id ASC;
`

	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询菜单失败")
		return
	}
	defer rows.Close()

	type menuRow struct {
		id       int64
		title    string
		parentID int64
		sort     int32
		status   int16
	}

	var flat []menuRow
	for rows.Next() {
		var m menuRow
		if err := rows.Scan(&m.id, &m.title, &m.parentID, &m.sort, &m.status); err != nil {
			Fail(c, "500", "解析菜单数据失败")
			return
		}
		flat = append(flat, m)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询菜单失败")
		return
	}
	if len(flat) == 0 {
		OK(c, []MenuTreeNode{})
		return
	}

	nodeMap := make(map[int64]*MenuTreeNode, len(flat))
	for _, m := range flat {
		nodeMap[m.id] = &MenuTreeNode{
			Key:      m.id,
			Title:    m.title,
			Disabled: m.status != 1, // 禁用菜单在树中置为 disabled
		}
	}

	var roots []*MenuTreeNode
	for _, m := range flat {
		node := nodeMap[m.id]
		if m.parentID == 0 {
			roots = append(roots, node)
			continue
		}
		parent, ok := nodeMap[m.parentID]
		if !ok {
			roots = append(roots, node)
			continue
		}
		parent.Children = append(parent.Children, *node)
	}

	result := make([]MenuTreeNode, 0, len(roots))
	for _, n := range roots {
		result = append(result, *n)
	}
	OK(c, result)
}

// ListDeptTree handles GET /common/tree/dept and returns a department tree
// compatible with the front-end TreeNodeData definition.
func (h *CommonHandler) ListDeptTree(c *gin.Context) {
	// 当前实现简单返回所有部门树结构，前端已经有本地搜索能力。
	const query = `
SELECT id, name, parent_id, sort, status, is_system
FROM sys_dept
ORDER BY sort ASC, id ASC;
`

	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询部门失败")
		return
	}
	defer rows.Close()

	type deptRow struct {
		id       int64
		name     string
		parentID int64
		sort     int32
		status   int16
		isSystem bool
	}

	var flat []deptRow
	for rows.Next() {
		var d deptRow
		if err := rows.Scan(&d.id, &d.name, &d.parentID, &d.sort, &d.status, &d.isSystem); err != nil {
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
		OK(c, []DeptTreeNode{})
		return
	}

	// 构建 id 到节点的映射，用于组装树结构。
	nodeMap := make(map[int64]*DeptTreeNode, len(flat))
	for _, d := range flat {
		nodeMap[d.id] = &DeptTreeNode{
			Key:      d.id,
			Title:    d.name,
			Disabled: false, // 暂时不根据状态禁用，保持前端交互简单。
		}
	}

	var roots []*DeptTreeNode
	for _, d := range flat {
		node := nodeMap[d.id]
		if d.parentID == 0 {
			roots = append(roots, node)
			continue
		}
		parent, ok := nodeMap[d.parentID]
		if !ok {
			// 如果缺少上级节点，则将其视为根节点，避免数据丢失。
			roots = append(roots, node)
			continue
		}
		parent.Children = append(parent.Children, *node)
	}

	// 转换为值类型切片以进行 JSON 编码。
	result := make([]DeptTreeNode, 0, len(roots))
	for _, n := range roots {
		result = append(result, *n)
	}
	OK(c, result)
}

// ListUserDict handles GET /common/dict/user and returns a simple user dictionary.
// label: nickname (fallback to username)
// value: user ID (number)
// extra: username
func (h *CommonHandler) ListUserDict(c *gin.Context) {
	statusStr := strings.TrimSpace(c.Query("status"))

	baseSQL := `
SELECT id,
       COALESCE(nickname, username, ''),
       COALESCE(username, '')
FROM sys_user
WHERE status = 1
`
	args := []any{}

	// 如果显式传入 status，则按指定状态过滤。
	if statusStr != "" {
		if s, err := strconv.ParseInt(statusStr, 10, 64); err == nil && s > 0 {
			baseSQL = `
SELECT id,
       COALESCE(nickname, username, ''),
       COALESCE(username, '')
FROM sys_user
WHERE status = $1
`
			args = append(args, s)
		}
	}

	baseSQL += " ORDER BY id ASC;"

	rows, err := h.db.QueryContext(c.Request.Context(), baseSQL, args...)
	if err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	defer rows.Close()

	var list []LabelValue
	for rows.Next() {
		var (
			id       int64
			nickname string
			username string
		)
		if err := rows.Scan(&id, &nickname, &username); err != nil {
			Fail(c, "500", "解析用户数据失败")
			return
		}
		list = append(list, LabelValue{
			Label: nickname,
			Value: id,
			Extra: username,
		})
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询用户失败")
		return
	}
	OK(c, list)
}

// ListRoleDict handles GET /common/dict/role and returns a simple role dictionary.
// label: role name
// value: role ID (number)
// extra: role code
func (h *CommonHandler) ListRoleDict(c *gin.Context) {
	const query = `
SELECT id, name, code
FROM sys_role
ORDER BY sort ASC, id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query)
	if err != nil {
		Fail(c, "500", "查询角色失败")
		return
	}
	defer rows.Close()

	var list []LabelValue
	for rows.Next() {
		var (
			id   int64
			name string
			code string
		)
		if err := rows.Scan(&id, &name, &code); err != nil {
			Fail(c, "500", "解析角色数据失败")
			return
		}
		list = append(list, LabelValue{
			Label: name,
			Value: id,
			Extra: code,
		})
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询角色失败")
		return
	}
	OK(c, list)
}

// ListDictByCode handles GET /common/dict/{code} and returns dictionary items
// defined in sys_dict/sys_dict_item, compatible with the Java implementation.
// extra: color
func (h *CommonHandler) ListDictByCode(c *gin.Context) {
	code := strings.TrimSpace(c.Param("code"))
	if code == "" {
		OK(c, []LabelValue{})
		return
	}

	const query = `
SELECT t1.label,
       t1.value,
       COALESCE(t1.color, '') AS extra
FROM sys_dict_item AS t1
LEFT JOIN sys_dict AS t2 ON t1.dict_id = t2.id
WHERE t1.status = 1
  AND t2.code = $1
ORDER BY t1.sort ASC, t1.id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), query, code)
	if err != nil {
		Fail(c, "500", "查询字典失败")
		return
	}
	defer rows.Close()

	// 使用非 nil 切片，避免前端拿到 data=null 导致报错
	list := make([]LabelValue, 0)
	for rows.Next() {
		var item LabelValue
		if err := rows.Scan(&item.Label, &item.Value, &item.Extra); err != nil {
			Fail(c, "500", "解析字典数据失败")
			return
		}
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询字典失败")
		return
	}
	OK(c, list)
}
