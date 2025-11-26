from __future__ import annotations

from typing import List, Optional, Dict, Any

from pydantic import BaseModel, Field

from ..db import get_db_cursor
from ..id_generator import next_id


def _format_time(dt) -> str:
    """统一时间格式为 YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


class DictResp(BaseModel):
    """字典响应结构。"""

    id: int
    name: str
    code: str
    description: str
    isSystem: bool
    createUserString: str
    createTime: str
    updateUserString: str
    updateTime: str


class DictReq(BaseModel):
    """字典创建/修改请求体。"""

    name: str = Field(..., description="字典名称")
    code: str = Field(..., description="字典编码")
    description: str = Field("", description="描述")


class DictItemResp(BaseModel):
    """字典项响应结构。"""

    id: int
    label: str
    value: str
    color: str
    sort: int
    description: str
    status: int
    dictId: int
    createUserString: str
    createTime: str
    updateUserString: str
    updateTime: str


class DictItemReq(BaseModel):
    """字典项创建/修改请求体。"""

    dictId: int
    label: str
    value: str
    color: str = ""
    sort: int = 999
    description: str = ""
    status: int = 1


class PageResult(BaseModel):
    """分页结果泛型容器。"""

    list: List[DictItemResp]
    total: int


def list_dict_service(description: str | None) -> List[Dict[str, Any]]:
    """查询字典列表服务实现。"""
    desc = (description or "").strip()

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '') AS description,
       COALESCE(d.is_system, FALSE) AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
ORDER BY d.create_time DESC, d.id DESC;
"""
        )
        rows = cur.fetchall()

    result: List[DictResp] = []
    for r in rows:
        item = DictResp(
            id=int(r["id"]),
            name=r["name"],
            code=r["code"],
            description=r["description"],
            isSystem=bool(r["is_system"]),
            createUserString=r["create_user_string"],
            createTime=_format_time(r["create_time"]),
            updateUserString=r["update_user_string"],
            updateTime=_format_time(r["update_time"]),
        )
        if desc and (desc not in item.name) and (desc not in item.description):
            continue
        result.append(item)

    return [d.dict() for d in result]


def list_dict_item_service(
    dict_id_str: str | None,
    page: str | None,
    size: str | None,
    description: str | None,
    status: str | None,
) -> Dict[str, Any]:
    """分页查询字典项服务实现。"""
    try:
        page_num = int(page or "1")
    except ValueError:
        page_num = 1
    try:
        size_num = int(size or "10")
    except ValueError:
        size_num = 10
    if page_num <= 0:
        page_num = 1
    if size_num <= 0:
        size_num = 10

    desc = (description or "").strip()
    status_val = 0
    if status:
        try:
            v = int(status.strip())
            status_val = v
        except ValueError:
            status_val = 0

    base_sql = """
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '') AS color,
       COALESCE(di.sort, 999) AS sort,
       COALESCE(di.description, '') AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
"""
    order_by = "ORDER BY di.sort ASC, di.id ASC;"

    rows: List[dict]
    with get_db_cursor() as cur:
        if dict_id_str:
            try:
                did = int(dict_id_str.strip())
            except ValueError:
                raise ValueError("字典 ID 不正确") from None
            if did <= 0:
                raise ValueError("字典 ID 不正确")
            cur.execute(base_sql + "WHERE di.dict_id = %s " + order_by, (did,))
        else:
            cur.execute(base_sql + order_by)
        rows = cur.fetchall()

    filtered: List[DictItemResp] = []
    for r in rows:
        item = DictItemResp(
            id=int(r["id"]),
            label=r["label"],
            value=r["value"],
            color=r["color"] or "",
            sort=int(r["sort"] or 999),
            description=r["description"] or "",
            status=int(r["status"]),
            dictId=int(r["dict_id"]),
            createUserString=r["create_user_string"],
            createTime=_format_time(r["create_time"]),
            updateUserString=r["update_user_string"],
            updateTime=_format_time(r["update_time"]),
        )
        if desc and (desc not in item.label) and (desc not in item.description):
            continue
        if status_val and item.status != status_val:
            continue
        filtered.append(item)

    total = len(filtered)
    start = min((page_num - 1) * size_num, total)
    end = min(start + size_num, total)
    page_list = filtered[start:end]

    resp = PageResult(list=page_list, total=total)
    return resp.dict()


def get_dict_item_service(item_id: int) -> Optional[Dict[str, Any]]:
    """查询字典项详情服务实现。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '') AS color,
       COALESCE(di.sort, 999) AS sort,
       COALESCE(di.description, '') AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
WHERE di.id = %s
LIMIT 1;
""",
            (item_id,),
        )
        r = cur.fetchone()

    if not r:
        return None

    resp = DictItemResp(
        id=int(r["id"]),
        label=r["label"],
        value=r["value"],
        color=r["color"] or "",
        sort=int(r["sort"] or 999),
        description=r["description"] or "",
        status=int(r["status"]),
        dictId=int(r["dict_id"]),
        createUserString=r["create_user_string"],
        createTime=_format_time(r["create_time"]),
        updateUserString=r["update_user_string"],
        updateTime=_format_time(r["update_time"]),
    )
    return resp.dict()


def create_dict_item_service(body: DictItemReq, current_uid: int) -> None:
    """新增字典项服务实现。"""
    label = (body.label or "").strip()
    value = (body.value or "").strip()
    dict_id = int(body.dictId or 0)
    if not label or not value or dict_id <= 0:
        raise ValueError("标签、值和字典 ID 不能为空")

    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status if body.status and body.status > 0 else 1

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT 1
FROM sys_dict_item
WHERE dict_id = %s AND value = %s
LIMIT 1;
""",
            (dict_id, value),
        )
        exists = cur.fetchone()
        if exists:
            raise ValueError(f"新增失败，字典值 [{value}] 已存在")

        try:
            cur.execute(
                """
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status, dict_id,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s, %s, %s,
    %s, NOW()
);
""",
                (
                    next_id(),
                    label,
                    value,
                    body.color or "",
                    sort,
                    body.description or "",
                    status,
                    dict_id,
                    current_uid,
                ),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("新增字典项失败") from exc


def update_dict_item_service(
    item_id: int,
    body: DictItemReq,
    current_uid: int,
) -> None:
    """修改字典项服务实现。"""
    label = (body.label or "").strip()
    value = (body.value or "").strip()
    if not label or not value:
        raise ValueError("标签和值不能为空")
    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status if body.status and body.status > 0 else 1

    with get_db_cursor() as cur:
        cur.execute(
            "SELECT dict_id FROM sys_dict_item WHERE id = %s LIMIT 1;",
            (item_id,),
        )
        row = cur.fetchone()
        if not row:
            raise ValueError("字典项不存在")
        dict_id = int(row["dict_id"])

        cur.execute(
            """
SELECT 1
FROM sys_dict_item
WHERE dict_id = %s AND value = %s AND id <> %s
LIMIT 1;
""",
            (dict_id, value, item_id),
        )
        exists = cur.fetchone()
        if exists:
            raise ValueError(f"修改失败，字典值 [{value}] 已存在")

        try:
            cur.execute(
                """
UPDATE sys_dict_item
   SET label       = %s,
       value       = %s,
       color       = %s,
       sort        = %s,
       description = %s,
       status      = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
                (
                    label,
                    value,
                    body.color or "",
                    sort,
                    body.description or "",
                    status,
                    current_uid,
                    item_id,
                ),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("修改字典项失败") from exc


def delete_dict_item_service(ids: List[int]) -> None:
    """删除字典项服务实现。"""
    raw_ids = ids or []
    if not raw_ids:
        raise ValueError("ID 列表不能为空")
    valid_ids: List[int] = []
    for raw in raw_ids:
        if raw and raw > 0:
            valid_ids.append(int(raw))
        else:
            raise ValueError("ID 列表不能为空")

    with get_db_cursor() as cur:
        try:
            cur.execute(
                "DELETE FROM sys_dict_item WHERE id = ANY(%s::bigint[]);",
                (valid_ids,),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("删除字典项失败") from exc


def get_dict_service(dict_id: int) -> Optional[Dict[str, Any]]:
    """查询字典详情服务实现。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '') AS description,
       COALESCE(d.is_system, FALSE) AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = %s
LIMIT 1;
""",
            (dict_id,),
        )
        r = cur.fetchone()

    if not r:
        return None

    resp = DictResp(
        id=int(r["id"]),
        name=r["name"],
        code=r["code"],
        description=r["description"],
        isSystem=bool(r["is_system"]),
        createUserString=r["create_user_string"],
        createTime=_format_time(r["create_time"]),
        updateUserString=r["update_user_string"],
        updateTime=_format_time(r["update_time"]),
    )
    return resp.dict()


def create_dict_service(body: DictReq, current_uid: int) -> int:
    """新增字典服务实现。"""
    name = (body.name or "").strip()
    code = (body.code or "").strip()
    if not name or not code:
        raise ValueError("名称和编码不能为空")

    with get_db_cursor() as cur:
        cur.execute("SELECT 1 FROM sys_dict WHERE name = %s LIMIT 1;", (name,))
        if cur.fetchone():
            raise ValueError(f"新增失败，[{name}] 已存在")
        cur.execute("SELECT 1 FROM sys_dict WHERE code = %s LIMIT 1;", (code,))
        if cur.fetchone():
            raise ValueError(f"新增失败，[{code}] 已存在")

        new_id = next_id()
        try:
            cur.execute(
                """
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
VALUES (%s, %s, %s, %s, FALSE, %s, NOW());
""",
                (new_id, name, code, body.description or "", current_uid),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("新增字典失败") from exc

    return new_id


def update_dict_service(
    dict_id: int,
    body: DictReq,
    current_uid: int,
) -> None:
    """修改字典服务实现。"""
    name = (body.name or "").strip()
    if not name:
        raise ValueError("名称不能为空")

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
UPDATE sys_dict
   SET name = %s,
       description = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id = %s;
""",
                (name, body.description or "", current_uid, dict_id),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("修改字典失败") from exc


def delete_dict_service(ids: List[int]) -> None:
    """删除字典服务实现。"""
    raw_ids = ids or []
    if not raw_ids:
        raise ValueError("ID 列表不能为空")

    valid_ids: List[int] = []
    for raw in raw_ids:
        if raw and raw > 0:
            valid_ids.append(int(raw))
        else:
            raise ValueError("ID 列表不能为空")

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT name, COALESCE(is_system, FALSE) AS is_system
FROM sys_dict
WHERE id = ANY(%s::bigint[]);
""",
            (valid_ids,),
        )
        rows = cur.fetchall()
        for r in rows:
            if r["is_system"]:
                raise ValueError(
                    f"所选字典 [{r['name']}] 是系统内置字典，不允许删除",
                )

        for did in valid_ids:
            cur.execute("DELETE FROM sys_dict_item WHERE dict_id = %s;", (did,))
            cur.execute("DELETE FROM sys_dict WHERE id = %s;", (did,))

