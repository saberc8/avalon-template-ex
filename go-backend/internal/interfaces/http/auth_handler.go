package http

import (
	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/application/auth"
)

// AuthHandler exposes authentication HTTP endpoints.
type AuthHandler struct {
	svc *auth.Service
}

func NewAuthHandler(svc *auth.Service) *AuthHandler {
	return &AuthHandler{svc: svc}
}

// RegisterAuthRoutes registers /auth related routes on the given router group.
func (h *AuthHandler) RegisterAuthRoutes(r *gin.Engine) {
	r.POST("/auth/login", h.Login)
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

	// Successful login, return LoginResp as data.
	c.Header("Content-Type", "application/json; charset=utf-8")
	OK(c, resp)
}
