package http

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/security"
)

// OptionResp matches OptionResp in admin/src/apis/system/type.ts.
type OptionResp struct {
	ID          int64  `json:"id"`
	Name        string `json:"name"`
	Code        string `json:"code"`
	Value       string `json:"value"`
	Description string `json:"description"`
}

// OptionQuery represents query params for listing options.
type OptionQuery struct {
	Code     []string
	Category string
}

// OptionHandler exposes /system/option endpoints used by /system/config* tabs.
type OptionHandler struct {
	db       *sql.DB
	tokenSvc *security.TokenService
}

func NewOptionHandler(db *sql.DB, tokenSvc *security.TokenService) *OptionHandler {
	return &OptionHandler{
		db:       db,
		tokenSvc: tokenSvc,
	}
}

// RegisterOptionRoutes registers /system/option endpoints.
func (h *OptionHandler) RegisterOptionRoutes(r *gin.Engine) {
	r.GET("/system/option", h.ListOption)
	r.PUT("/system/option", h.UpdateOption)
	r.PATCH("/system/option/value", h.ResetOptionValue)
}

// currentUserID parses token and returns userID; shared with other handlers.
func (h *OptionHandler) currentUserID(c *gin.Context) int64 {
	authz := c.GetHeader("Authorization")
	claims, err := h.tokenSvc.Parse(authz)
	if err != nil {
		Fail(c, "401", "未授权，请重新登录")
		return 0
	}
	return claims.UserID
}

// ListOption handles GET /system/option.
func (h *OptionHandler) ListOption(c *gin.Context) {
	var query OptionQuery
	if codes, ok := c.GetQueryArray("code"); ok && len(codes) > 0 {
		// support both repeated & comma-joined form
		for _, raw := range codes {
			parts := strings.Split(raw, ",")
			for _, p := range parts {
				p = strings.TrimSpace(p)
				if p != "" {
					query.Code = append(query.Code, p)
				}
			}
		}
	}
	query.Category = strings.TrimSpace(c.Query("category"))

	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if len(query.Code) > 0 {
		placeholders := make([]string, len(query.Code))
		for i, code := range query.Code {
			placeholders[i] = "$" + strconv.Itoa(argPos)
			args = append(args, code)
			argPos++
		}
		where += " AND code IN (" + strings.Join(placeholders, ",") + ")"
	}
	if query.Category != "" {
		where += fmt.Sprintf(" AND category = $%d", argPos)
		args = append(args, query.Category)
		argPos++
	}

	sqlText := `
SELECT id, name, code,
       COALESCE(value, default_value, '') AS value,
       COALESCE(description, '')
FROM sys_option
` + where + `
ORDER BY id ASC;
`
	rows, err := h.db.QueryContext(c.Request.Context(), sqlText, args...)
	if err != nil {
		Fail(c, "500", "查询系统配置失败")
		return
	}
	defer rows.Close()

	var list []OptionResp
	for rows.Next() {
		var item OptionResp
		if err := rows.Scan(&item.ID, &item.Name, &item.Code, &item.Value, &item.Description); err != nil {
			Fail(c, "500", "解析系统配置失败")
			return
		}
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询系统配置失败")
		return
	}
	OK(c, list)
}

// UpdateOption handles PUT /system/option (bulk update).
func (h *OptionHandler) UpdateOption(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}

	// 这里 Value 使用 interface{}，以兼容前端传递字符串、数字、布尔等多种类型，
	// 避免 Go 的 JSON 反序列化因类型不匹配而报错（例如 value 为 0 时不能直接解到 string）。
	var body []struct {
		ID    int64       `json:"id"`
		Code  string      `json:"code"`
		Value interface{} `json:"value"`
	}
	if err := c.ShouldBindJSON(&body); err != nil || len(body) == 0 {
		Fail(c, "400", "请求参数不正确")
		return
	}

	tx, err := h.db.BeginTx(c.Request.Context(), nil)
	if err != nil {
		Fail(c, "500", "保存系统配置失败")
		return
	}
	defer tx.Rollback()

	const stmt = `
UPDATE sys_option
   SET value = $1,
       update_user = $2,
       update_time = $3
 WHERE id = $4 AND code = $5;
`
	now := time.Now()
	for _, o := range body {
		// 将任意类型的 Value 转换为字符串存入数据库，保持与 Java 版本一致的行为。
		valStr := toOptionValueString(o.Value)
		if _, err := tx.ExecContext(c.Request.Context(), stmt, valStr, userID, now, o.ID, o.Code); err != nil {
			Fail(c, "500", "保存系统配置失败")
			return
		}
	}

	if err := tx.Commit(); err != nil {
		Fail(c, "500", "保存系统配置失败")
		return
	}
	OK(c, true)
}

// ResetOptionValue handles PATCH /system/option/value to reset to defaults.
func (h *OptionHandler) ResetOptionValue(c *gin.Context) {
	userID := h.currentUserID(c)
	if userID == 0 {
		return
	}
	_ = userID // reserved for audit usage

	var body struct {
		Code     []string `json:"code"`
		Category string   `json:"category"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		Fail(c, "400", "请求参数不正确")
		return
	}

	if len(body.Code) == 0 && strings.TrimSpace(body.Category) == "" {
		Fail(c, "400", "键列表或类别不能为空")
		return
	}

	where := ""
	args := []any{}
	argPos := 1
	if body.Category != "" {
		where = fmt.Sprintf("category = $%d", argPos)
		args = append(args, body.Category)
		argPos++
	} else if len(body.Code) > 0 {
		placeholders := make([]string, len(body.Code))
		for i, code := range body.Code {
			placeholders[i] = "$" + strconv.Itoa(argPos)
			args = append(args, code)
			argPos++
		}
		where = "code IN (" + strings.Join(placeholders, ",") + ")"
	}

	stmt := "UPDATE sys_option SET value = NULL"
	if where != "" {
		stmt += " WHERE " + where
	}
	if _, err := h.db.ExecContext(c.Request.Context(), stmt, args...); err != nil {
		Fail(c, "500", "恢复默认配置失败")
		return
	}
	OK(c, true)
}

// toOptionValueString 将任意 JSON 解析后的值转换为字符串，便于存入 sys_option.value。
// - 字符串：直接返回
// - 数字：转为不带小数的整数字符串（如 0, 1, 2）
// - 布尔：true/false
// - nil：空串
// - 其他复杂类型：序列化为 JSON 字符串
func toOptionValueString(v interface{}) string {
	if v == nil {
		return ""
	}
	switch t := v.(type) {
	case string:
		return t
	case float64:
		// JSON 数字默认解析为 float64，这里统一按整数方式输出（当前配置值场景足够）。
		return strconv.FormatInt(int64(t), 10)
	case bool:
		if t {
			return "true"
		}
		return "false"
	default:
		b, err := json.Marshal(t)
		if err != nil {
			return ""
		}
		return string(b)
	}
}
