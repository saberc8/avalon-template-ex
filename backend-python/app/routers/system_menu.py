from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Body, Header, Path
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..db import get_db_cursor
from ..id_generator import next_id
from ..config import get_settings
from ..security.jwt_token import TokenService

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例（与 auth 模块一致）。"""
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


class MenuResp(BaseModel):
    """菜单响应结构，字段与前端 MenuResp 保持一致。"""

    id: int
    title: str
    parentId: int
    type: int
    path: str
    name: str
    component: str
    redirect: str
    icon: str
    isExternal: bool
    isCache: bool
    isHidden: bool
    permission: str
    sort: int
    status: int
    createUserString: str
    createTime: str
    updateUserString: str
    updateTime: str
    children: List["MenuResp"] = []


MenuResp.update_forward_refs()


class MenuReq(BaseModel):
    """创建/修改菜单请求体，字段与前端 MenuReq 对齐。"""

    type: int = Field(1, description="菜单类型：1=目录，2=菜单，3=按钮")
    icon: str = Field("", description="图标")
    title: str = Field(..., description="菜单标题")
    sort: int = Field(999, description="排序")
    permission: str = Field("", description="按钮权限标识")
    path: str = Field("", description="路由地址")
    name: str = Field("", description="路由名称")
    component: str = Field("", description="前端组件路径")
    redirect: str = Field("", description="重定向地址")
    isExternal: bool | None = Field(None, description="是否外链")
    isCache: bool | None = Field(None, description="是否缓存")
    isHidden: bool | None = Field(None, description="是否隐藏")
    parentId: int = Field(0, description="父级菜单 ID")
    status: int = Field(1, description="状态 1=启用 2=禁用")


class IdsRequest(BaseModel):
    """批量 ID 请求体。"""

    ids: List[int]


def _format_time(dt) -> str:
    """统一时间格式为 YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


@router.get("/system/menu/tree")
def list_menu_tree():
    """查询菜单树：GET /system/menu/tree。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
ORDER BY m.sort ASC, m.id ASC;
"""
        )
        rows = cur.fetchall()

    flat: List[MenuResp] = []
    for r in rows:
        flat.append(
            MenuResp(
                id=int(r["id"]),
                title=r["title"],
                parentId=int(r["parent_id"]),
                type=int(r["type"]),
                path=r["path"] or "",
                name=r["name"] or "",
                component=r["component"] or "",
                redirect=r["redirect"] or "",
                icon=r["icon"] or "",
                isExternal=bool(r["is_external"]),
                isCache=bool(r["is_cache"]),
                isHidden=bool(r["is_hidden"]),
                permission=r["permission"] or "",
                sort=int(r["sort"]),
                status=int(r["status"]),
                createUserString=r["create_user_string"],
                createTime=_format_time(r["create_time"]),
                updateUserString=r["update_user_string"],
                updateTime=_format_time(r["update_time"]),
                children=[],
            )
        )

    if not flat:
        return ok([])

    node_map: Dict[int, MenuResp] = {m.id: m for m in flat}
    roots: List[MenuResp] = []
    for item in flat:
        if item.parentId == 0:
            roots.append(item)
            continue
        parent = node_map.get(item.parentId)
        if not parent:
            roots.append(item)
            continue
        parent.children.append(item)

    return ok([m.dict() for m in roots])


@router.get("/system/menu/{menu_id}")
def get_menu(menu_id: int = Path(..., alias="menu_id")):
    """查询菜单详情：GET /system/menu/{id}。"""
    if menu_id <= 0:
        return fail("400", "ID 参数不正确")

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
WHERE m.id = %s
LIMIT 1;
""",
            (menu_id,),
        )
        r = cur.fetchone()

    if not r:
        return fail("404", "菜单不存在")

    item = MenuResp(
        id=int(r["id"]),
        title=r["title"],
        parentId=int(r["parent_id"]),
        type=int(r["type"]),
        path=r["path"] or "",
        name=r["name"] or "",
        component=r["component"] or "",
        redirect=r["redirect"] or "",
        icon=r["icon"] or "",
        isExternal=bool(r["is_external"]),
        isCache=bool(r["is_cache"]),
        isHidden=bool(r["is_hidden"]),
        permission=r["permission"] or "",
        sort=int(r["sort"]),
        status=int(r["status"]),
        createUserString=r["create_user_string"],
        createTime=_format_time(r["create_time"]),
        updateUserString=r["update_user_string"],
        updateTime=_format_time(r["update_time"]),
        children=[],
    )
    return ok(item.dict())


@router.post("/system/menu")
def create_menu(
    body: MenuReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增菜单：POST /system/menu。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    title = (body.title or "").strip()
    if not title:
        return fail("400", "菜单标题不能为空")

    is_external = bool(body.isExternal) if body.isExternal is not None else False
    is_cache = bool(body.isCache) if body.isCache is not None else False
    is_hidden = bool(body.isHidden) if body.isHidden is not None else False

    path = (body.path or "").strip()
    name = (body.name or "").strip()
    component = (body.component or "").strip()

    if is_external:
        if not (path.startswith("http://") or path.startswith("https://")):
            return fail("400", "路由地址格式不正确，请以 http:// 或 https:// 开头")
    else:
        if path.startswith("http://") or path.startswith("https://"):
            return fail("400", "路由地址格式不正确")
        if path and not path.startswith("/"):
            path = "/" + path
        if name.startswith("/"):
            name = name[1:]
        if component.startswith("/"):
            component = component[1:]

    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status or 1

    new_id = next_id()
    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect,
    icon, is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s, %s, %s,
    %s, %s, %s, %s, %s, %s, %s,
    %s, NOW()
);
""",
                (
                    new_id,
                    title,
                    body.parentId or 0,
                    body.type or 1,
                    path,
                    name,
                    component,
                    body.redirect or "",
                    body.icon or "",
                    is_external,
                    is_cache,
                    is_hidden,
                    body.permission or "",
                    sort,
                    status,
                    current_uid,
                ),
            )
        except Exception:
            return fail("500", "新增菜单失败")

    return ok({"id": new_id})


@router.put("/system/menu/{menu_id}")
def update_menu(
    menu_id: int = Path(..., alias="menu_id"),
    body: MenuReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改菜单：PUT /system/menu/{id}。"""
    if menu_id <= 0:
        return fail("400", "ID 参数不正确")

    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    title = (body.title or "").strip()
    if not title:
        return fail("400", "菜单标题不能为空")

    is_external = bool(body.isExternal) if body.isExternal is not None else False
    is_cache = bool(body.isCache) if body.isCache is not None else False
    is_hidden = bool(body.isHidden) if body.isHidden is not None else False

    path = (body.path or "").strip()
    name = (body.name or "").strip()
    component = (body.component or "").strip()

    if is_external:
        if not (path.startswith("http://") or path.startswith("https://")):
            return fail("400", "路由地址格式不正确，请以 http:// 或 https:// 开头")
    else:
        if path.startswith("http://") or path.startswith("https://"):
            return fail("400", "路由地址格式不正确")
        if path and not path.startswith("/"):
            path = "/" + path
        if name.startswith("/"):
            name = name[1:]
        if component.startswith("/"):
            component = component[1:]

    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status or 1

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
UPDATE sys_menu
   SET title       = %s,
       parent_id   = %s,
       type        = %s,
       path        = %s,
       name        = %s,
       component   = %s,
       redirect    = %s,
       icon        = %s,
       is_external = %s,
       is_cache    = %s,
       is_hidden   = %s,
       permission  = %s,
       sort        = %s,
       status      = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
                (
                    title,
                    body.parentId or 0,
                    body.type or 1,
                    path,
                    name,
                    component,
                    body.redirect or "",
                    body.icon or "",
                    is_external,
                    is_cache,
                    is_hidden,
                    body.permission or "",
                    sort,
                    status,
                    current_uid,
                    menu_id,
                ),
            )
        except Exception:
            return fail("500", "修改菜单失败")

    return ok(True)


@router.delete("/system/menu")
def delete_menu(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """批量删除菜单（含子节点）：DELETE /system/menu。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i and i > 0]
    if not ids:
        return fail("400", "ID 列表不能为空")

    with get_db_cursor() as cur:
        cur.execute("SELECT id, parent_id FROM sys_menu;")
        rows = cur.fetchall()

    children_of: Dict[int, List[int]] = {}
    for r in rows:
        pid = int(r["parent_id"])
        mid = int(r["id"])
        children_of.setdefault(pid, []).append(mid)

    seen: set[int] = set()

    def collect(mid: int) -> None:
        if mid in seen:
            return
        seen.add(mid)
        for c in children_of.get(mid, []):
            collect(c)

    for mid in ids:
        collect(mid)

    if not seen:
        return ok(True)

    all_ids = sorted(seen)

    with get_db_cursor() as cur:
        try:
            cur.execute(
                "DELETE FROM sys_role_menu WHERE menu_id = ANY(%s::bigint[]);",
                (all_ids,),
            )
            cur.execute(
                "DELETE FROM sys_menu WHERE id = ANY(%s::bigint[]);",
                (all_ids,),
            )
        except Exception:
            return fail("500", "删除菜单失败")

    return ok(True)


@router.delete("/system/menu/cache")
def clear_menu_cache():
    """
    清除菜单缓存：DELETE /system/menu/cache。

    当前 Python 版本未实现缓存，保持与 Node/Go 版本一致，直接返回成功。
    """
    return ok(True)

