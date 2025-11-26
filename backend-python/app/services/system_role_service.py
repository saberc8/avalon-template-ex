from __future__ import annotations

from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field

from ..repositories.system_role_repository import (
    assign_role_to_users,
    create_role_row,
    delete_roles,
    delete_user_roles_by_ids,
    fill_role_user_roles,
    get_role_dept_ids,
    get_role_menu_ids,
    get_role_row,
    list_role_user_ids,
    list_role_users_rows,
    list_roles,
    update_role_permission_row,
    update_role_row,
)


def _format_time(dt) -> str:
    """统一时间格式化：YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


class RoleReq(BaseModel):
    """创建/修改角色请求体，字段与前端 RoleReq 对齐。"""

    name: str = Field(..., description="角色名称")
    code: str | None = Field(None, description="角色编码（仅新增必填）")
    sort: int = Field(999, description="排序")
    description: str | None = Field(None, description="描述")
    dataScope: int = Field(4, description="数据范围")
    deptIds: List[int] = Field(default_factory=list, description="部门 ID 列表")
    deptCheckStrictly: bool = Field(True, description="部门数据权限是否严格模式")


class RolePermissionReq(BaseModel):
    """更新角色菜单权限请求体。"""

    menuIds: List[int] = Field(default_factory=list, description="菜单 ID 列表")
    menuCheckStrictly: bool = Field(True, description="菜单数据权限是否严格模式")


def list_role_service(description: str | None) -> List[Dict[str, Any]]:
    """查询角色列表，结构与前端 RoleResp[] 对齐。"""
    rows = list_roles()
    desc = (description or "").strip()

    data: List[Dict[str, Any]] = []
    for r in rows:
        name = r["name"] or ""
        desc_text = r["description"] or ""
        if desc and (desc not in name) and (desc not in desc_text):
            continue
        is_system = bool(r["is_system"])
        code = r["code"] or ""
        item = {
            "id": int(r["id"]),
            "name": name,
            "code": code,
            "sort": int(r["sort"]),
            "description": desc_text,
            "dataScope": int(r["data_scope"]),
            "isSystem": is_system,
            "createUserString": r["create_user_name"],
            "createTime": _format_time(r["create_time"]),
            "updateUserString": r["update_user_name"],
            "updateTime": _format_time(r["update_time"]),
            "disabled": is_system and code == "admin",
        }
        data.append(item)
    return data


def get_role_service(role_id: int) -> Optional[Dict[str, Any]]:
    """获取角色详情，返回 RoleDetailResp 结构。"""
    row = get_role_row(role_id)
    if not row:
        return None

    is_system = bool(row["is_system"])
    code = row["code"] or ""
    base = {
        "id": int(row["id"]),
        "name": row["name"],
        "code": code,
        "sort": int(row["sort"]),
        "description": row["description"],
        "dataScope": int(row["data_scope"]),
        "isSystem": is_system,
        "createUserString": row["create_user_name"],
        "createTime": _format_time(row["create_time"]),
        "updateUserString": row["update_user_name"],
        "updateTime": _format_time(row["update_time"]),
        "disabled": is_system and code == "admin",
    }

    menu_ids = get_role_menu_ids(role_id)
    dept_ids = get_role_dept_ids(role_id)

    resp = {
        **base,
        "menuIds": menu_ids,
        "deptIds": dept_ids,
        "menuCheckStrictly": bool(row["menu_check_strictly"]),
        "deptCheckStrictly": bool(row["dept_check_strictly"]),
    }
    return resp


def create_role_service(req: RoleReq, current_uid: int) -> int:
    """新增角色服务实现。"""
    name = (req.name or "").strip()
    code = (req.code or "").strip()
    if not name or not code:
        raise ValueError("名称和编码不能为空")

    sort = req.sort or 999
    data_scope = req.dataScope or 4
    dept_check_strict = bool(req.deptCheckStrictly)

    new_id = create_role_row(
        name=name,
        code=code,
        sort=sort,
        description=req.description,
        data_scope=data_scope,
        dept_check_strict=dept_check_strict,
        dept_ids=req.deptIds or [],
        create_user=current_uid,
    )
    return new_id


def update_role_service(role_id: int, req: RoleReq, current_uid: int) -> None:
    """修改角色基础信息及数据范围服务实现。"""
    name = (req.name or "").strip()
    if not name:
        raise ValueError("名称不能为空")

    sort = req.sort or 999
    data_scope = req.dataScope or 4
    dept_check_strict = bool(req.deptCheckStrictly)

    update_role_row(
        role_id=role_id,
        name=name,
        description=req.description,
        sort=sort,
        data_scope=data_scope,
        dept_check_strict=dept_check_strict,
        dept_ids=req.deptIds or [],
        update_user=current_uid,
    )


def delete_role_service(ids: List[int]) -> None:
    """删除角色服务实现（跳过系统内置 admin 角色）。"""
    delete_roles(ids)


def update_role_permission_service(
    role_id: int,
    req: RolePermissionReq,
    current_uid: int,
) -> None:
    """更新角色的菜单权限服务实现。"""
    update_role_permission_row(
        role_id=role_id,
        menu_ids=req.menuIds or [],
        menu_check_strictly=bool(req.menuCheckStrictly),
        update_user=current_uid,
    )


def page_role_user_service(
    role_id: int,
    page: int,
    size: int,
    description: str | None,
) -> Dict[str, Any]:
    """分页查询角色关联用户服务实现。"""
    rows = list_role_users_rows(role_id)
    desc = (description or "").strip()

    all_items: List[Dict[str, Any]] = []
    for r in rows:
        username = r["username"]
        nickname = r["nickname"]
        desc_text = r["description"]
        if desc and (desc not in username) and (desc not in nickname) and (desc not in desc_text):
            continue
        item = {
            "id": int(r["id"]),
            "roleId": int(r["role_id"]),
            "userId": int(r["user_id"]),
            "username": username,
            "nickname": nickname,
            "gender": int(r["gender"]),
            "status": int(r["status"]),
            "isSystem": bool(r["is_system"]),
            "description": desc_text,
            "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
            "deptName": r["dept_name"],
            "roleIds": [],
            "roleNames": [],
            "disabled": False,
        }
        all_items.append(item)

    fill_role_user_roles(all_items)

    for item in all_items:
        # admin 角色下的系统用户禁用操作
        item["disabled"] = item["isSystem"] and item["roleId"] == 1

    total = len(all_items)
    if page <= 0:
        page = 1
    if size <= 0:
        size = 10
    start = (page - 1) * size
    if start > total:
        start = total
    end = start + size
    if end > total:
        end = total
    page_list = all_items[start:end]

    return {"list": page_list, "total": total}


def assign_to_users_service(role_id: int, user_ids: List[int]) -> None:
    """为角色分配用户服务实现。"""
    ids = [i for i in (user_ids or []) if i > 0]
    if not ids:
        raise ValueError("用户ID列表不能为空")
    assign_role_to_users(role_id, ids)


def unassign_from_users_service(ids: List[int]) -> None:
    """按 user_role 记录 ID 取消用户与角色的关联。"""
    valid_ids = [i for i in (ids or []) if i > 0]
    if not valid_ids:
        raise ValueError("用户角色ID列表不能为空")
    delete_user_roles_by_ids(valid_ids)


def list_role_user_ids_service(role_id: int) -> List[int]:
    """查询角色下所有用户 ID 列表服务实现。"""
    return list_role_user_ids(role_id)

