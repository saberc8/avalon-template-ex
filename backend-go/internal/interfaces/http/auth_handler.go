package http

import (
	"strings"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/application/auth"
)

// AuthHandler exposes authentication HTTP endpoints.
type AuthHandler struct {
	svc    *auth.Service
	online *OnlineStore
}

func NewAuthHandler(svc *auth.Service, online *OnlineStore) *AuthHandler {
	return &AuthHandler{
		svc:    svc,
		online: online,
	}
}

// RegisterAuthRoutes registers /auth related routes on the given router group.
func (h *AuthHandler) RegisterAuthRoutes(r *gin.Engine) {
	r.POST("/auth/login", h.Login)
	r.POST("/auth/logout", h.Logout)
}

// Login handles POST /auth/login.
func (h *AuthHandler) Login(c *gin.Context) {
	var req auth.LoginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		// 参数缺失或格式不正确
		Fail(c, "400", "参数缺失或格式不正确")
		return
	}

	resp, err := h.svc.Login(c.Request.Context(), req)
	if err != nil {
		// Treat any error returned by the service as a 400-style business error,
		// with the message coming from the service (already localized).
		Fail(c, "400", err.Error())
		return
	}

	// 登录成功后记录在线用户信息（仅在当前 Go 进程内维护内存状态）。
	if h.online != nil && resp != nil {
		h.online.RecordLogin(c, resp.UserID, resp.Username, resp.Nickname, req.ClientID, resp.Token)
	}

	// Successful login, return LoginResp as data.
	c.Header("Content-Type", "application/json; charset=utf-8")
	OK(c, resp)
}

// Logout handles POST /auth/logout.
// 前端仅依赖服务端返回成功，本实现主要用于清理 Go 进程内的在线用户列表。
func (h *AuthHandler) Logout(c *gin.Context) {
	authz := c.GetHeader("Authorization")
	token := strings.TrimSpace(authz)
	if strings.HasPrefix(strings.ToLower(token), "bearer ") {
		token = strings.TrimSpace(token[7:])
	}
	if token != "" && h.online != nil {
		h.online.RemoveByToken(token)
	}
	OK(c, true)
}
