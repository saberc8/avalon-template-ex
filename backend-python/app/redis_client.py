from __future__ import annotations

import threading
from typing import Optional

import redis

from .config import get_settings

_lock = threading.Lock()
_client: Optional[redis.Redis] = None


def get_redis_client() -> redis.Redis:
    """获取全局 Redis 客户端实例，配置参考 Java application-dev.yml。"""
    global _client
    if _client is not None:
        return _client
    with _lock:
        if _client is not None:
            return _client
        s = get_settings()
        kwargs: dict = {
            "host": s.redis_host,
            "port": s.redis_port,
            "db": s.redis_db,
            "decode_responses": True,
        }
        if s.redis_password:
            kwargs["password"] = s.redis_password
        _client = redis.Redis(**kwargs)
        return _client


def redis_set(key: str, value: str, ex_seconds: int) -> None:
    """写入字符串键值，指定过期时间（秒）。"""
    client = get_redis_client()
    client.set(name=key, value=value, ex=ex_seconds)


def redis_get(key: str) -> Optional[str]:
    """读取字符串键值，不存在时返回 None。"""
    client = get_redis_client()
    return client.get(name=key)


def redis_delete(key: str) -> None:
    """删除指定键。"""
    client = get_redis_client()
    client.delete(key)

