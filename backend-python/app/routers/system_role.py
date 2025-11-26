from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Body, Header
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_role_service import (
    RolePermissionReq,
    RoleReq,
    assign_to_users_service,
    create_role_service,
    delete_role_service,
    get_role_service,
    list_role_service,
    list_role_user_ids_service,
    page_role_user_service,
    unassign_from_users_service,
    update_role_permission_service,
    update_role_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


class IdsBody(BaseModel):
    """通用 ID 列表请求体。"""

    ids: List[int]


@router.get("/system/role/list")
def list_role(description: str | None = None):
    """查询角色列表，结构与前端 RoleResp[] 对齐。"""
    data = list_role_service(description)
    return ok(data)


@router.get("/system/role/{role_id}")
def get_role(role_id: int):
    """获取角色详情，返回 RoleDetailResp 结构。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    resp = get_role_service(role_id)
    if not resp:
        return fail("404", "角色不存在")
    return ok(resp)


@router.post("/system/role")
def create_role(
    req: RoleReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增角色。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        new_id = create_role_service(req, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    return ok({"id": new_id})


@router.put("/system/role/{role_id}")
def update_role(
    role_id: int,
    req: RoleReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改角色基础信息及数据范围。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        update_role_service(role_id, req, current_uid)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


@router.delete("/system/role")
def delete_role(
    body: IdsBody = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除角色（跳过系统内置角色）。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    if not ids:
        return fail("400", "ID 列表不能为空")

    delete_role_service(ids)

    return ok(True)


@router.put("/system/role/{role_id}/permission")
def update_role_permission(
    role_id: int,
    req: RolePermissionReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """更新角色的菜单权限。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    update_role_permission_service(role_id, req, current_uid)

    return ok(True)


@router.get("/system/role/{role_id}/user")
def page_role_user(
    role_id: int,
    page: int = 1,
    size: int = 10,
    description: str | None = None,
):
    """分页查询角色关联用户。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    data = page_role_user_service(role_id, page, size, description)
    return ok(data)


@router.post("/system/role/{role_id}/user")
def assign_to_users(
    role_id: int,
    user_ids: List[int] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """为角色分配用户。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        assign_to_users_service(role_id, user_ids)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


@router.delete("/system/role/user")
def unassign_from_users(
    ids: List[int] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """取消用户与角色的关联（按 user_role 记录 ID）。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        unassign_from_users_service(ids)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


@router.get("/system/role/{role_id}/user/id")
def list_role_user_ids(role_id: int):
    """查询角色下所有用户 ID 列表。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    ids = list_role_user_ids_service(role_id)
    return ok(ids)
