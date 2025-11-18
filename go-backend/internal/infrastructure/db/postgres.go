package db

import (
	"database/sql"
	"fmt"
	"os"
	"time"

	_ "github.com/lib/pq"
)

// Config holds PostgreSQL connection configuration.
type Config struct {
	Host     string
	Port     string
	User     string
	Password string
	DBName   string
	SSLMode  string
}

// LoadConfigFromEnv builds a Config from environment variables with
// reasonable defaults matching the existing Java project.
func LoadConfigFromEnv() Config {
	cfg := Config{
		Host:     getenvDefault("DB_HOST", "127.0.0.1"),
		Port:     getenvDefault("DB_PORT", "5432"),
		User:     getenvDefault("DB_USER", "postgres"),
		Password: getenvDefault("DB_PWD", "123456"),
		DBName:   getenvDefault("DB_NAME", "nv_admin"),
		SSLMode:  getenvDefault("DB_SSLMODE", "disable"),
	}
	return cfg
}

// NewPostgres opens a PostgreSQL connection using the given config.
func NewPostgres(cfg Config) (*sql.DB, error) {
	dsn := fmt.Sprintf(
		"host=%s port=%s user=%s password=%s dbname=%s sslmode=%s",
		cfg.Host, cfg.Port, cfg.User, cfg.Password, cfg.DBName, cfg.SSLMode,
	)

	db, err := sql.Open("postgres", dsn)
	if err != nil {
		return nil, err
	}

	// Basic pool tuning; adjust as needed.
	db.SetMaxOpenConns(20)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(30 * time.Minute)

	if err := db.Ping(); err != nil {
		_ = db.Close()
		return nil, err
	}
	return db, nil
}

func getenvDefault(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}

