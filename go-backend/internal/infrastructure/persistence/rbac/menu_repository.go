package rbac

import (
	"context"
	"database/sql"

	domain "voc-go-backend/internal/domain/rbac"
)

// PgMenuRepository implements MenuRepository using PostgreSQL tables
// sys_menu, sys_role_menu, sys_user_role, sys_role and sys_user.
type PgMenuRepository struct {
	db *sql.DB
}

func NewPgMenuRepository(db *sql.DB) *PgMenuRepository {
	return &PgMenuRepository{db: db}
}

var _ domain.MenuRepository = (*PgMenuRepository)(nil)

// ListByRoleID returns menus for a given role id.
func (r *PgMenuRepository) ListByRoleID(ctx context.Context, roleID int64) ([]domain.Menu, error) {
	const query = `
SELECT
  m.id,
  m.parent_id,
  m.title,
  m.type,
  m.path,
  m.name,
  m.component,
  m.redirect,
  m.icon,
  COALESCE(m.is_external, false),
  COALESCE(m.is_cache, false),
  COALESCE(m.is_hidden, false),
  m.permission,
  COALESCE(m.sort, 0),
  m.status
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
WHERE rm.role_id = $1;
`
	rows, err := r.db.QueryContext(ctx, query, roleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var menus []domain.Menu
	for rows.Next() {
		var (
			m                 domain.Menu
			path, name        sql.NullString
			component         sql.NullString
			redirect          sql.NullString
			icon              sql.NullString
			permission        sql.NullString
		)
		if err := rows.Scan(
			&m.ID,
			&m.ParentID,
			&m.Title,
			&m.Type,
			&path,
			&name,
			&component,
			&redirect,
			&icon,
			&m.IsExternal,
			&m.IsCache,
			&m.IsHidden,
			&permission,
			&m.Sort,
			&m.Status,
		); err != nil {
			return nil, err
		}
		if path.Valid {
			m.Path = path.String
		}
		if name.Valid {
			m.Name = name.String
		}
		if component.Valid {
			m.Component = component.String
		}
		if redirect.Valid {
			m.Redirect = redirect.String
		}
		if icon.Valid {
			m.Icon = icon.String
		}
		if permission.Valid {
			m.Permission = permission.String
		}
		menus = append(menus, m)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return menus, nil
}

// ListPermissionsByUserID returns distinct permission strings for a user.
func (r *PgMenuRepository) ListPermissionsByUserID(ctx context.Context, userID int64) ([]string, error) {
	const query = `
SELECT DISTINCT m.permission
FROM sys_menu AS m
LEFT JOIN sys_role_menu AS rm ON rm.menu_id = m.id
LEFT JOIN sys_role AS r ON r.id = rm.role_id
LEFT JOIN sys_user_role AS ur ON ur.role_id = r.id
LEFT JOIN sys_user AS u ON u.id = ur.user_id
WHERE u.id = $1
  AND m.status = 1
  AND m.permission IS NOT NULL;
`
	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var perms []string
	for rows.Next() {
		var p string
		if err := rows.Scan(&p); err != nil {
			return nil, err
		}
		perms = append(perms, p)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return perms, nil
}
