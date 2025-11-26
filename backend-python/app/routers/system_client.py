from __future__ import annotations

from typing import List

from fastapi import APIRouter, Body, Header, Query
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_client_service import (
    ClientReq,
    create_client_service,
    delete_client_service,
    get_client_detail_service,
    list_client_service,
    update_client_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


class IdsBody(BaseModel):
    """通用 ID 列表请求体。"""

    ids: List[int]


@router.get("/system/client")
def list_client(
    page: int = Query(1, ge=1),
    size: int = Query(10, ge=1),
    clientType: str | None = Query(default=None),
    status: str | None = Query(default=None),
    authType: List[str] | None = Query(default=None),
    sort: List[str] | None = Query(default=None),
):
    """
    分页查询客户端列表：GET /system/client。

    返回数据结构为 PageRes<ClientResp>，与前端类型定义对齐：
    {
      "list": [...],
      "total": 0
    }
    """
    data = list_client_service(page, size, clientType, status, authType, sort)
    return ok(data)


@router.get("/system/client/{client_id}")
def get_client_detail(client_id: int):
    """查询单个客户端详情：GET /system/client/{id}。"""
    if client_id <= 0:
        return fail("400", "ID 参数不正确")
    data = get_client_detail_service(client_id)
    if not data:
        return fail("404", "客户端不存在")
    return ok(data)


@router.post("/system/client")
def create_client(
    req: ClientReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增客户端：POST /system/client。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        data = create_client_service(req, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(data)


@router.put("/system/client/{client_id}")
def update_client(
    client_id: int,
    req: ClientReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改客户端：PUT /system/client/{id}。"""
    if client_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_client_service(client_id, req, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "客户端不存在":
            return fail("404", msg)
        return fail("400", msg)

    return ok(True)


@router.delete("/system/client")
def delete_client(
    body: IdsBody = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """批量删除客户端：DELETE /system/client。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    try:
        delete_client_service(ids)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)
