from contextlib import contextmanager
from typing import Iterator

import psycopg2
from psycopg2.extras import RealDictCursor

from .config import get_settings


def _build_dsn() -> str:
    """构造 PostgreSQL 连接 DSN 字符串。"""
    s = get_settings()
    return (
        f"host={s.db_host} port={s.db_port} user={s.db_user} "
        f"password={s.db_password} dbname={s.db_name} sslmode={s.db_sslmode}"
    )


@contextmanager
def get_db_cursor() -> Iterator[RealDictCursor]:
    """
    获取数据库游标（简单版，每次请求新建连接）。

    说明：
    - 与 Go 版共用同一套表结构与数据；
    - 这里采用简单连接模型，后续如需优化可改为连接池。
    """
    conn = psycopg2.connect(dsn=_build_dsn(), cursor_factory=RealDictCursor)
    try:
        with conn.cursor() as cursor:
            yield cursor
            conn.commit()
    finally:
        conn.close()


