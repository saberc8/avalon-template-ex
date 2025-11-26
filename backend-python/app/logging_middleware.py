from __future__ import annotations

import json
import time
from typing import Any, Dict

from fastapi import Request
from starlette.responses import Response

from .repositories.sys_log_repository import insert_sys_log
from .utils.request_utils import (
    get_authorization_user_id,
    get_browser_and_os,
    get_client_ip,
)


def _resolve_module(path: str) -> str:
    """根据请求路径推断所属模块名称，用于日志列表筛选。"""
    mapping = {
        "/auth/login": "登录",
        "/auth/logout": "登录",
        "/system/user": "用户管理",
        "/system/role": "角色管理",
        "/system/menu": "菜单管理",
        "/system/dept": "部门管理",
        "/system/file": "文件管理",
        "/system/storage": "存储管理",
        "/system/client": "客户端配置",
        "/system/dict": "字典管理",
        "/system/option": "系统配置",
        "/monitor/online": "在线用户",
        "/system/log": "系统日志",
    }
    for prefix, name in mapping.items():
        if path.startswith(prefix):
            return name
    # 未匹配到时归类为“其它”，避免写入 NULL 触发 NOT NULL 约束
    return "其它"


async def sys_log_middleware(request: Request, call_next):
    """
    系统日志中间件：
    - 记录所有业务接口的请求/响应概要信息到 sys_log；
    - 尽量不影响主流程，异常时静默忽略。
    """
    # 跳过文档与静态资源
    path = request.url.path
    if path.startswith("/docs") or path.startswith("/openapi") or path.startswith(
        "/redoc"
    ):
        return await call_next(request)

    start = time.time()
    response: Response = await call_next(request)
    duration_ms = int((time.time() - start) * 1000)

    try:
        method = request.method
        # 只记录有业务含义的接口，OPTIONS 等跳过
        if method in {"OPTIONS"}:
            return response

        # 描述与模块
        description = f"{method} {path}"
        module = _resolve_module(path)

        # 请求信息
        headers_dict: Dict[str, Any] = dict(request.headers)
        ip = get_client_ip(request)
        user_agent = request.headers.get("user-agent", "") or ""
        browser, os = get_browser_and_os(user_agent)

        # 这里只记录查询参数，不读取 Body，避免影响下游读取流
        query_params = dict(request.query_params)
        request_body_str = _safe_json_dump(query_params) if query_params else ""

        # 响应信息
        status_code = int(response.status_code)
        resp_headers_dict: Dict[str, Any] = dict(response.headers)
        response_body_str = ""

        # 状态与错误信息：简单根据 HTTP 状态码判断
        status = 1 if status_code < 400 else 2
        error_msg = ""

        # 操作人
        create_user = get_authorization_user_id(headers_dict)

        insert_sys_log(
            description=description,
            module=module,
            request_url=str(request.url),
            request_method=method,
            request_headers=headers_dict,
            request_body=request_body_str,
            status_code=status_code,
            response_headers=resp_headers_dict,
            response_body=response_body_str,
            time_taken_ms=duration_ms,
            ip=ip,
            address="",
            browser=browser,
            os=os,
            status=status,
            error_msg=error_msg,
            create_user=create_user,
        )
    except Exception:
        # 日志记录失败不影响业务响应
        pass

    return response
