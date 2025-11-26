from __future__ import annotations

from typing import List

from fastapi import APIRouter, Body, Header, Path, Query
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_dict_service import (
    DictItemReq,
    DictReq,
    delete_dict_item_service,
    delete_dict_service,
    get_dict_item_service,
    get_dict_service,
    list_dict_item_service,
    list_dict_service,
    create_dict_item_service,
    create_dict_service,
    update_dict_item_service,
    update_dict_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


class IdsRequest(BaseModel):
    """批量 ID 请求体。"""

    ids: List[int]


@router.get("/system/dict/list")
def list_dict(description: str | None = Query(default=None)):
    """查询字典列表：GET /system/dict/list。"""
    data = list_dict_service(description)
    return ok(data)


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
        data = list_dict_item_service(dictId, page, size, description, status)
    except ValueError as e:
        return fail("400", str(e))
    return ok(data)


@router.get("/system/dict/item/{item_id}")
def get_dict_item(item_id: int = Path(..., alias="item_id")):
    """查询字典项详情：GET /system/dict/item/{id}。"""
    if item_id <= 0:
        return fail("400", "ID 参数不正确")
    data = get_dict_item_service(item_id)
    if not data:
        return fail("404", "字典项不存在")
    return ok(data)


@router.post("/system/dict/item")
def create_dict_item(
    body: DictItemReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增字典项：POST /system/dict/item。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        create_dict_item_service(body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.put("/system/dict/item/{item_id}")
def update_dict_item(
    item_id: int = Path(..., alias="item_id"),
    body: DictItemReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改字典项：PUT /system/dict/item/{id}。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if item_id <= 0:
        return fail("400", "ID 参数不正确")

    try:
        update_dict_item_service(item_id, body, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "字典项不存在":
            return fail("404", msg)
        return fail("400", msg)
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/dict/item")
def delete_dict_item(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除字典项：DELETE /system/dict/item。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        delete_dict_item_service(body.ids or [])
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.get("/system/dict/{dict_id}")
def get_dict(dict_id: int = Path(..., alias="dict_id")):
    """查询字典详情：GET /system/dict/{id}。"""
    if dict_id <= 0:
        return fail("400", "ID 参数不正确")
    data = get_dict_service(dict_id)
    if not data:
        return fail("404", "字典不存在")
    return ok(data)


@router.post("/system/dict")
def create_dict(
    body: DictReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增字典：POST /system/dict。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        new_id = create_dict_service(body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok({"id": new_id})


@router.put("/system/dict/{dict_id}")
def update_dict(
    dict_id: int = Path(..., alias="dict_id"),
    body: DictReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改字典：PUT /system/dict/{id}。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if dict_id <= 0:
        return fail("400", "ID 参数不正确")

    try:
        update_dict_service(dict_id, body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/dict")
def delete_dict(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除字典：DELETE /system/dict。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        delete_dict_service(body.ids or [])
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/dict/cache/{code}")
def clear_dict_cache(code: str = Path(...)):
    """清理字典缓存（当前无缓存逻辑，直接返回成功）。"""
    return ok(True)
