package rbac

import "context"

// RoleRepository provides access to roles and their association with users.
type RoleRepository interface {
	// ListByUserID returns all roles bound to a user.
	ListByUserID(ctx context.Context, userID int64) ([]Role, error)
	// ListCodesByUserID returns role codes for a user.
	ListCodesByUserID(ctx context.Context, userID int64) ([]string, error)
}

// MenuRepository provides access to menus and permissions.
type MenuRepository interface {
	// ListByRoleID returns all menus bound to a role.
	ListByRoleID(ctx context.Context, roleID int64) ([]Menu, error)
	// ListPermissionsByUserID returns all permission strings for a user.
	ListPermissionsByUserID(ctx context.Context, userID int64) ([]string, error)
}

