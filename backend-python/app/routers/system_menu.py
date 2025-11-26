from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Body, Header, Path
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_menu_service import (
    MenuReq,
    create_menu_service,
    delete_menu_service,
    get_menu_service,
    list_menu_tree_service,
    update_menu_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


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
    data = list_menu_tree_service()
    return ok(data)


@router.get("/system/menu/{menu_id}")
def get_menu(menu_id: int = Path(..., alias="menu_id")):
    """查询菜单详情：GET /system/menu/{id}。"""
    if menu_id <= 0:
        return fail("400", "ID 参数不正确")
    resp = get_menu_service(menu_id)
    if not resp:
        return fail("404", "菜单不存在")
    return ok(resp)


@router.post("/system/menu")
def create_menu(
    body: MenuReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增菜单：POST /system/menu。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        new_id = create_menu_service(body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
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

    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_menu_service(menu_id, body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except Exception:
        return fail("500", "修改菜单失败")

    return ok(True)


@router.delete("/system/menu")
def delete_menu(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """批量删除菜单（含子节点）：DELETE /system/menu。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        delete_menu_service(body.ids)
    except ValueError as e:
        return fail("400", str(e))
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
