package rbac

import (
	"context"
	"database/sql"

	domain "voc-go-backend/internal/domain/rbac"
)

// PgRoleRepository implements RoleRepository using PostgreSQL tables
// sys_role and sys_user_role.
type PgRoleRepository struct {
	db *sql.DB
}

func NewPgRoleRepository(db *sql.DB) *PgRoleRepository {
	return &PgRoleRepository{db: db}
}

var _ domain.RoleRepository = (*PgRoleRepository)(nil)

// ListByUserID returns roles for a given user id.
func (r *PgRoleRepository) ListByUserID(ctx context.Context, userID int64) ([]domain.Role, error) {
	const query = `
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = $1;
`
	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var roles []domain.Role
	for rows.Next() {
		var rl domain.Role
		if err := rows.Scan(&rl.ID, &rl.Name, &rl.Code, &rl.DataScope); err != nil {
			return nil, err
		}
		roles = append(roles, rl)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return roles, nil
}

// ListCodesByUserID returns role codes for a given user.
func (r *PgRoleRepository) ListCodesByUserID(ctx context.Context, userID int64) ([]string, error) {
	const query = `
SELECT r.code
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = $1;
`
	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var codes []string
	for rows.Next() {
		var code string
		if err := rows.Scan(&code); err != nil {
			return nil, err
		}
		codes = append(codes, code)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return codes, nil
}

