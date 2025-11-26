from __future__ import annotations

from typing import Any, Dict, List, Optional

import json
import uuid
from pydantic import BaseModel, Field

from ..db import get_db_cursor
from ..id_generator import next_id


def _format_time(dt) -> str:
    """统一时间格式化：YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def _parse_auth_type(raw: Any) -> List[str]:
    """解析数据库中的 auth_type 字段为字符串数组。"""
    if raw is None:
        return []
    if isinstance(raw, list):
        return [str(x) for x in raw]
    if isinstance(raw, str):
        try:
            data = json.loads(raw)
            if isinstance(data, list):
                return [str(x) for x in data]
        except Exception:
            return []
    return []


class ClientReq(BaseModel):
    """客户端创建或修改请求体，字段与前端表单对齐。"""

    clientType: str = Field(..., description="客户端类型")
    authType: List[str] = Field(default_factory=list, description="认证类型列表")
    activeTimeout: int = Field(1800, description="Token 最低活跃频率（秒）")
    timeout: int = Field(2592000, description="Token 有效期（秒）")
    status: int = Field(1, description="状态（1 启用，2 禁用）")


def list_client_service(
    page: int,
    size: int,
    client_type: str | None,
    status: str | None,
    auth_type: List[str] | None,
    sort: List[str] | None,
) -> Dict[str, Any]:
    """分页查询客户端列表服务实现。"""
    where: List[str] = ["1=1"]
    params: List[Any] = []

    ct = (client_type or "").strip()
    if ct:
        where.append("c.client_type = %s")
        params.append(ct)

    st = (status or "").strip()
    if st:
        try:
            st_int = int(st)
            if st_int > 0:
                where.append("c.status = %s")
                params.append(st_int)
        except Exception:
            pass

    if auth_type:
        where.append("c.auth_type::jsonb ?| %s")
        params.append(auth_type)

    where_sql = " WHERE " + " AND ".join(where)

    order_by = "ORDER BY c.id DESC"
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
            "id": "c.id",
            "createTime": "c.create_time",
        }
        col = column_map.get(field)
        if col:
            direction_sql = "DESC" if direction.lower() == "desc" else "ASC"
            order_by = f"ORDER BY {col} {direction_sql}"

    with get_db_cursor() as cur:
        cur.execute(f"SELECT COUNT(*) AS cnt FROM sys_client AS c{where_sql}", params)
        row = cur.fetchone()
        total = int(row["cnt"]) if row and row["cnt"] is not None else 0
        if total == 0:
            return {"list": [], "total": 0}

        offset = (page - 1) * size
        list_params = params + [size, offset]
        cur.execute(
            f"""
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, '') AS create_user_name,
       COALESCE(uu.nickname, '') AS update_user_name
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
{where_sql}
{order_by}
LIMIT %s OFFSET %s;
""",
            list_params,
        )
        rows = cur.fetchall()

    items: List[Dict[str, Any]] = []
    for r in rows:
        auth_list = _parse_auth_type(r.get("auth_type"))
        item = {
            "id": int(r["id"]),
            "clientId": r["client_id"],
            "clientType": r["client_type"],
            "authType": auth_list,
            "activeTimeout": int(r["active_timeout"]),
            "timeout": int(r["timeout"]),
            "status": int(r["status"]),
            "createUser": int(r["create_user"]) if r.get("create_user") is not None else 0,
            "createTime": _format_time(r["create_time"]),
            "updateUser": int(r["update_user"]) if r.get("update_user") is not None else 0,
            "updateTime": _format_time(r["update_time"]),
            "createUserString": r["create_user_name"],
            "updateUserString": r["update_user_name"],
        }
        items.append(item)

    return {"list": items, "total": total}


def get_client_detail_service(client_id: int) -> Optional[Dict[str, Any]]:
    """查询单个客户端详情服务实现。"""
    sql = """
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, '') AS create_user_name,
       COALESCE(uu.nickname, '') AS update_user_name
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
WHERE c.id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (client_id,))
        r = cur.fetchone()

    if not r:
        return None

    auth_list = _parse_auth_type(r.get("auth_type"))
    data = {
        "id": int(r["id"]),
        "clientId": r["client_id"],
        "clientType": r["client_type"],
        "authType": auth_list,
        "activeTimeout": int(r["active_timeout"]),
        "timeout": int(r["timeout"]),
        "status": int(r["status"]),
        "createUser": int(r["create_user"]) if r.get("create_user") is not None else 0,
        "createTime": _format_time(r["create_time"]),
        "updateUser": int(r["update_user"]) if r.get("update_user") is not None else 0,
        "updateTime": _format_time(r["update_time"]),
        "createUserString": r["create_user_name"],
        "updateUserString": r["update_user_name"],
    }
    return data


def create_client_service(req: ClientReq, current_uid: int) -> Dict[str, Any]:
    """新增客户端服务实现。"""
    client_type = (req.clientType or "").strip()
    if not client_type:
        raise ValueError("客户端类型不能为空")
    if not req.authType:
        raise ValueError("认证类型不能为空")

    active_timeout = req.activeTimeout if req.activeTimeout is not None else -1
    timeout = req.timeout if req.timeout is not None else 2592000
    status = req.status or 1

    client_id = uuid.uuid4().hex
    new_id = next_id()

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
INSERT INTO sys_client (
    id, client_id, client_type, auth_type,
    active_timeout, timeout, status,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s,
    %s, %s, %s,
    %s, NOW()
);
""",
                (
                    new_id,
                    client_id,
                    client_type,
                    json.dumps(req.authType, ensure_ascii=False),
                    active_timeout,
                    timeout,
                    status,
                    current_uid,
                ),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("保存客户端失败") from exc

    return {"id": new_id, "clientId": client_id}


def update_client_service(
    client_id: int,
    req: ClientReq,
    current_uid: int,
) -> None:
    """修改客户端服务实现。"""
    client_type = (req.clientType or "").strip()
    if not client_type:
        raise ValueError("客户端类型不能为空")
    if not req.authType:
        raise ValueError("认证类型不能为空")

    active_timeout = req.activeTimeout if req.activeTimeout is not None else -1
    timeout = req.timeout if req.timeout is not None else 2592000
    status = req.status or 1

    with get_db_cursor() as cur:
        cur.execute(
            """
UPDATE sys_client
   SET client_type    = %s,
       auth_type      = %s,
       active_timeout = %s,
       timeout        = %s,
       status         = %s,
       update_user    = %s,
       update_time    = NOW()
 WHERE id             = %s;
""",
            (
                client_type,
                json.dumps(req.authType, ensure_ascii=False),
                active_timeout,
                timeout,
                status,
                current_uid,
                client_id,
            ),
        )
        if cur.rowcount <= 0:
            raise ValueError("客户端不存在")


def delete_client_service(ids: List[int]) -> None:
    """批量删除客户端服务实现。"""
    ids = [i for i in (ids or []) if i > 0]
    if not ids:
        raise ValueError("ID 列表不能为空")

    with get_db_cursor() as cur:
        cur.execute(
            """
DELETE FROM sys_client
 WHERE id = ANY(%s::bigint[]);
""",
            (ids,),
        )

