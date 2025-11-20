import os
from functools import lru_cache


class Settings:
    """应用配置，主要来自环境变量，保持与 Go 版一致的命名。"""

    # 数据库配置
    db_host: str = os.getenv("DB_HOST", "127.0.0.1")
    db_port: str = os.getenv("DB_PORT", "5432")
    db_user: str = os.getenv("DB_USER", "postgres")
    db_password: str = os.getenv("DB_PWD", "123456")
    db_name: str = os.getenv("DB_NAME", "nv_admin")
    db_sslmode: str = os.getenv("DB_SSLMODE", "disable")

    # 认证配置（与 Go/Java 保持一致）
    rsa_private_key_b64: str = os.getenv(
        "AUTH_RSA_PRIVATE_KEY",
        "MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAznV2Bi0zIX61NC3zSx8U6lJXbtru325pRV4Wt0aJXGxy6LMTsfxIye1ip+f2WnxrkYfk/X8YZ6FWNQPaAX/iRwIDAQABAkEAk/VcAusrpIqA5Ac2P5Tj0VX3cOuXmyouaVcXonr7f+6y2YTjLQuAnkcfKKocQI/juIRQBFQIqqW/m1nmz1wGeQIhAO8XaA/KxzOIgU0l/4lm0A2Wne6RokJ9HLs1YpOzIUmVAiEA3Q9DQrpAlIuiT1yWAGSxA9RxcjUM/1kdVLTkv0avXWsCIE0X8woEjK7lOSwzMG6RpEx9YHdopjViOj1zPVH61KTxAiBmv/dlhqkJ4rV46fIXELZur0pj6WC3N7a4brR8a+CLLQIhAMQyerWl2cPNVtE/8tkziHKbwW3ZUiBXU24wFxedT9iV",  # noqa: E501
    )
    jwt_secret: str = os.getenv("AUTH_JWT_SECRET", "asdasdasifhueuiwyurfewbfjsdafjk")
    jwt_ttl_hours: int = int(os.getenv("AUTH_JWT_TTL_HOURS", "24"))


@lru_cache()
def get_settings() -> Settings:
    """获取单例配置实例。"""
    return Settings()


