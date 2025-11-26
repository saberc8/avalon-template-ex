from __future__ import annotations

from typing import List, Optional

from fastapi import APIRouter, Body, Header, Path, Query
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..config import get_settings
from ..db import get_db_cursor
from ..id_generator import next_id
from ..security.jwt_token import TokenService

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _current_user_id(authorization: str | None) -> Optional[int]:
    """从 Authorization 头解析当前登录用户 ID，未授权返回 None。"""
    if not authorization:
        return None
    token_svc = _get_token_service()
    claims = token_svc.parse(authorization)
    if not claims:
        return None
    return claims.user_id


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


class DictItemListQuery(BaseModel):
    """字典项分页查询条件。"""

    dictId: str | None = None
    page: str | None = None
    size: str | None = None
    description: str | None = None
    status: str | None = None


class IdsRequest(BaseModel):
    """批量 ID 请求体。"""

    ids: List[int]


class PageResult(BaseModel):
    """分页结果泛型容器。"""

    list: List[DictItemResp]
    total: int


def _format_time(dt) -> str:
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


@router.get("/system/dict/list")
def list_dict(description: str | None = Query(default=None)):
    """查询字典列表：GET /system/dict/list。"""
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

    return ok([d.dict() for d in result])


@router.get("/system/dict/item")
def list_dict_item(
    dictId: str | None = Query(default=None),
    page: str | None = Query(default=None),
    size: str | None = Query(default=None),
    description: str | None = Query(default=None),
    status: str | None = Query(default=None),
):
    """分页查询字典项：GET /system/dict/item。"""
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

    desc = (description or "").trim() if description else ""
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
        if dictId:
            try:
                did = int(dictId.strip())
            except ValueError:
                return fail("400", "字典 ID 不正确")
            if did <= 0:
                return fail("400", "字典 ID 不正确")
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
    return ok(resp.dict())


@router.get("/system/dict/item/{item_id}")
def get_dict_item(item_id: int = Path(..., alias="item_id")):
    """查询字典项详情：GET /system/dict/item/{id}。"""
    if item_id <= 0:
        return fail("400", "ID 参数不正确")

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
        return fail("404", "字典项不存在")

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
    return ok(resp.dict())


@router.post("/system/dict/item")
def create_dict_item(
    body: DictItemReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增字典项：POST /system/dict/item。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    label = (body.label or "").strip()
    value = (body.value or "").strip()
    dict_id = int(body.dictId or 0)
    if not label or not value or dict_id <= 0:
        return fail("400", "标签、值和字典 ID 不能为空")

    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status if body.status and body.status > 0 else 1

    with get_db_cursor() as cur:
        # 校验同一字典下 value 是否重复
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
            return fail("400", f"新增失败，字典值 [{value}] 已存在")

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
        except Exception:
            return fail("500", "新增字典项失败")

    return ok(True)


@router.put("/system/dict/item/{item_id}")
def update_dict_item(
    item_id: int = Path(..., alias="item_id"),
    body: DictItemReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改字典项：PUT /system/dict/item/{id}。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if item_id <= 0:
        return fail("400", "ID 参数不正确")

    label = (body.label or "").strip()
    value = (body.value or "").strip()
    if not label or not value:
        return fail("400", "标签和值不能为空")
    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status if body.status and body.status > 0 else 1

    with get_db_cursor() as cur:
        # 查询当前字典项所属字典
        cur.execute(
            "SELECT dict_id FROM sys_dict_item WHERE id = %s LIMIT 1;",
            (item_id,),
        )
        row = cur.fetchone()
        if not row:
            return fail("404", "字典项不存在")
        dict_id = int(row["dict_id"])

        # 校验同一字典下 value 是否重复（排除自身）
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
            return fail("400", f"修改失败，字典值 [{value}] 已存在")

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
        except Exception:
            return fail("500", "修改字典项失败")

    return ok(True)


@router.delete("/system/dict/item")
def delete_dict_item(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除字典项：DELETE /system/dict/item。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    raw_ids = body.ids or []
    if not raw_ids:
        return fail("400", "ID 列表不能为空")
    ids: List[int] = []
    for raw in raw_ids:
        if raw and raw > 0:
            ids.append(int(raw))
        else:
            return fail("400", "ID 列表不能为空")

    with get_db_cursor() as cur:
        try:
            cur.execute(
                "DELETE FROM sys_dict_item WHERE id = ANY(%s::bigint[]);",
                (ids,),
            )
        except Exception:
            return fail("500", "删除字典项失败")

    return ok(True)


@router.get("/system/dict/{dict_id}")
def get_dict(dict_id: int = Path(..., alias="dict_id")):
    """查询字典详情：GET /system/dict/{id}。"""
    if dict_id <= 0:
        return fail("400", "ID 参数不正确")

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
        return fail("404", "字典不存在")

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
    return ok(resp.dict())


@router.post("/system/dict")
def create_dict(
    body: DictReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增字典：POST /system/dict。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    name = (body.name or "").strip()
    code = (body.code or "").strip()
    if not name or not code:
        return fail("400", "名称和编码不能为空")

    with get_db_cursor() as cur:
        # 名称唯一
        cur.execute("SELECT 1 FROM sys_dict WHERE name = %s LIMIT 1;", (name,))
        if cur.fetchone():
            return fail("400", f"新增失败，[{name}] 已存在")
        # 编码唯一
        cur.execute("SELECT 1 FROM sys_dict WHERE code = %s LIMIT 1;", (code,))
        if cur.fetchone():
            return fail("400", f"新增失败，[{code}] 已存在")

        new_id = next_id()
        try:
            cur.execute(
                """
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
VALUES (%s, %s, %s, %s, FALSE, %s, NOW());
""",
                (new_id, name, code, body.description or "", current_uid),
            )
        except Exception:
            return fail("500", "新增字典失败")

    return ok({"id": new_id})


@router.put("/system/dict/{dict_id}")
def update_dict(
    dict_id: int = Path(..., alias="dict_id"),
    body: DictReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改字典：PUT /system/dict/{id}。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if dict_id <= 0:
        return fail("400", "ID 参数不正确")

    name = (body.name or "").strip()
    if not name:
        return fail("400", "名称不能为空")

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
        except Exception:
            return fail("500", "修改字典失败")

    return ok(True)


@router.delete("/system/dict")
def delete_dict(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除字典：DELETE /system/dict。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    raw_ids = body.ids or []
    if not raw_ids:
        return fail("400", "ID 列表不能为空")

    ids: List[int] = []
    for raw in raw_ids:
        if raw and raw > 0:
            ids.append(int(raw))
        else:
            return fail("400", "ID 列表不能为空")

    with get_db_cursor() as cur:
        # 系统内置校验
        cur.execute(
            """
SELECT name, COALESCE(is_system, FALSE) AS is_system
FROM sys_dict
WHERE id = ANY(%s::bigint[]);
""",
            (ids,),
        )
        rows = cur.fetchall()
        for r in rows:
            if r["is_system"]:
                return fail(
                    "400",
                    f"所选字典 [{r['name']}] 是系统内置字典，不允许删除",
                )

        # 删除字典与字典项
        for did in ids:
            cur.execute("DELETE FROM sys_dict_item WHERE dict_id = %s;", (did,))
            cur.execute("DELETE FROM sys_dict WHERE id = %s;", (did,))

    return ok(True)


@router.delete("/system/dict/cache/{code}")
def clear_dict_cache(code: str = Path(...)):
    """清理字典缓存（当前无缓存逻辑，直接返回成功）。"""
    return ok(True)
