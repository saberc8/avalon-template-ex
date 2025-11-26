from __future__ import annotations

from typing import Any, Dict, List

from pydantic import BaseModel, Field

from ..repositories.system_menu_repository import (
    delete_menus_with_ids,
    get_menu_row,
    list_menu_parent_relations,
    list_menus_for_tree,
    insert_menu_row,
    update_menu_row,
)


def _format_time(dt) -> str:
    """统一时间格式为 YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


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


def _build_menu_resp(row: Dict[str, Any]) -> MenuResp:
    """根据数据库行构造 MenuResp 对象。"""
    return MenuResp(
        id=int(row["id"]),
        title=row["title"],
        parentId=int(row["parent_id"]),
        type=int(row["type"]),
        path=row["path"] or "",
        name=row["name"] or "",
        component=row["component"] or "",
        redirect=row["redirect"] or "",
        icon=row["icon"] or "",
        isExternal=bool(row["is_external"]),
        isCache=bool(row["is_cache"]),
        isHidden=bool(row["is_hidden"]),
        permission=row["permission"] or "",
        sort=int(row["sort"]),
        status=int(row["status"]),
        createUserString=row["create_user_string"],
        createTime=_format_time(row["create_time"]),
        updateUserString=row["update_user_string"],
        updateTime=_format_time(row["update_time"]),
        children=[],
    )


def list_menu_tree_service() -> List[Dict[str, Any]]:
    """查询菜单树结构。"""
    rows = list_menus_for_tree()
    flat: List[MenuResp] = []
    for r in rows:
        flat.append(_build_menu_resp(r))

    if not flat:
        return []

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

    return [m.dict() for m in roots]


def get_menu_service(menu_id: int) -> Dict[str, Any] | None:
    """查询单个菜单详情。"""
    row = get_menu_row(menu_id)
    if not row:
        return None
    resp = _build_menu_resp(row)
    return resp.dict()


def _normalize_menu_fields(body: MenuReq) -> Dict[str, Any]:
    """校验并规范化菜单字段，返回可用于持久化的字段字典。"""
    title = (body.title or "").strip()
    if not title:
        raise ValueError("菜单标题不能为空")

    is_external = bool(body.isExternal) if body.isExternal is not None else False
    is_cache = bool(body.isCache) if body.isCache is not None else False
    is_hidden = bool(body.isHidden) if body.isHidden is not None else False

    path = (body.path or "").strip()
    name = (body.name or "").strip()
    component = (body.component or "").strip()

    if is_external:
        if not (path.startswith("http://") or path.startswith("https://")):
            raise ValueError("路由地址格式不正确，请以 http:// 或 https:// 开头")
    else:
        if path.startswith("http://") or path.startswith("https://"):
            raise ValueError("路由地址格式不正确")
        if path and not path.startswith("/"):
            path = "/" + path
        if name.startswith("/"):
            name = name[1:]
        if component.startswith("/"):
            component = component[1:]

    sort = body.sort if body.sort and body.sort > 0 else 999
    status = body.status or 1

    return {
        "title": title,
        "parent_id": body.parentId or 0,
        "type": body.type or 1,
        "path": path,
        "name": name,
        "component": component,
        "redirect": body.redirect or "",
        "icon": body.icon or "",
        "is_external": is_external,
        "is_cache": is_cache,
        "is_hidden": is_hidden,
        "permission": body.permission or "",
        "sort": sort,
        "status": status,
    }


def create_menu_service(body: MenuReq, current_uid: int) -> int:
    """新增菜单服务实现。"""
    fields = _normalize_menu_fields(body)
    new_id = insert_menu_row(
        title=fields["title"],
        parent_id=fields["parent_id"],
        type_=fields["type"],
        path=fields["path"],
        name=fields["name"],
        component=fields["component"],
        redirect=fields["redirect"],
        icon=fields["icon"],
        is_external=fields["is_external"],
        is_cache=fields["is_cache"],
        is_hidden=fields["is_hidden"],
        permission=fields["permission"],
        sort=fields["sort"],
        status=fields["status"],
        create_user=current_uid,
    )
    return new_id


def update_menu_service(menu_id: int, body: MenuReq, current_uid: int) -> None:
    """修改菜单服务实现。"""
    fields = _normalize_menu_fields(body)
    update_menu_row(
        menu_id=menu_id,
        title=fields["title"],
        parent_id=fields["parent_id"],
        type_=fields["type"],
        path=fields["path"],
        name=fields["name"],
        component=fields["component"],
        redirect=fields["redirect"],
        icon=fields["icon"],
        is_external=fields["is_external"],
        is_cache=fields["is_cache"],
        is_hidden=fields["is_hidden"],
        permission=fields["permission"],
        sort=fields["sort"],
        status=fields["status"],
        update_user=current_uid,
    )


def delete_menu_service(ids: List[int]) -> None:
    """批量删除菜单（含子节点）。"""
    ids = [i for i in (ids or []) if i and i > 0]
    if not ids:
        raise ValueError("ID 列表不能为空")

    rows = list_menu_parent_relations()

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
        return

    all_ids = sorted(seen)
    delete_menus_with_ids(all_ids)

