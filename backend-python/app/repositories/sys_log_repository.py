from __future__ import annotations

from typing import Any, Dict, Optional

from ..db import get_db_cursor
from ..id_generator import next_id


def _safe_json_dump(data: Any) -> str:
    """安全地序列化为 JSON 字符串，失败时返回空串。"""
    import json

    try:
        return json.dumps(data, ensure_ascii=False)
    except Exception:  # noqa: BLE001
        return ""


def insert_sys_log(
    description: str,
    module: str,
    request_url: str,
    request_method: str,
    request_headers: Dict[str, Any],
    request_body: str,
    status_code: int,
    response_headers: Dict[str, Any],
    response_body: str,
    time_taken_ms: int,
    ip: str,
    address: str,
    browser: str,
    os: str,
    status: int,
    error_msg: str,
    create_user: Optional[int],
) -> None:
    """写入一条系统日志到 sys_log 表。"""
    log_id = next_id()
    headers_json = _safe_json_dump(request_headers)
    resp_headers_json = _safe_json_dump(response_headers)

    safe_module = module or "其它"

    with get_db_cursor() as cur:
        cur.execute(
            """
INSERT INTO sys_log (
    id, trace_id, description, module,
    request_url, request_method, request_headers, request_body,
    status_code, response_headers, response_body,
    time_taken, ip, address, browser, os,
    status, error_msg, create_user, create_time
) VALUES (
    %s, NULL, %s, %s,
    %s, %s, %s, %s,
    %s, %s, %s,
    %s, %s, %s, %s, %s,
    %s, %s, %s, NOW()
);
""",
            (
                log_id,
                description,
                safe_module,
                request_url,
                request_method,
                headers_json,
                request_body,
                status_code,
                resp_headers_json,
                response_body,
                time_taken_ms,
                ip,
                address,
                browser,
                os,
                status,
                error_msg or "",
                create_user,
            ),
        )

