from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from threading import Lock
from typing import Dict, List, Optional, Tuple


@dataclass
class OnlineSession:
    """在线会话记录结构，对齐 Go/Node 版本的 OnlineSession。"""

    user_id: int
    username: str
    nickname: str
    token: str
    client_type: str
    client_id: str
    ip: str
    address: str
    browser: str
    os: str
    login_time: datetime
    last_active_time: datetime


@dataclass
class OnlineUserResp:
    """在线用户响应结构，对齐前端 OnlineUserResp。"""

    id: int
    token: str
    username: str
    nickname: str
    clientType: str
    clientId: str
    ip: str
    address: str
    browser: str
    os: str
    loginTime: str
    lastActiveTime: str


def _pad(num: int) -> str:
    """时间数字补零函数。"""
    return f"{num:02d}"


def _format_dt(dt: datetime) -> str:
    """将时间格式化为 YYYY-MM-DD HH:MM:SS，与前端展示一致。"""
    return (
        f"{dt.year:04d}-{_pad(dt.month)}-{_pad(dt.day)} "
        f"{_pad(dt.hour)}:{_pad(dt.minute)}:{_pad(dt.second)}"
    )


class OnlineStore:
    """
    在线会话内存存储，生命周期与 Python 进程一致。

    说明：
    - 与 Node 版 OnlineStoreService 行为保持一致，仅在内存中保存会话；
    - 服务重启后会话将被清空，适用于单进程部署场景。
    """

    def __init__(self) -> None:
        self._sessions: Dict[str, OnlineSession] = {}
        self._lock = Lock()

    def record_login(
        self,
        *,
        user_id: int,
        username: str,
        nickname: str,
        client_id: str,
        token: str,
        ip: str | None = None,
        user_agent: str | None = None,
    ) -> None:
        """记录登录会话信息，在登录成功后调用。"""
        if not user_id or not token:
            return
        now = datetime.now()
        ip_val = (ip or "").strip()
        ua = (user_agent or "").strip()

        session = OnlineSession(
            user_id=user_id,
            username=username,
            nickname=nickname,
            token=token,
            client_type="PC",
            client_id=client_id,
            ip=ip_val,
            address="",
            browser=ua,
            os="",
            login_time=now,
            last_active_time=now,
        )
        with self._lock:
            self._sessions[token] = session

    def remove_by_token(self, token: str) -> None:
        """根据 token 移除在线会话，用于强退或登出。"""
        t = (token or "").strip()
        if not t:
            return
        with self._lock:
            self._sessions.pop(t, None)

    def list(
        self,
        *,
        nickname: str | None,
        login_start: Optional[datetime],
        login_end: Optional[datetime],
        page: int,
        size: int,
    ) -> Tuple[List[OnlineUserResp], int]:
        """
        查询在线用户列表，按登录时间倒序分页。

        返回值：
        - 在线用户响应列表；
        - 总记录数。
        """
        page = page if page > 0 else 1
        size = size if size > 0 else 10
        nickname_val = (nickname or "").strip()

        with self._lock:
            all_sessions = list(self._sessions.values())

        filtered: List[OnlineSession] = []
        for sess in all_sessions:
            if nickname_val and (
                nickname_val not in sess.username and nickname_val not in sess.nickname
            ):
                continue
            if login_start and sess.login_time < login_start:
                continue
            if login_end and sess.login_time > login_end:
                continue
            filtered.append(sess)

        if len(filtered) > 1:
            filtered.sort(key=lambda s: s.login_time, reverse=True)

        total = len(filtered)
        start = (page - 1) * size
        end = start + size
        if start < 0:
            start = 0
        if end > total:
            end = total
        page_items = filtered[start:end]

        list_resp = [
            OnlineUserResp(
                id=sess.user_id,
                token=sess.token,
                username=sess.username,
                nickname=sess.nickname,
                clientType=sess.client_type,
                clientId=sess.client_id,
                ip=sess.ip,
                address=sess.address,
                browser=sess.browser,
                os=sess.os,
                loginTime=_format_dt(sess.login_time),
                lastActiveTime=_format_dt(sess.last_active_time),
            )
            for sess in page_items
        ]
        return list_resp, total


online_store = OnlineStore()


def get_online_store() -> OnlineStore:
    """获取全局在线会话存储实例。"""
    return online_store

