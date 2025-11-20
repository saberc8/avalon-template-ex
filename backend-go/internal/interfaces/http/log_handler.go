package http

import (
	"database/sql"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
)

// LogResp 与前端 LogResp 类型对齐。
type LogResp struct {
	ID               int64  `json:"id"`
	Description      string `json:"description"`
	Module           string `json:"module"`
	TimeTaken        int64  `json:"timeTaken"`
	IP               string `json:"ip"`
	Address          string `json:"address"`
	Browser          string `json:"browser"`
	OS               string `json:"os"`
	Status           int16  `json:"status"`
	ErrorMsg         string `json:"errorMsg"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
}

// LogDetailResp 与前端 LogDetailResp 类型对齐。
type LogDetailResp struct {
	ID               int64  `json:"id"`
	TraceID          string `json:"traceId"`
	Description      string `json:"description"`
	Module           string `json:"module"`
	RequestURL       string `json:"requestUrl"`
	RequestMethod    string `json:"requestMethod"`
	RequestHeaders   string `json:"requestHeaders"`
	RequestBody      string `json:"requestBody"`
	StatusCode       int32  `json:"statusCode"`
	ResponseHeaders  string `json:"responseHeaders"`
	ResponseBody     string `json:"responseBody"`
	TimeTaken        int64  `json:"timeTaken"`
	IP               string `json:"ip"`
	Address          string `json:"address"`
	Browser          string `json:"browser"`
	OS               string `json:"os"`
	Status           int16  `json:"status"`
	ErrorMsg         string `json:"errorMsg"`
	CreateUserString string `json:"createUserString"`
	CreateTime       string `json:"createTime"`
}

// LogHandler 提供 /system/log 相关接口。
type LogHandler struct {
	db *sql.DB
}

// NewLogHandler 创建日志 handler。
func NewLogHandler(db *sql.DB) *LogHandler {
	return &LogHandler{db: db}
}

// RegisterLogRoutes 注册系统日志路由。
func (h *LogHandler) RegisterLogRoutes(r *gin.Engine) {
	r.GET("/system/log", h.PageLog)
	r.GET("/system/log/:id", h.GetLog)
	r.GET("/system/log/export/login", h.ExportLoginLog)
	r.GET("/system/log/export/operation", h.ExportOperationLog)
}

// PageLog 处理 GET /system/log，返回分页日志列表。
func (h *LogHandler) PageLog(c *gin.Context) {
	page, _ := strconv.Atoi(c.Query("page"))
	size, _ := strconv.Atoi(c.Query("size"))
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}

	description := strings.TrimSpace(c.Query("description"))
	module := strings.TrimSpace(c.Query("module"))
	ip := strings.TrimSpace(c.Query("ip"))
	createUser := strings.TrimSpace(c.Query("createUserString"))
	statusStr := strings.TrimSpace(c.Query("status"))

	var statusFilter int64
	if statusStr != "" {
		statusFilter, _ = strconv.ParseInt(statusStr, 10, 64)
	}

	var (
		startTime *time.Time
		endTime   *time.Time
	)
	timeRange := c.QueryArray("createTime")
	if len(timeRange) == 2 {
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[0], time.Local); err == nil {
			startTime = &t
		}
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[1], time.Local); err == nil {
			endTime = &t
		}
	}

	baseFrom := `
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
`
	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if description != "" {
		where += fmt.Sprintf(" AND (t1.description ILIKE $%d OR t1.module ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+description+"%")
		argPos++
	}
	if module != "" {
		where += fmt.Sprintf(" AND t1.module = $%d", argPos)
		args = append(args, module)
		argPos++
	}
	if ip != "" {
		where += fmt.Sprintf(" AND (t1.ip ILIKE $%d OR t1.address ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+ip+"%")
		argPos++
	}
	if createUser != "" {
		where += fmt.Sprintf(" AND (t2.username ILIKE $%d OR t2.nickname ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+createUser+"%")
		argPos++
	}
	if statusFilter != 0 {
		where += fmt.Sprintf(" AND t1.status = $%d", argPos)
		args = append(args, statusFilter)
		argPos++
	}
	if startTime != nil && endTime != nil {
		where += fmt.Sprintf(" AND t1.create_time BETWEEN $%d AND $%d", argPos, argPos+1)
		args = append(args, *startTime, *endTime)
		argPos += 2
	}

	countSQL := "SELECT COUNT(*) " + baseFrom + where
	var total int64
	if err := h.db.QueryRowContext(c.Request.Context(), countSQL, args...).Scan(&total); err != nil {
		Fail(c, "500", "查询日志失败")
		return
	}
	if total == 0 {
		OK(c, PageResult[LogResp]{List: []LogResp{}, Total: 0})
		return
	}

	offset := int64((page - 1) * size)
	argsWithPage := append(args, int64(size), offset)
	limitPos := argPos
	offsetPos := argPos + 1

	query := fmt.Sprintf(`
SELECT t1.id,
       t1.description,
       t1.module,
       COALESCE(t1.time_taken, 0),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.status, 1),
       COALESCE(t1.error_msg, ''),
       t1.create_time,
       COALESCE(t2.nickname, '')
%s
%s
ORDER BY t1.create_time DESC, t1.id DESC
LIMIT $%d OFFSET $%d;
`, baseFrom, where, limitPos, offsetPos)

	rows, err := h.db.QueryContext(c.Request.Context(), query, argsWithPage...)
	if err != nil {
		Fail(c, "500", "查询日志失败")
		return
	}
	defer rows.Close()

	var list []LogResp
	for rows.Next() {
		var (
			item     LogResp
			createAt time.Time
			createBy string
		)
		if err := rows.Scan(
			&item.ID,
			&item.Description,
			&item.Module,
			&item.TimeTaken,
			&item.IP,
			&item.Address,
			&item.Browser,
			&item.OS,
			&item.Status,
			&item.ErrorMsg,
			&createAt,
			&createBy,
		); err != nil {
			Fail(c, "500", "解析日志数据失败")
			return
		}
		item.CreateUserString = createBy
		item.CreateTime = formatTime(createAt)
		list = append(list, item)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "查询日志失败")
		return
	}

	OK(c, PageResult[LogResp]{List: list, Total: total})
}

// GetLog 处理 GET /system/log/:id，返回日志详情。
func (h *LogHandler) GetLog(c *gin.Context) {
	idVal, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil || idVal <= 0 {
		Fail(c, "400", "ID 参数不正确")
		return
	}

	const query = `
SELECT t1.id,
       COALESCE(t1.trace_id, ''),
       t1.description,
       t1.module,
       t1.request_url,
       t1.request_method,
       COALESCE(t1.request_headers, ''),
       COALESCE(t1.request_body, ''),
       t1.status_code,
       COALESCE(t1.response_headers, ''),
       COALESCE(t1.response_body, ''),
       COALESCE(t1.time_taken, 0),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.status, 1),
       COALESCE(t1.error_msg, ''),
       t1.create_time,
       COALESCE(t2.nickname, '')
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
WHERE t1.id = $1;
`

	var (
		resp     LogDetailResp
		createAt time.Time
		createBy string
	)
	err = h.db.QueryRowContext(c.Request.Context(), query, idVal).Scan(
		&resp.ID,
		&resp.TraceID,
		&resp.Description,
		&resp.Module,
		&resp.RequestURL,
		&resp.RequestMethod,
		&resp.RequestHeaders,
		&resp.RequestBody,
		&resp.StatusCode,
		&resp.ResponseHeaders,
		&resp.ResponseBody,
		&resp.TimeTaken,
		&resp.IP,
		&resp.Address,
		&resp.Browser,
		&resp.OS,
		&resp.Status,
		&resp.ErrorMsg,
		&createAt,
		&createBy,
	)
	if err == sql.ErrNoRows {
		Fail(c, "404", "日志不存在")
		return
	}
	if err != nil {
		Fail(c, "500", "查询日志失败")
		return
	}
	resp.CreateUserString = createBy
	resp.CreateTime = formatTime(createAt)

	OK(c, resp)
}

// ExportLoginLog 处理 GET /system/log/export/login，导出登录日志 CSV。
func (h *LogHandler) ExportLoginLog(c *gin.Context) {
	h.exportLogCSV(c, true)
}

// ExportOperationLog 处理 GET /system/log/export/operation，导出操作日志 CSV。
func (h *LogHandler) ExportOperationLog(c *gin.Context) {
	h.exportLogCSV(c, false)
}

// exportLogCSV 按条件导出登录/操作日志为 CSV。
func (h *LogHandler) exportLogCSV(c *gin.Context, isLogin bool) {
	description := strings.TrimSpace(c.Query("description"))
	module := strings.TrimSpace(c.Query("module"))
	ip := strings.TrimSpace(c.Query("ip"))
	createUser := strings.TrimSpace(c.Query("createUserString"))
	statusStr := strings.TrimSpace(c.Query("status"))

	var statusFilter int64
	if statusStr != "" {
		statusFilter, _ = strconv.ParseInt(statusStr, 10, 64)
	}

	var (
		startTime *time.Time
		endTime   *time.Time
	)
	timeRange := c.QueryArray("createTime")
	if len(timeRange) == 2 {
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[0], time.Local); err == nil {
			startTime = &t
		}
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[1], time.Local); err == nil {
			endTime = &t
		}
	}

	baseFrom := `
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
`
	where := "WHERE 1=1"
	args := []any{}
	argPos := 1

	if description != "" {
		where += fmt.Sprintf(" AND (t1.description ILIKE $%d OR t1.module ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+description+"%")
		argPos++
	}
	if module != "" {
		where += fmt.Sprintf(" AND t1.module = $%d", argPos)
		args = append(args, module)
		argPos++
	}
	if ip != "" {
		where += fmt.Sprintf(" AND (t1.ip ILIKE $%d OR t1.address ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+ip+"%")
		argPos++
	}
	if createUser != "" {
		where += fmt.Sprintf(" AND (t2.username ILIKE $%d OR t2.nickname ILIKE $%d)", argPos, argPos)
		args = append(args, "%"+createUser+"%")
		argPos++
	}
	if statusFilter != 0 {
		where += fmt.Sprintf(" AND t1.status = $%d", argPos)
		args = append(args, statusFilter)
		argPos++
	}
	if startTime != nil && endTime != nil {
		where += fmt.Sprintf(" AND t1.create_time BETWEEN $%d AND $%d", argPos, argPos+1)
		args = append(args, *startTime, *endTime)
		argPos += 2
	}

	// 登录日志/操作日志使用相同数据源，仅导出列标题不同。
	selectSQL := `
SELECT t1.id,
       t1.create_time,
       COALESCE(t2.nickname, ''),
       t1.description,
       t1.module,
       COALESCE(t1.status, 1),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.time_taken, 0)
`

	query := selectSQL + baseFrom + where + " ORDER BY t1.create_time DESC, t1.id DESC;"
	rows, err := h.db.QueryContext(c.Request.Context(), query, args...)
	if err != nil {
		Fail(c, "500", "导出日志失败")
		return
	}
	defer rows.Close()

	type rowItem struct {
		ID         int64
		CreateTime time.Time
		UserNick   string
		Desc       string
		Module     string
		Status     int16
		IP         string
		Address    string
		Browser    string
		OS         string
		TimeTaken  int64
	}

	var items []rowItem
	for rows.Next() {
		var r rowItem
		if err := rows.Scan(
			&r.ID,
			&r.CreateTime,
			&r.UserNick,
			&r.Desc,
			&r.Module,
			&r.Status,
			&r.IP,
			&r.Address,
			&r.Browser,
			&r.OS,
			&r.TimeTaken,
		); err != nil {
			Fail(c, "500", "解析日志数据失败")
			return
		}
		items = append(items, r)
	}
	if err := rows.Err(); err != nil {
		Fail(c, "500", "导出日志失败")
		return
	}

	if len(items) == 0 {
		// 无数据时返回一个空文件，避免前端认为是错误响应。
		c.Header("Content-Type", "text/csv; charset=utf-8")
		if isLogin {
			c.Header("Content-Disposition", "attachment; filename=\"login-log.csv\"")
		} else {
			c.Header("Content-Disposition", "attachment; filename=\"operation-log.csv\"")
		}
		c.String(http.StatusOK, "")
		return
	}

	c.Header("Content-Type", "text/csv; charset=utf-8")
	if isLogin {
		c.Header("Content-Disposition", "attachment; filename=\"login-log.csv\"")
	} else {
		c.Header("Content-Disposition", "attachment; filename=\"operation-log.csv\"")
	}

	// 简单按逗号分隔输出 CSV，字段中若包含逗号/换行可以按需增强转义，这里先满足基础导出需求。
	w := c.Writer

	if isLogin {
		// 登录日志导出列
		fmt.Fprintln(w, "ID,登录时间,用户昵称,登录行为,状态,登录 IP,登录地点,浏览器,终端系统")
		for _, r := range items {
			statusText := "成功"
			if r.Status != 1 {
				statusText = "失败"
			}
			line := fmt.Sprintf("%d,%s,%s,%s,%s,%s,%s,%s,%s",
				r.ID,
				formatTime(r.CreateTime),
				escapeCSV(r.UserNick),
				escapeCSV(r.Desc),
				statusText,
				escapeCSV(r.IP),
				escapeCSV(r.Address),
				escapeCSV(r.Browser),
				escapeCSV(r.OS),
			)
			fmt.Fprintln(w, line)
		}
	} else {
		// 操作日志导出列
		fmt.Fprintln(w, "ID,操作时间,操作人,操作内容,所属模块,状态,操作 IP,操作地点,耗时（ms）,浏览器,终端系统")
		for _, r := range items {
			statusText := "成功"
			if r.Status != 1 {
				statusText = "失败"
			}
			line := fmt.Sprintf("%d,%s,%s,%s,%s,%s,%s,%s,%d,%s,%s",
				r.ID,
				formatTime(r.CreateTime),
				escapeCSV(r.UserNick),
				escapeCSV(r.Desc),
				escapeCSV(r.Module),
				statusText,
				escapeCSV(r.IP),
				escapeCSV(r.Address),
				r.TimeTaken,
				escapeCSV(r.Browser),
				escapeCSV(r.OS),
			)
			fmt.Fprintln(w, line)
		}
	}
}

// escapeCSV 对包含逗号或引号的字段进行简单转义。
func escapeCSV(val string) string {
	if val == "" {
		return ""
	}
	if !strings.ContainsAny(val, ",\"\n\r") {
		return val
	}
	escaped := strings.ReplaceAll(val, `"`, `""`)
	return `"` + escaped + `"`
}
