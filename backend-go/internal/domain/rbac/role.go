package rbac

// Role represents a system role (sys_role).
type Role struct {
	ID        int64
	Name      string
	Code      string
	DataScope int32
}

