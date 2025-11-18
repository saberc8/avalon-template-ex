package http

import (
	"time"

	"github.com/gin-gonic/gin"
)

// CaptchaResp matches the Java CaptchaResp structure.
type CaptchaResp struct {
	UUID       string `json:"uuid"`
	Img        string `json:"img"`
	ExpireTime int64  `json:"expireTime"`
	IsEnabled  bool   `json:"isEnabled"`
}

// CaptchaHandler exposes /captcha endpoints.
type CaptchaHandler struct{}

func NewCaptchaHandler() *CaptchaHandler {
	return &CaptchaHandler{}
}

// RegisterCaptchaRoutes registers /captcha endpoints.
func (h *CaptchaHandler) RegisterCaptchaRoutes(r *gin.Engine) {
	r.GET("/captcha/image", h.GetImageCaptcha)
}

// GetImageCaptcha implements a simplified captcha:
// it always returns isEnabled=false, so the front-end hides the captcha input.
func (h *CaptchaHandler) GetImageCaptcha(c *gin.Context) {
	now := time.Now().UnixMilli()
	resp := CaptchaResp{
		UUID:       "",
		Img:        "",
		ExpireTime: now,
		IsEnabled:  false,
	}
	OK(c, resp)
}

