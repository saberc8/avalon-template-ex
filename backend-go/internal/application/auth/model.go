package auth

// LoginRequest mirrors the JSON structure sent by the front-end
// when calling POST /auth/login with ACCOUNT auth type.
type LoginRequest struct {
	ClientID string `json:"clientId"`
	AuthType string `json:"authType"`

	Username string `json:"username"`
	Password string `json:"password"` // RSA + Base64 encrypted
	Captcha  string `json:"captcha"`
	UUID     string `json:"uuid"`
}

// LoginResponse 匹配 Java LoginResp 返回结构，前端当前仅使用 token 字段。
// 额外携带的用户基础信息用于 Go 端在线用户统计，不会影响前端兼容性。
type LoginResponse struct {
	Token    string `json:"token"`
	UserID   int64  `json:"userId,omitempty"`
	Username string `json:"username,omitempty"`
	Nickname string `json:"nickname,omitempty"`
}
