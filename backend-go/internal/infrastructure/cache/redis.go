package cache

import (
	"context"
	"os"
	"strconv"
	"time"

	"github.com/redis/go-redis/v9"
)

// Config 表示 Redis 连接配置。
type Config struct {
	Addr     string
	Password string
	DB       int
}

// LoadConfigFromEnv 从环境变量加载 Redis 配置，默认对齐 Java 工程：
// - REDIS_HOST（默认 127.0.0.1）
// - REDIS_PORT（默认 6379）
// - REDIS_PWD  （可选）
// - REDIS_DB   （默认 0）
func LoadConfigFromEnv() Config {
	host := getenvDefault("REDIS_HOST", "127.0.0.1")
	port := getenvDefault("REDIS_PORT", "6379")
	addr := host + ":" + port

	dbIndexStr := getenvDefault("REDIS_DB", "0")
	dbIndex, err := strconv.Atoi(dbIndexStr)
	if err != nil {
		dbIndex = 0
	}

	return Config{
		Addr:     addr,
		Password: os.Getenv("REDIS_PWD"),
		DB:       dbIndex,
	}
}

// NewRedis 根据配置创建 Redis 客户端并做一次 Ping 校验。
func NewRedis(cfg Config) (*redis.Client, error) {
	client := redis.NewClient(&redis.Options{
		Addr:     cfg.Addr,
		Password: cfg.Password,
		DB:       cfg.DB,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Ping(ctx).Err(); err != nil {
		_ = client.Close()
		return nil, err
	}

	return client, nil
}

func getenvDefault(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}

