package syslog

import "time"

// Status 表示系统日志状态（1：成功；2：失败）。
type Status int16

const (
	StatusSuccess Status = 1
	StatusFailure Status = 2
)

// Record 表示一条系统日志记录，对应 sys_log 表。
// 字段设计参考 Java 版 LogDO 与 PostgreSQL 表结构。
type Record struct {
	ID          int64
	TraceID     string
	Description string
	Module      string

	RequestURL     string
	RequestMethod  string
	RequestHeaders string
	RequestBody    string

	StatusCode      int
	ResponseHeaders string
	ResponseBody    string

	TimeTaken int64 // 耗时（毫秒）

	IP      string
	Address string
	Browser string
	OS      string

	Status   Status
	ErrorMsg string

	CreateUser *int64
	CreateTime time.Time
}

