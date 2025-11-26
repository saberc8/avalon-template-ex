from __future__ import annotations

import hashlib
import os
from datetime import datetime
from typing import Any, Dict, List

from fastapi import APIRouter, Body, File, Form, Header, Query, UploadFile
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_file_service import (
    calc_dir_size_service,
    check_file_service,
    create_dir_service,
    delete_file_service,
    file_statistics_service,
    list_file_service,
    upload_file_service,
    update_file_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


@router.post("/system/file/upload")
@router.post("/common/file")
def upload_file(
    file: UploadFile = File(...),
    parentPath: str | None = Form(default=None),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """上传文件，兼容 /system/file/upload 与 /common/file。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        data = upload_file_service(file, parentPath, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))
    return ok(data)


@router.get("/system/file")
def list_file(
    originalName: str | None = Query(default=None),
    type: str | None = Query(default=None),
    parentPath: str | None = Query(default=None),
    page: int = 1,
    size: int = 30,
):
    """分页查询文件列表：GET /system/file。"""
    data = list_file_service(originalName, type, parentPath, page, size)
    return ok(data)


@router.post("/system/file/dir")
def create_dir(
    body: Dict[str, Any] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """创建文件夹：POST /system/file/dir。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        create_dir_service(body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.get("/system/file/dir/{dir_id}/size")
def calc_dir_size(dir_id: int):
    """计算文件夹大小：GET /system/file/dir/{id}/size。"""
    if dir_id <= 0:
        return fail("400", "ID 参数不正确")
    try:
        data = calc_dir_size_service(dir_id)
    except ValueError as e:
        return fail("400", str(e))
    return ok(data)


@router.get("/system/file/statistics")
def file_statistics():
    """文件资源统计：GET /system/file/statistics。"""
    data = file_statistics_service()
    return ok(data)


@router.get("/system/file/check")
def check_file(fileHash: str | None = Query(default=None)):
    """检测文件是否存在：GET /system/file/check?fileHash=...。"""
    data = check_file_service(fileHash)
    return ok(data)


@router.put("/system/file/{file_id}")
def update_file(
    file_id: int,
    body: Dict[str, Any] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """重命名文件：PUT /system/file/{id}。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if file_id <= 0:
        return fail("400", "ID 参数不正确")

    try:
        update_file_service(file_id, body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/file")
def delete_file(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除文件：DELETE /system/file。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = body.ids or []
    try:
        delete_file_service(ids, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)
