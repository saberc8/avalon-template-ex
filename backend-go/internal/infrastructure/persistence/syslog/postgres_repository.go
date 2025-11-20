package syslog

import (
	"context"
	"database/sql"
	"time"

	domain "voc-go-backend/internal/domain/syslog"
	"voc-go-backend/internal/infrastructure/id"
)

// PgRepository 基于 PostgreSQL 的系统日志仓储实现。
type PgRepository struct {
	db *sql.DB
}

// NewPgRepository 创建基于 PostgreSQL 的系统日志仓储。
func NewPgRepository(db *sql.DB) *PgRepository {
	return &PgRepository{db: db}
}

var _ domain.Repository = (*PgRepository)(nil)

// Save 将日志记录插入 sys_log 表。
func (r *PgRepository) Save(ctx context.Context, rec *domain.Record) error {
	if rec == nil {
		return nil
	}
	if rec.ID == 0 {
		rec.ID = id.Next()
	}
	if rec.CreateTime.IsZero() {
		rec.CreateTime = time.Now()
	}

	const query = `
INSERT INTO sys_log (
    id,
    trace_id,
    description,
    module,
    request_url,
    request_method,
    request_headers,
    request_body,
    status_code,
    response_headers,
    response_body,
    time_taken,
    ip,
    address,
    browser,
    os,
    status,
    error_msg,
    create_user,
    create_time
) VALUES (
    $1, $2, $3, $4, $5,
    $6, $7, $8, $9, $10,
    $11, $12, $13, $14, $15,
    $16, $17, $18, $19, $20
);
`

	var createUser sql.NullInt64
	if rec.CreateUser != nil {
		createUser = sql.NullInt64{
			Int64: *rec.CreateUser,
			Valid: true,
		}
	}

	_, err := r.db.ExecContext(
		ctx,
		query,
		rec.ID,
		rec.TraceID,
		rec.Description,
		rec.Module,
		rec.RequestURL,
		rec.RequestMethod,
		rec.RequestHeaders,
		rec.RequestBody,
		rec.StatusCode,
		rec.ResponseHeaders,
		rec.ResponseBody,
		rec.TimeTaken,
		rec.IP,
		rec.Address,
		rec.Browser,
		rec.OS,
		int16(rec.Status),
		rec.ErrorMsg,
		createUser,
		rec.CreateTime,
	)
	return err
}

