package syslog

import "context"

// Repository 定义系统日志持久化接口，便于后续扩展（例如异步落库、分库分表）。
type Repository interface {
	// Save 保存一条系统日志记录。
	Save(ctx context.Context, rec *Record) error
}

