package http

import (
	"github.com/gin-gonic/gin"
)

// LabelValue represents a simple label/value pair for dictionaries.
type LabelValue struct {
	Label string `json:"label"`
	Value string `json:"value"`
	Extra string `json:"extra,omitempty"`
}

// CommonHandler exposes /common related endpoints.
type CommonHandler struct{}

func NewCommonHandler() *CommonHandler {
	return &CommonHandler{}
}

// RegisterCommonRoutes registers /common endpoints.
func (h *CommonHandler) RegisterCommonRoutes(r *gin.Engine) {
	r.GET("/common/dict/option/site", h.ListSiteOptions)
}

// ListSiteOptions returns basic site configuration dictionary.
// It mirrors the SITE_* options used by the Java backend.
func (h *CommonHandler) ListSiteOptions(c *gin.Context) {
	data := []LabelValue{
		{Label: "SITE_TITLE", Value: "ContiNew Admin"},
		{Label: "SITE_DESCRIPTION", Value: "持续迭代优化的前后端分离中后台管理系统框架"},
		{Label: "SITE_COPYRIGHT", Value: "Copyright © 2022 - present ContiNew Admin 版权所有"},
		{Label: "SITE_BEIAN", Value: ""},
		{Label: "SITE_FAVICON", Value: "/favicon.ico"},
		{Label: "SITE_LOGO", Value: "/logo.svg"},
	}
	OK(c, data)
}

