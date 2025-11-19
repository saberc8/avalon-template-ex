package auth

import (
	domain "voc-go-backend/internal/domain/user"
	rbac "voc-go-backend/internal/domain/rbac"
)

// UserInfo is the shape returned by /auth/user/info,
// aligned with the front-end's UserInfo type.
type UserInfo struct {
	ID              int64    `json:"id"`
	Username        string   `json:"username"`
	Nickname        string   `json:"nickname"`
	Gender          int16    `json:"gender"`
	Email           string   `json:"email"`
	Phone           string   `json:"phone"`
	Avatar          string   `json:"avatar"`
	Description     string   `json:"description"`
	PwdResetTime    string   `json:"pwdResetTime"`
	PwdExpired      bool     `json:"pwdExpired"`
	RegistrationDate string  `json:"registrationDate"`
	DeptName        string   `json:"deptName"`
	Roles           []string `json:"roles"`
	Permissions     []string `json:"permissions"`
}

// BuildUserInfo maps a domain.User to UserInfo.
// roles and permissions are supplied by caller (RBAC layer).
func BuildUserInfo(u *domain.User, roleCodes []string, permissions []string, deptName string, pwdExpired bool) UserInfo {
	var pwdResetTime string
	if u.PwdResetTime != nil {
		pwdResetTime = u.PwdResetTime.Format("2006-01-02 15:04:05")
	}
	regDate := u.CreateTime.Format("2006-01-02")

	var email, phone, avatar, desc string
	if u.Email != nil {
		email = *u.Email
	}
	if u.Phone != nil {
		phone = *u.Phone
	}
	if u.Avatar != nil {
		avatar = *u.Avatar
	}
	if u.Description != nil {
		desc = *u.Description
	}

	return UserInfo{
		ID:               u.ID,
		Username:         u.Username,
		Nickname:         u.Nickname,
		Gender:           u.Gender,
		Email:            email,
		Phone:            phone,
		Avatar:           avatar,
		Description:      desc,
		PwdResetTime:     pwdResetTime,
		PwdExpired:       pwdExpired,
		RegistrationDate: regDate,
		DeptName:         deptName,
		Roles:            roleCodes,
		Permissions:      permissions,
	}
}

// ExtractRoleCodes helper to convert []Role to []string of code.
func ExtractRoleCodes(roles []rbac.Role) []string {
	if len(roles) == 0 {
		return []string{}
	}
	out := make([]string, 0, len(roles))
	for _, r := range roles {
		out = append(out, r.Code)
	}
	return out
}
