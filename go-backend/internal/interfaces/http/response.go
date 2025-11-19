package http

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
)

// APIResponse matches the front-end's ApiRes<T> definition.
type APIResponse[T any] struct {
	Code      string `json:"code"`
	Data      T      `json:"data"`
	Msg       string `json:"msg"`
	Success   bool   `json:"success"`
	Timestamp string `json:"timestamp"`
}

// PageResult represents a generic paginated result.
type PageResult[T any] struct {
	List  []T   `json:"list"`
	Total int64 `json:"total"`
}

// nowString returns current time as epoch milliseconds string,
// matching the front-end expectation (Number(res.timestamp)).
func nowString() string {
	return strconv.FormatInt(time.Now().UnixMilli(), 10)
}

// OK wraps data in a success response.
func OK[T any](c *gin.Context, data T) {
	resp := APIResponse[T]{
		Code:      "200",
		Data:      data,
		Msg:       "操作成功",
		Success:   true,
		Timestamp: nowString(),
	}
	c.JSON(http.StatusOK, resp)
}

// Fail returns a failed response with the given code and message.
func Fail(c *gin.Context, code, msg string) {
	resp := APIResponse[any]{
		Code:      code,
		Data:      nil,
		Msg:       msg,
		Success:   false,
		Timestamp: nowString(),
	}
	c.JSON(http.StatusOK, resp)
}
