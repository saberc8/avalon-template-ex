package user

import "time"

// User is the domain entity for a system user.
// It mirrors the Java UserDO and the sys_user PostgreSQL table.
type User struct {
	ID           int64
	Username     string
	Nickname     string
	Password     string
	Gender       int16
	Email        *string
	Phone        *string
	Avatar       *string
	Description  *string
	Status       int16
	IsSystem     bool
	PwdResetTime *time.Time
	DeptID       int64

	CreateUser  *int64
	CreateTime  time.Time
	UpdateUser  *int64
	UpdateTime  *time.Time
}

// IsEnabled returns true if the user status is "enabled" (1).
func (u *User) IsEnabled() bool {
	return u != nil && u.Status == 1
}

