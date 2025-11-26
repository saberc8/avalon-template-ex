from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Body, File, Header, UploadFile
from fastapi.responses import Response
from pydantic import BaseModel

from ..api_response import fail, ok
from ..models.auth import RouteItem  # 仅为类型兼容引入，避免循环依赖检查误报
from ..security.password import PasswordVerifier
from ..services.system_user_service import (
    UserPasswordResetReq,
    UserReq,
    UserRoleUpdateReq,
    create_user_service,
    current_user_id,
    delete_users_service,
    export_user_service,
    get_user_detail_service,
    list_user_all_service,
    list_user_page_service,
    reset_password_service,
    update_user_role_service,
    update_user_service,
)

router = APIRouter()


@router.get("/system/user")
def list_user_page(
    page: int = 1,
    size: int = 10,
    description: str | None = None,
    status: int | None = None,
    deptId: int | None = None,
):
    """分页查询用户列表，响应结构与 Java/Go UserResp PageResult 一致。"""
    data = list_user_page_service(page, size, description, status, deptId)
    return ok(data)


@router.get("/system/user/list")
def list_user_all(userIds: List[int] | None = None):
    """查询全部用户或指定 ID 列表，结构与 Java/Go 一致。"""
    users = list_user_all_service(userIds)
    return ok(users)


@router.get("/system/user/{user_id}")
def get_user_detail(user_id: int):
    """查询单个用户详情，返回 UserDetailResp 结构。"""
    if user_id <= 0:
        return fail("400", "ID 参数不正确")
    user = get_user_detail_service(user_id)
    if not user:
        return fail("404", "用户不存在")
    return ok(user)


@router.post("/system/user")
def create_user(
    req: UserReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增用户，逻辑与 Go/Java 保持一致（含 RSA 解密与密码强度校验）。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    try:
        new_id = create_user_service(req, current_uid)
    except ValueError as e:
        return fail("400", str(e))

    return ok({"id": new_id})


@router.put("/system/user/{user_id}")
def update_user(
    user_id: int,
    req: UserReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """修改用户基本信息及角色绑定。"""
    if user_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        update_user_service(user_id, req, current_uid)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


class IdsBody(BaseModel):
    """通用 ID 列表请求体。"""

    ids: List[int]


@router.delete("/system/user")
def delete_user(
    body: IdsBody = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """批量删除用户，系统内置用户不会被删除。"""
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    if not ids:
        return fail("400", "ID 列表不能为空")

    delete_users_service(ids, current_uid)

    return ok(True)


@router.patch("/system/user/{user_id}/password")
def reset_password(
    user_id: int,
    req: UserPasswordResetReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """重置用户密码，逻辑与 Go/Java 一致。"""
    if user_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    try:
        reset_password_service(user_id, req, current_uid)
    except ValueError as e:
        return fail("400", str(e))

    return ok(True)


@router.patch("/system/user/{user_id}/role")
def update_user_role(
    user_id: int,
    req: UserRoleUpdateReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """仅调整用户角色绑定。"""
    if user_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    update_user_role_service(user_id, req, current_uid)

    return ok(True)


@router.get("/system/user/export")
def export_user():
    """
    导出用户为 CSV 文件。

    说明：与 Go 版一致，直接返回文件流，不包裹在统一响应结构中。
    """
    content = export_user_service()

    headers = {
        "Content-Type": "text/csv; charset=utf-8",
        "Content-Disposition": 'attachment; filename="users.csv"',
    }
    return Response(content=content, media_type="text/csv; charset=utf-8", headers=headers)


@router.get("/system/user/import/template")
def download_import_template():
    """下载用户导入模板（CSV），仅提供基础列头。"""
    content = "username,nickname,gender,email,phone\n"
    headers = {
        "Content-Type": "text/csv; charset=utf-8",
        "Content-Disposition": 'attachment; filename="user_import_template.csv"',
    }
    return Response(content=content, media_type="text/csv; charset=utf-8", headers=headers)


class UserImportParseResp(BaseModel):
    """解析导入文件返回结果（简化版，与 Go 当前实现一致）。"""

    importKey: str
    totalRows: int
    validRows: int
    duplicateUserRows: int
    duplicateEmailRows: int
    duplicatePhoneRows: int


class UserImportResp(BaseModel):
    """实际导入结果（占位结构，与前端类型对齐）。"""

    totalRows: int
    insertRows: int
    updateRows: int


@router.post("/system/user/import/parse")
def parse_import_user(file: UploadFile = File(...)):
    """
    解析导入文件。

    说明：当前实现与 Go 版一致，仅校验文件非空并返回一个空的解析结果，
    用于打通前端流程，后续可按需补充真实解析逻辑。
    """
    if not file.filename:
        return fail("400", "文件不能为空")

    resp = UserImportParseResp(
        importKey=str(next_id()),
        totalRows=0,
        validRows=0,
        duplicateUserRows=0,
        duplicateEmailRows=0,
        duplicatePhoneRows=0,
    )
    return ok(resp.dict())


@router.post("/system/user/import")
def import_user(_: Dict[str, Any] = Body(...)):
    """
    执行导入。

    说明：当前与 Go/Java 简化实现保持一致，返回全 0 统计，主要用于前端流程兼容。
    """
    resp = UserImportResp(totalRows=0, insertRows=0, updateRows=0)
    return ok(resp.dict())
