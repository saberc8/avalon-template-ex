from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Path, Query

from ..api_response import fail, ok
from ..db import get_db_cursor

router = APIRouter()


def _format_time(dt: datetime | None) -> str:
    """时间格式化为前端期望的字符串。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def _parse_datetime(value: str | None) -> Optional[datetime]:
    """解析前端传递的日期时间字符串，兼容 YYYY-MM-DD HH:MM:SS。"""
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


@router.get("/system/log")
def list_log(
    page: int = Query(1, ge=1),
    size: int = Query(10, ge=1),
    description: str | None = Query(default=None),
    module: str | None = Query(default=None),
    ip: str | None = Query(default=None),
    createUserString: str | None = Query(default=None),
    createTime: Optional[List[str]] = Query(default=None),
    status: int | None = Query(default=None),
    sort: Optional[List[str]] = Query(default=None),
):
    """
    分页查询系统日志：GET /system/log。

    对齐前端 LogResp PageRes 结构：
    - 请求参数：description/module/ip/createUserString/createTime[2]/status/sort/page/size
    - 返回：{"list": [LogResp...], "total": number}
    """
    where: List[str] = ["1=1"]
    params: List[Any] = []

    desc_kw = (description or "").strip()
    if desc_kw:
        where.append("description ILIKE %s")
        params.append(f"%{desc_kw}%")

    module_kw = (module or "").strip()
    if module_kw:
        where.append("module ILIKE %s")
        params.append(f"%{module_kw}%")

    ip_kw = (ip or "").strip()
    if ip_kw:
        where.append("(ip ILIKE %s OR address ILIKE %s)")
        like = f"%{ip_kw}%"
        params.extend([like, like])

    user_kw = (createUserString or "").strip()
    if user_kw:
        where.append("create_user_string ILIKE %s")
        params.append(f"%{user_kw}%")

    # 时间范围
    if createTime and len(createTime) == 2:
        start = _parse_datetime(createTime[0])
        end = _parse_datetime(createTime[1])
        if start:
            where.append("create_time >= %s")
            params.append(start)
        if end:
            where.append("create_time <= %s")
            params.append(end)

    if status and status in (1, 2):
        where.append("status = %s")
        params.append(status)

    where_sql = " WHERE " + " AND ".join(where)

    # 排序：默认 create_time DESC
    order_by = "ORDER BY create_time DESC, id DESC"
    sort_values: List[str] = []
    for s in sort or []:
        if not s:
            continue
        for part in s.split(","):
            val = part.strip()
            if val:
                sort_values.append(val)
    if sort_values:
        field = sort_values[0]
        direction = sort_values[1] if len(sort_values) > 1 else "desc"
        column_map = {
            "id": "id",
            "createTime": "create_time",
        }
        col = column_map.get(field)
        if col:
            direction_sql = "DESC" if direction.lower() == "desc" else "ASC"
            order_by = f"ORDER BY {col} {direction_sql}"

    with get_db_cursor() as cur:
        # 计算总数
        cur.execute(f"SELECT COUNT(*) AS cnt FROM sys_log{where_sql}", params)
        row = cur.fetchone()
        total = int(row["cnt"]) if row and row["cnt"] is not None else 0
        if total == 0:
            return ok({"list": [], "total": 0})

        offset = (page - 1) * size
        cur.execute(
            f"""
SELECT id,
       description,
       module,
       time_taken,
       ip,
       address,
       browser,
       os,
       status,
       COALESCE(error_msg, '') AS error_msg,
       COALESCE(create_user, 0) AS create_user,
       create_time
FROM sys_log
{where_sql}
{order_by}
LIMIT %s OFFSET %s;
""",
            params + [size, offset],
        )
        rows = cur.fetchall()

    items: List[Dict[str, Any]] = []
    for r in rows:
        item = {
            "id": int(r["id"]),
            "description": r["description"],
            "module": r["module"],
            "timeTaken": int(r["time_taken"]),
            "ip": r["ip"] or "",
            "address": r["address"] or "",
            "browser": r["browser"] or "",
            "os": r["os"] or "",
            "status": int(r["status"]),
            "errorMsg": r["error_msg"],
            # 这里暂不从用户表反查昵称，直接用 ID 字符串，保证前端字段存在
            "createUserString": str(r["create_user"]) if r["create_user"] else "",
            "createTime": _format_time(r["create_time"]),
        }
        items.append(item)

    return ok({"list": items, "total": total})


@router.get("/system/log/{log_id}")
def get_log_detail(log_id: int = Path(..., ge=1)):
    """查询日志详情：GET /system/log/{id}。"""
    sql = """
SELECT id,
       trace_id,
       description,
       module,
       request_url,
       request_method,
       COALESCE(request_headers, '')  AS request_headers,
       COALESCE(request_body, '')     AS request_body,
       status_code,
       COALESCE(response_headers, '') AS response_headers,
       COALESCE(response_body, '')    AS response_body,
       time_taken,
       ip,
       address,
       browser,
       os,
       status,
       COALESCE(error_msg, '')        AS error_msg,
       COALESCE(create_user, 0)       AS create_user,
       create_time
FROM sys_log
WHERE id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (log_id,))
        r = cur.fetchone()

    if not r:
        return fail("404", "日志不存在")

    data = {
        "id": int(r["id"]),
        "traceId": r["trace_id"] or "",
        "description": r["description"],
        "module": r["module"],
        "requestUrl": r["request_url"],
        "requestMethod": r["request_method"],
        "requestHeaders": r["request_headers"],
        "requestBody": r["request_body"],
        "statusCode": int(r["status_code"]),
        "responseHeaders": r["response_headers"],
        "responseBody": r["response_body"],
        "timeTaken": int(r["time_taken"]),
        "ip": r["ip"] or "",
        "address": r["address"] or "",
        "browser": r["browser"] or "",
        "os": r["os"] or "",
        "status": int(r["status"]),
        "errorMsg": r["error_msg"],
        "createUserString": str(r["create_user"]) if r["create_user"] else "",
        "createTime": _format_time(r["create_time"]),
    }
    return ok(data)

