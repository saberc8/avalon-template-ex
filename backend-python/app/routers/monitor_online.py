from __future__ import annotations

from datetime import datetime
from typing import List, Optional

from fastapi import APIRouter, Body, Header, Path, Query

from ..api_response import fail, ok
from ..config import get_settings
from ..security.jwt_token import TokenService
from ..security.online_store import OnlineUserResp, get_online_store

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _parse_datetime(value: str | None) -> Optional[datetime]:
    """解析前端传递的日期时间字符串，兼容 `YYYY-MM-DD HH:MM:SS`。"""
    if not value:
        return None
    s = value.strip()
    if not s:
        return None
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"):
        try:
            return datetime.strptime(s, fmt)
        except ValueError:
            continue
    return None


@router.get("/monitor/online")
def page_online(
    page: int = Query(1),
    size: int = Query(10),
    nickname: str | None = Query(default=None),
    loginTime: Optional[List[str]] = Query(default=None),
):
    """
    分页查询在线用户列表：GET /monitor/online。

    返回结构与前端 OnlineUserResp PageResult 对齐。
    """
    if page <= 0:
        page = 1
    if size <= 0:
        size = 10

    login_start: Optional[datetime] = None
    login_end: Optional[datetime] = None
    if loginTime and len(loginTime) == 2:
        login_start = _parse_datetime(loginTime[0])
        login_end = _parse_datetime(loginTime[1])

    store = get_online_store()
    list_resp, total = store.list(
        nickname=nickname,
        login_start=login_start,
        login_end=login_end,
        page=page,
        size=size,
    )

    return ok({"list": [u.__dict__ for u in list_resp], "total": total})


@router.delete("/monitor/online/{token}")
def kickout(
    token: str = Path(..., description="目标会话的 token"),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """
    强退在线用户：DELETE /monitor/online/{token}。

    行为与 Node/Go 版本保持一致：
    - 校验当前请求已登录，且不能强退自己；
    - 从在线会话列表中移除目标 token。
    """
    t = (token or "").strip()
    if not t:
        return fail("400", "令牌不能为空")

    authz = (authorization or "").strip()
    raw_current = authz
    lower = raw_current.lower()
    if lower.startswith("bearer "):
        raw_current = raw_current[7:].strip()

    if raw_current and raw_current == t:
        return fail("400", "不能强退自己")

    token_svc = _get_token_service()
    if not authz or not token_svc.parse(authz):
        return fail("401", "未授权，请重新登录")

    store = get_online_store()
    store.remove_by_token(t)
    return ok(True)

