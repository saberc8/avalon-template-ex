from __future__ import annotations

from typing import List

from fastapi import APIRouter, Body, Header, Query
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_storage_service import (
    StatusReq,
    StorageReq,
    create_storage_service,
    delete_storage_service,
    get_storage_service,
    list_storage_service,
    set_default_storage_service,
    update_storage_service,
    update_storage_status_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


class IdsBody(BaseModel):
    """通用 ID 列表请求体。"""

    ids: List[int]


@router.get("/system/storage/list")
def list_storage(
    description: str | None = Query(default=None),
    type: int | None = Query(default=None),
    sort: List[str] | None = Query(default=None),
):
    """
    查询存储列表：GET /system/storage/list。

    返回 StorageResp[]，供前端按 type 分组展示。
    """
    data = list_storage_service(description, type, sort)
    return ok(data)


@router.get("/system/storage/{storage_id}")
def get_storage(storage_id: int):
    """查询存储详情：GET /system/storage/{id}。"""
    if storage_id <= 0:
        return fail("400", "ID 参数不正确")
    data = get_storage_service(storage_id)
    if not data:
        return fail("404", "存储不存在")
    return ok(data)


@router.post("/system/storage")
def create_storage(
    req: StorageReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增存储：POST /system/storage。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        new_id = create_storage_service(req, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok({"id": new_id})


@router.put("/system/storage/{storage_id}")
def update_storage(
    storage_id: int,
    req: StorageReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改存储：PUT /system/storage/{id}。"""
    if storage_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_storage_service(storage_id, req, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "存储不存在":
            return fail("404", msg)
        return fail("400", msg)
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/storage")
def delete_storage(
    body: IdsBody = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除存储：DELETE /system/storage。默认存储不允许删除。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    try:
        delete_storage_service(ids)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


@router.put("/system/storage/{storage_id}/status")
def update_storage_status(
    storage_id: int,
    req: StatusReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改存储状态：PUT /system/storage/{id}/status。"""
    if storage_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_storage_status_service(storage_id, req, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "存储不存在":
            return fail("404", msg)
        return fail("400", msg)

    return ok(True)


@router.put("/system/storage/{storage_id}/default")
def set_default_storage(
    storage_id: int,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """设为默认存储：PUT /system/storage/{id}/default。"""
    if storage_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        set_default_storage_service(storage_id, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "存储不存在":
            return fail("404", msg)
        return fail("400", msg)

    return ok(True)
