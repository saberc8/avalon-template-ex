import time
from dataclasses import dataclass
from typing import Optional

import jwt


@dataclass
class TokenClaims:
    """JWT Claims 结构，仅保留 userId 字段以匹配前端。"""

    user_id: int


@dataclass
class TokenService:
    """JWT 生成与解析服务，保持与 Go 版字段一致。"""

    secret: str
    ttl_seconds: int

    def generate(self, user_id: int) -> str:
        """生成包含 userId、iat、exp 的 HS256 Token。"""
        now = int(time.time())
        payload = {
            "userId": user_id,
            "iat": now,
            "exp": now + self.ttl_seconds,
        }
        return jwt.encode(payload, self.secret, algorithm="HS256")

    def parse(self, token_str: str) -> Optional[TokenClaims]:
        """解析 Token，支持 `Bearer xxx` 形式，返回 TokenClaims。"""
        if not token_str:
            return None
        token = token_str.strip()
        lower = token.lower()
        if lower.startswith("bearer "):
            token = token[7:].strip()
        try:
            data = jwt.decode(token, self.secret, algorithms=["HS256"])
        except Exception:
            return None
        user_id = data.get("userId")
        if not isinstance(user_id, int):
            return None
        return TokenClaims(user_id=user_id)


