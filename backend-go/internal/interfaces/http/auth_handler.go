package http

import (
	"database/sql"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/mojocn/base64Captcha"

	"voc-go-backend/internal/application/auth"
)

// AuthHandler 暴露认证相关 HTTP 接口。
type AuthHandler struct {
	svc    *auth.Service
	online *OnlineStore
	db     *sql.DB
}

// NewAuthHandler 创建认证接口处理器。
// 其中 db 用于读取登录相关配置（如是否启用验证码）。
func NewAuthHandler(svc *auth.Service, online *OnlineStore, db *sql.DB) *AuthHandler {
	return &AuthHandler{
		svc:    svc,
		online: online,
		db:     db,
	}
}

// RegisterAuthRoutes 注册 /auth 相关路由。
func (h *AuthHandler) RegisterAuthRoutes(r *gin.Engine) {
	r.POST("/auth/login", h.Login)
	r.POST("/auth/logout", h.Logout)
}

// Login 处理 POST /auth/login。
// @Summary 用户登录
// @Description 使用账号密码进行登录，可选启用图形验证码。
// @Tags 认证
// @Accept json
// @Produce json
// @Param data body auth.LoginRequest true "登录请求参数"
// @Success 200 {object} map[string]interface{} "统一响应包装，data 为 LoginResponse"
// @Failure 200 {object} map[string]interface{} "失败时 code!=200，msg 为错误信息"
// @Router /auth/login [post]
func (h *AuthHandler) Login(c *gin.Context) {
	var req auth.LoginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		// 参数缺失或格式不正确
		Fail(c, "400", "参数缺失或格式不正确")
		return
	}

	// 当配置启用登录验证码且为账号登录时，先校验图形验证码。
	// 行为对齐 Java AccountLoginHandler.preLogin：
	// - LOGIN_CAPTCHA_ENABLED=0：不校验验证码；
	// - LOGIN_CAPTCHA_ENABLED!=0：必须校验 uuid + captcha。
	authType := strings.ToUpper(strings.TrimSpace(req.AuthType))
	if authType == "" || authType == "ACCOUNT" {
		enabled, err := isLoginCaptchaEnabled(c.Request.Context(), h.db)
		if err != nil {
			Fail(c, "500", "查询登录验证码配置失败")
			return
		}
		if enabled {
			if strings.TrimSpace(req.Captcha) == "" {
				Fail(c, "400", "验证码不能为空")
				return
			}
			if strings.TrimSpace(req.UUID) == "" {
				Fail(c, "400", "验证码标识不能为空")
				return
			}
			// 使用 base64Captcha.DefaultMemStore 校验验证码并在成功时自动删除。
			if ok := base64Captcha.DefaultMemStore.Verify(req.UUID, req.Captcha, true); !ok {
				Fail(c, "400", "验证码不正确或已过期")
				return
			}
		}
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

// Logout 处理 POST /auth/logout。
// 前端仅依赖服务端返回成功，本实现主要用于清理 Go 进程内的在线用户列表。
// @Summary 用户登出
// @Description 基于 Authorization Bearer Token 进行登出，仅清理服务端在线用户。
// @Tags 认证
// @Produce json
// @Success 200 {object} map[string]interface{} "统一响应包装，data 为 true/false"
// @Router /auth/logout [post]
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
