from __future__ import annotations

from typing import Mapping, Optional, Tuple

from fastapi import Request


def get_client_ip(request: Request) -> str:
    """从请求头或连接信息中获取客户端 IP。"""
    xff = request.headers.get("x-forwarded-for", "") or request.headers.get(
        "X-Forwarded-For", ""
    )
    if xff:
        return (xff.split(",")[0] or "").strip()
    if request.client:
        return request.client.host or ""
    return ""


def get_browser_and_os(user_agent: str) -> Tuple[str, str]:
    """
    简单从 User-Agent 中提取浏览器和系统信息。

    说明：这里不做复杂解析，直接返回 UA 字符串作为浏览器，操作系统留空即可满足前端展示。
    """
    ua = (user_agent or "").strip()
    if not ua:
        return "", ""
    return ua, ""


def get_authorization_user_id(headers: Mapping[str, str]) -> Optional[int]:
    """从请求头中的 Authorization 解析当前用户 ID。"""
    from ..config import get_settings
    from ..security.jwt_token import TokenService

    auth = headers.get("Authorization") or headers.get("authorization")
    if not auth:
        return None
    settings = get_settings()
    token_svc = TokenService(
        secret=settings.jwt_secret, ttl_seconds=settings.jwt_ttl_hours * 3600
    )
    claims = token_svc.parse(auth)
    if not claims:
        return None
    return claims.user_id

