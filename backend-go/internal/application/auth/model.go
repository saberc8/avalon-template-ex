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

// LoginResponse matches the Java LoginResp payload.
type LoginResponse struct {
	Token string `json:"token"`
}

