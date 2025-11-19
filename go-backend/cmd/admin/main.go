package main

import (
	"log"
	"os"
	"time"

	"github.com/gin-gonic/gin"

	appauth "voc-go-backend/internal/application/auth"
	rbacdomain "voc-go-backend/internal/domain/rbac"
	"voc-go-backend/internal/domain/user"
	"voc-go-backend/internal/infrastructure/db"
	rbacp "voc-go-backend/internal/infrastructure/persistence/rbac"
	persistence "voc-go-backend/internal/infrastructure/persistence/user"
	"voc-go-backend/internal/infrastructure/security"
	httpif "voc-go-backend/internal/interfaces/http"
)

func main() {
	// 1. 初始化数据库连接（PostgreSQL）
	dbCfg := db.LoadConfigFromEnv()
	pg, err := db.NewPostgres(dbCfg)
	if err != nil {
		log.Fatalf("failed to connect postgres: %v", err)
	}
	defer pg.Close()

	// 1.1 自动迁移/初始化数据库（仅 sys_user 和默认 admin）
	if err := db.AutoMigrate(pg); err != nil {
		log.Fatalf("failed to auto-migrate database: %v", err)
	}

	// 2. 初始化安全组件：RSA 解密器、BCrypt 密码校验、JWT 生成器
	rsaKey := getenvDefault("AUTH_RSA_PRIVATE_KEY",
		"MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAznV2Bi0zIX61NC3zSx8U6lJXbtru325pRV4Wt0aJXGxy6LMTsfxIye1ip+f2WnxrkYfk/X8YZ6FWNQPaAX/iRwIDAQABAkEAk/VcAusrpIqA5Ac2P5Tj0VX3cOuXmyouaVcXonr7f+6y2YTjLQuAnkcfKKocQI/juIRQBFQIqqW/m1nmz1wGeQIhAO8XaA/KxzOIgU0l/4lm0A2Wne6RokJ9HLs1YpOzIUmVAiEA3Q9DQrpAlIuiT1yWAGSxA9RxcjUM/1kdVLTkv0avXWsCIE0X8woEjK7lOSwzMG6RpEx9YHdopjViOj1zPVH61KTxAiBmv/dlhqkJ4rV46fIXELZur0pj6WC3N7a4brR8a+CLLQIhAMQyerWl2cPNVtE/8tkziHKbwW3ZUiBXU24wFxedT9iV",
	)
	rsaDecryptor, err := security.NewRSADecryptorFromBase64(rsaKey)
	if err != nil {
		log.Fatalf("failed to init RSA decryptor: %v", err)
	}
	pwdVerifier := security.BcryptVerifier{}
	pwdHasher := security.BcryptHasher{}

	jwtSecret := getenvDefault("AUTH_JWT_SECRET", "asdasdasifhueuiwyurfewbfjsdafjk")
	tokenTTL := 24 * time.Hour
	tokenSvc := security.NewTokenService(jwtSecret, tokenTTL)

	// 3. 初始化领域仓储和应用服务
	var userRepo user.Repository = persistence.NewPgRepository(pg)
	var roleRepo rbacdomain.RoleRepository = rbacp.NewPgRoleRepository(pg)
	var menuRepo rbacdomain.MenuRepository = rbacp.NewPgMenuRepository(pg)
	authSvc := appauth.NewService(userRepo, rsaDecryptor, pwdVerifier, tokenSvc)

	// 4. 初始化 HTTP 服务（Gin）
	r := gin.Default()

	// 全局 CORS（开发阶段允许前端本地调试）
	r.Use(func(c *gin.Context) {
		origin := c.Request.Header.Get("Origin")
		// 只在本地开发时放开 localhost:3000，如需更多域名可按需扩展
		if origin == "http://localhost:3000" {
			c.Writer.Header().Set("Access-Control-Allow-Origin", origin)
			c.Writer.Header().Set("Vary", "Origin")
		}
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")

		// 预检请求直接返回
		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	})

	// 公共接口
	commonHandler := httpif.NewCommonHandler(pg)
	commonHandler.RegisterCommonRoutes(r)

	// 验证码接口（简化版）
	captchaHandler := httpif.NewCaptchaHandler()
	captchaHandler.RegisterCaptchaRoutes(r)

	// 登录与用户接口
	authHandler := httpif.NewAuthHandler(authSvc)
	authHandler.RegisterAuthRoutes(r)
	userHandler := httpif.NewUserHandler(userRepo, roleRepo, menuRepo, tokenSvc)
	userHandler.RegisterUserRoutes(r)

	// 系统管理：菜单管理
	menuHandler := httpif.NewMenuHandler(pg, tokenSvc)
	menuHandler.RegisterMenuRoutes(r)

	// 系统管理：角色管理
	roleHandler := httpif.NewRoleHandler(pg, tokenSvc)
	roleHandler.RegisterRoleRoutes(r)

	// 系统管理：部门管理（仅树查询）
	deptHandler := httpif.NewDeptHandler(pg, tokenSvc)
	deptHandler.RegisterDeptRoutes(r)

	// 系统管理：用户管理
	systemUserHandler := httpif.NewSystemUserHandler(pg, tokenSvc, rsaDecryptor, pwdHasher)
	systemUserHandler.RegisterSystemUserRoutes(r)

	// 系统管理：字典管理
	dictHandler := httpif.NewDictHandler(pg, tokenSvc)
	dictHandler.RegisterDictRoutes(r)

	// 系统管理：文件管理
	fileHandler := httpif.NewFileHandler(pg, tokenSvc)
	fileHandler.RegisterFileRoutes(r)

	// 静态文件访问（上传文件）
	fileRoot := getenvDefault("FILE_STORAGE_DIR", "./data/file")
	r.Static("/file", fileRoot)

	// 5. 启动 HTTP 服务
	port := getenvDefault("HTTP_PORT", "4398")
	if err := r.Run(":" + port); err != nil {
		log.Fatalf("failed to start http server: %v", err)
	}
}

func getenvDefault(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}
