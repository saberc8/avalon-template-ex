use std::env;

/// 应用配置，复用 backend-go / Java 的环境变量约定。
pub struct AppConfig {
    /// 数据库主机（DB_HOST）
    pub db_host: String,
    /// 数据库端口（DB_PORT）
    pub db_port: String,
    /// 数据库用户名（DB_USER）
    pub db_user: String,
    /// 数据库密码（DB_PWD）
    pub db_pwd: String,
    /// 数据库名（DB_NAME）
    pub db_name: String,

    /// JWT 密钥（AUTH_JWT_SECRET）
    pub auth_jwt_secret: String,
    /// JWT 有效期（小时），目前固定为 24，与 Go 实现保持一致
    pub auth_jwt_ttl_hours: i64,

    /// RSA 私钥（Base64 PKCS#8，AUTH_RSA_PRIVATE_KEY）
    pub auth_rsa_private_key: String,

    /// 静态文件根目录（FILE_STORAGE_DIR）
    pub file_storage_dir: String,

    /// HTTP 监听端口（HTTP_PORT）
    pub http_port: u16,
}

impl AppConfig {
    /// 从环境变量读取配置，提供与 backend-go 相同的默认值。
    pub fn from_env() -> Self {
        fn getenv(key: &str, default: &str) -> String {
            env::var(key).unwrap_or_else(|_| default.to_string())
        }

        let db_host = getenv("DB_HOST", "127.0.0.1");
        let db_port = getenv("DB_PORT", "5432");
        let db_user = getenv("DB_USER", "postgres");
        let db_pwd = getenv("DB_PWD", "123456");
        let db_name = getenv("DB_NAME", "nv_admin");

        let auth_jwt_secret =
            getenv("AUTH_JWT_SECRET", "asdasdasifhueuiwyurfewbfjsdafjk");
        let auth_rsa_private_key = getenv(
            "AUTH_RSA_PRIVATE_KEY",
            "MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAznV2Bi0zIX61NC3zSx8U6lJXbtru325pRV4Wt0aJXGxy6LMTsfxIye1ip+f2WnxrkYfk/X8YZ6FWNQPaAX/iRwIDAQABAkEAk/VcAusrpIqA5Ac2P5Tj0VX3cOuXmyouaVcXonr7f+6y2YTjLQuAnkcfKKocQI/juIRQBFQIqqW/m1nmz1wGeQIhAO8XaA/KxzOIgU0l/4lm0A2Wne6RokJ9HLs1YpOzIUmVAiEA3Q9DQrpAlIuiT1yWAGSxA9RxcjUM/1kdVLTkv0avXWsCIE0X8woEjK7lOSwzMG6RpEx9YHdopjViOj1zPVH61KTxAiBmv/dlhqkJ4rV46fIXELZur0pj6WC3N7a4brR8a+CLLQIhAMQyerWl2cPNVtE/8tkziHKbwW3ZUiBXU24wFxedT9iV",
        );

        let file_storage_dir = getenv("FILE_STORAGE_DIR", "./data/file");
        let http_port_str = getenv("HTTP_PORT", "4398");
        let http_port = http_port_str.parse::<u16>().unwrap_or(4398);

        AppConfig {
            db_host,
            db_port,
            db_user,
            db_pwd,
            db_name,
            auth_jwt_secret,
            auth_jwt_ttl_hours: 24,
            auth_rsa_private_key,
            file_storage_dir,
            http_port,
        }
    }

    /// 构造 PostgreSQL 连接字符串，供 sqlx 使用。
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{user}:{pwd}@{host}:{port}/{db}",
            user = self.db_user,
            pwd = self.db_pwd,
            host = self.db_host,
            port = self.db_port,
            db = self.db_name
        )
    }
}

