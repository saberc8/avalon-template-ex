package http

import (
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/gin-gonic/gin"

	"voc-go-backend/internal/infrastructure/security"
)

// OnlineSession 表示一条在线会话记录（基于内存的简单实现）。
type OnlineSession struct {
	UserID         int64
	Username       string
	Nickname       string
	Token          string
	ClientType     string
	ClientID       string
	IP             string
	Address        string
	Browser        string
	OS             string
	LoginTime      time.Time
	LastActiveTime time.Time
}

// OnlineUserResp 与前端 OnlineUserResp 类型对齐。
type OnlineUserResp struct {
	ID             int64  `json:"id"`
	Token          string `json:"token"`
	Username       string `json:"username"`
	Nickname       string `json:"nickname"`
	ClientType     string `json:"clientType"`
	ClientID       string `json:"clientId"`
	IP             string `json:"ip"`
	Address        string `json:"address"`
	Browser        string `json:"browser"`
	OS             string `json:"os"`
	LoginTime      string `json:"loginTime"`
	LastActiveTime string `json:"lastActiveTime"`
}

// OnlineStore 维护当前进程内的在线会话信息。
type OnlineStore struct {
	mu       sync.RWMutex
	sessions map[string]*OnlineSession
}

// NewOnlineStore 创建内存在线会话存储。
func NewOnlineStore() *OnlineStore {
	return &OnlineStore{
		sessions: make(map[string]*OnlineSession),
	}
}

// RecordLogin 在用户登录成功后记录在线会话信息。
func (s *OnlineStore) RecordLogin(c *gin.Context, userID int64, username, nickname, clientID, token string) {
	if userID == 0 || token == "" {
		return
	}
	now := time.Now()
	ip := c.ClientIP()
	ua := c.Request.UserAgent()

	s.mu.Lock()
	defer s.mu.Unlock()

	s.sessions[token] = &OnlineSession{
		UserID:         userID,
		Username:       username,
		Nickname:       nickname,
		Token:          token,
		ClientType:     "PC",
		ClientID:       clientID,
		IP:             ip,
		Address:        "",
		Browser:        ua,
		OS:             "",
		LoginTime:      now,
		LastActiveTime: now,
	}
}

// RemoveByToken 根据 token 移除在线会话。
func (s *OnlineStore) RemoveByToken(token string) {
	if token == "" {
		return
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.sessions, token)
}

// List 返回按登录时间倒序的在线用户分页结果。
func (s *OnlineStore) List(nickname string, loginStart, loginEnd *time.Time, page, size int) ([]OnlineUserResp, int64) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}

	var filtered []*OnlineSession
	for _, sess := range s.sessions {
		if nickname != "" &&
			!strings.Contains(sess.Username, nickname) &&
			!strings.Contains(sess.Nickname, nickname) {
			continue
		}
		if loginStart != nil && sess.LoginTime.Before(*loginStart) {
			continue
		}
		if loginEnd != nil && sess.LoginTime.After(*loginEnd) {
			continue
		}
		filtered = append(filtered, sess)
	}

	// 按登录时间倒序排序
	if len(filtered) > 1 {
		for i := 0; i < len(filtered)-1; i++ {
			for j := i + 1; j < len(filtered); j++ {
				if filtered[i].LoginTime.Before(filtered[j].LoginTime) {
					filtered[i], filtered[j] = filtered[j], filtered[i]
				}
			}
		}
	}

	total := int64(len(filtered))
	start := (page - 1) * size
	if start > len(filtered) {
		start = len(filtered)
	}
	end := start + size
	if end > len(filtered) {
		end = len(filtered)
	}

	result := make([]OnlineUserResp, 0, end-start)
	for _, sess := range filtered[start:end] {
		result = append(result, OnlineUserResp{
			ID:             sess.UserID,
			Token:          sess.Token,
			Username:       sess.Username,
			Nickname:       sess.Nickname,
			ClientType:     sess.ClientType,
			ClientID:       sess.ClientID,
			IP:             sess.IP,
			Address:        sess.Address,
			Browser:        sess.Browser,
			OS:             sess.OS,
			LoginTime:      formatTime(sess.LoginTime),
			LastActiveTime: formatTime(sess.LastActiveTime),
		})
	}

	return result, total
}

// OnlineUserHandler 提供 /monitor/online 相关接口。
type OnlineUserHandler struct {
	store    *OnlineStore
	tokenSvc *security.TokenService
}

// NewOnlineUserHandler 创建在线用户 handler。
func NewOnlineUserHandler(store *OnlineStore, tokenSvc *security.TokenService) *OnlineUserHandler {
	return &OnlineUserHandler{
		store:    store,
		tokenSvc: tokenSvc,
	}
}

// RegisterOnlineUserRoutes 注册在线用户路由。
func (h *OnlineUserHandler) RegisterOnlineUserRoutes(r *gin.Engine) {
	r.GET("/monitor/online", h.PageOnlineUser)
	r.DELETE("/monitor/online/:token", h.Kickout)
}

// PageOnlineUser 处理 GET /monitor/online，返回分页在线用户列表。
func (h *OnlineUserHandler) PageOnlineUser(c *gin.Context) {
	page, _ := strconv.Atoi(c.Query("page"))
	size, _ := strconv.Atoi(c.Query("size"))
	nickname := strings.TrimSpace(c.Query("nickname"))

	var (
		startTime *time.Time
		endTime   *time.Time
	)

	timeRange := c.QueryArray("loginTime")
	if len(timeRange) == 2 {
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[0], time.Local); err == nil {
			startTime = &t
		}
		if t, err := time.ParseInLocation("2006-01-02 15:04:05", timeRange[1], time.Local); err == nil {
			endTime = &t
		}
	}

	list, total := h.store.List(nickname, startTime, endTime, page, size)
	OK(c, PageResult[OnlineUserResp]{List: list, Total: total})
}

// Kickout 处理 DELETE /monitor/online/:token，将指定 token 标记为下线。
func (h *OnlineUserHandler) Kickout(c *gin.Context) {
	token := c.Param("token")
	if strings.TrimSpace(token) == "" {
		Fail(c, "400", "令牌不能为空")
		return
	}

	authz := c.GetHeader("Authorization")
	raw := strings.TrimSpace(authz)
	if strings.HasPrefix(strings.ToLower(raw), "bearer ") {
		raw = strings.TrimSpace(raw[7:])
	}
	currentToken := raw

	if currentToken != "" && currentToken == token {
		Fail(c, "400", "不能强退自己")
		return
	}

	// 鉴权：仅需要校验当前请求 token 是否有效。
	if authz == "" {
		c.JSON(http.StatusOK, gin.H{
			"code":      "401",
			"data":      nil,
			"msg":       "未授权，请重新登录",
			"success":   false,
			"timestamp": nowString(),
		})
		return
	}
	if _, err := h.tokenSvc.Parse(authz); err != nil {
		c.JSON(http.StatusOK, gin.H{
			"code":      "401",
			"data":      nil,
			"msg":       "未授权，请重新登录",
			"success":   false,
			"timestamp": nowString(),
		})
		return
	}

	// 移除在线会话（当前实现仅维护内存状态，不影响 JWT 本身的有效性）。
	h.store.RemoveByToken(token)
	OK(c, true)
}
