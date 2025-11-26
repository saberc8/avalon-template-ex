from __future__ import annotations

from typing import List

from fastapi import APIRouter, Body, Header, Path, Query
from pydantic import BaseModel

from ..api_response import fail, ok
from ..services.system_dept_service import (
    DeptReq,
    delete_dept_service,
    export_dept_service,
    get_dept_service,
    list_dept_tree_service,
    create_dept_service,
    update_dept_service,
)
from ..services.system_user_service import current_user_id

router = APIRouter()


class DeleteDeptReq(BaseModel):
    """删除部门请求体。"""

    ids: List[int]


@router.get("/system/dept/tree")
def list_dept_tree(
    description: str | None = Query(default=None),
    status: str | None = Query(default=None),
):
    """部门树：GET /system/dept/tree。"""
    data = list_dept_tree_service(description, status)
    return ok(data)


@router.get("/system/dept/{dept_id}")
def get_dept(dept_id: int = Path(..., alias="dept_id")):
    """获取部门详情：GET /system/dept/{id}。"""
    if dept_id <= 0:
        return fail("400", "无效的部门 ID")
    data = get_dept_service(dept_id)
    if not data:
        return fail("404", "部门不存在")
    return ok(data)


@router.post("/system/dept")
def create_dept(
    body: DeptReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增部门：POST /system/dept。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        create_dept_service(body, current_uid)
    except ValueError as e:
        return fail("400", str(e))
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.put("/system/dept/{dept_id}")
def update_dept(
    dept_id: int = Path(..., alias="dept_id"),
    body: DeptReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改部门：PUT /system/dept/{id}。"""
    if dept_id <= 0:
        return fail("400", "无效的部门 ID")

    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_dept_service(dept_id, body, current_uid)
    except ValueError as e:
        msg = str(e)
        if msg == "部门不存在":
            return fail("404", msg)
        return fail("400", msg)
    except RuntimeError as e:
        return fail("500", str(e))

    return ok(True)


@router.delete("/system/dept")
def delete_dept(body: DeleteDeptReq = Body(...)):
    """删除部门：DELETE /system/dept。"""
    try:
        delete_dept_service(body.ids or [])
    except ValueError as e:
        return fail("400", str(e))
    return ok(True)


@router.get("/system/dept/export")
def export_dept(
    description: str | None = Query(default=None),
    status: str | None = Query(default=None),
):
    """导出部门 CSV：GET /system/dept/export。"""
    content = export_dept_service(description, status)
    return content
