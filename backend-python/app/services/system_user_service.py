from __future__ import annotations

from typing import Any, Dict, List, Optional, Sequence

from pydantic import BaseModel

from ..config import get_settings
from ..security.jwt_token import TokenService
from ..security.password import PasswordHasher
from ..security.rsa import RSADecryptor
from ..repositories.system_user_repository import (
    create_user_row,
    delete_users_soft,
    export_user_rows,
    fill_user_roles,
    get_user_detail_row,
    list_users,
    list_users_page,
    reset_user_password,
    update_user_roles_only,
    update_user_row,
)


def _format_time(dt) -> str:
    """统一的时间格式化：`YYYY-MM-DD HH:MM:SS`。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _get_rsa_decryptor() -> RSADecryptor:
    """构造 RSA 解密器实例。"""
    s = get_settings()
    return RSADecryptor.from_base64_key(s.rsa_private_key_b64)


def current_user_id(authorization: str | None) -> Optional[int]:
    """
    从 Authorization 头解析当前登录用户 ID。

    未授权时返回 None，由调用方决定是否直接返回 401。
    """
    token_svc = _get_token_service()
    if not authorization:
        return None
    claims = token_svc.parse(authorization)
    if not claims:
        return None
    return claims.user_id


class UserReq(BaseModel):
    """创建/修改用户请求体，字段与前端 `UserReq` 对齐。"""

    username: str
    nickname: str
    password: str = ""
    gender: int = 1
    email: str | None = None
    phone: str | None = None
    avatar: str | None = None
    description: str | None = None
    status: int = 1
    deptId: int
    roleIds: List[int] = []


class UserPasswordResetReq(BaseModel):
    """重置密码请求体。"""

    newPassword: str


class UserRoleUpdateReq(BaseModel):
    """更新用户角色请求体。"""

    roleIds: List[int] = []


def build_user_brief_list(rows: Sequence[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """将用户行数据格式化为列表结构。"""
    users: List[Dict[str, Any]] = []
    for r in rows:
        users.append(
            {
                "id": int(r["id"]),
                "username": r["username"],
                "nickname": r["nickname"],
                "avatar": r["avatar"],
                "gender": int(r["gender"]),
                "email": r["email"],
                "phone": r["phone"],
                "description": r["description"],
                "status": int(r["status"]),
                "isSystem": bool(r["is_system"]),
                "createUserString": r.get("create_user_name", ""),
                "createTime": _format_time(r.get("create_time")),
                "updateUserString": r.get("update_user_name", ""),
                "updateTime": _format_time(r.get("update_time")),
                "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
                "deptName": r.get("dept_name", ""),
                "roleIds": [],
                "roleNames": [],
                "disabled": bool(r["is_system"]),
            }
        )
    return users


def list_user_page_service(
    page: int,
    size: int,
    description: str | None,
    status: int | None,
    dept_id: int | None,
) -> Dict[str, Any]:
    """分页查询用户列表的服务层封装。"""
    data = list_users_page(page, size, description, status, dept_id)
    if data["total"] == 0:
        return {"list": [], "total": 0}

    rows = data["list"]
    users = build_user_brief_list(rows)

    # 补充角色信息
    from ..db import get_db_cursor

    with get_db_cursor() as cur:
        fill_user_roles(cur, users)

    return {"list": users, "total": data["total"]}


def list_user_all_service(user_ids: Optional[Sequence[int]] = None) -> List[Dict[str, Any]]:
    """查询全部用户或指定 ID 列表。"""
    rows = list_users(user_ids)
    users = build_user_brief_list(rows)

    from ..db import get_db_cursor

    with get_db_cursor() as cur:
        fill_user_roles(cur, users)

    return users


def get_user_detail_service(user_id: int) -> Optional[Dict[str, Any]]:
    """查询单个用户详情。"""
    row = get_user_detail_row(user_id)
    if not row:
        return None

    pwd_reset = row.get("pwd_reset_time")
    user: Dict[str, Any] = {
        "id": int(row["id"]),
        "username": row["username"],
        "nickname": row["nickname"],
        "avatar": row["avatar"],
        "gender": int(row["gender"]),
        "email": row["email"],
        "phone": row["phone"],
        "description": row["description"],
        "status": int(row["status"]),
        "isSystem": bool(row["is_system"]),
        "createUserString": row.get("create_user_name", ""),
        "createTime": _format_time(row.get("create_time")),
        "updateUserString": row.get("update_user_name", ""),
        "updateTime": _format_time(row.get("update_time")),
        "deptId": int(row["dept_id"]) if row["dept_id"] is not None else 0,
        "deptName": row.get("dept_name", ""),
        "roleIds": [],
        "roleNames": [],
        "disabled": bool(row["is_system"]),
        "pwdResetTime": _format_time(pwd_reset),
    }

    from ..db import get_db_cursor

    with get_db_cursor() as cur:
        fill_user_roles(cur, [user])

    return user


def create_user_service(req: UserReq, current_uid: int) -> int:
    """新增用户服务实现，包含密码解密与强度校验。"""
    username = (req.username or "").strip()
    nickname = (req.nickname or "").strip()
    if not username or not nickname:
        raise ValueError("用户名和昵称不能为空")
    if not req.deptId:
        raise ValueError("所属部门不能为空")
    if not (req.password or "").strip():
        raise ValueError("密码不能为空")

    decryptor = _get_rsa_decryptor()
    try:
        raw_pwd = decryptor.decrypt_base64(req.password)
    except Exception as exc:  # noqa: BLE001
        raise ValueError("密码解密失败") from exc

    if not (8 <= len(raw_pwd) <= 32):
        raise ValueError("密码长度为 8-32 个字符，至少包含字母和数字")
    has_letter = any(ch.isalpha() for ch in raw_pwd)
    has_digit = any(ch.isdigit() for ch in raw_pwd)
    if not (has_letter and has_digit):
        raise ValueError("密码长度为 8-32 个字符，至少包含字母和数字")

    encoded_pwd = PasswordHasher.hash(raw_pwd)

    new_id = create_user_row(
        username=username,
        nickname=nickname,
        encoded_pwd=encoded_pwd,
        gender=req.gender,
        email=req.email,
        phone=req.phone,
        avatar=req.avatar,
        description=req.description,
        status=req.status or 1,
        dept_id=req.deptId,
        create_user=current_uid,
        role_ids=req.roleIds,
    )
    return new_id


def update_user_service(user_id: int, req: UserReq, current_uid: int) -> None:
    """更新用户基本信息及角色。"""
    username = (req.username or "").strip()
    nickname = (req.nickname or "").strip()
    if not username or not nickname:
        raise ValueError("用户名和昵称不能为空")
    if not req.deptId:
        raise ValueError("所属部门不能为空")

    update_user_row(
        user_id=user_id,
        username=username,
        nickname=nickname,
        gender=req.gender,
        email=req.email,
        phone=req.phone,
        avatar=req.avatar,
        description=req.description,
        status=req.status or 1,
        dept_id=req.deptId,
        update_user=current_uid,
        role_ids=req.roleIds,
    )


def delete_users_service(ids: Sequence[int], current_uid: int) -> None:  # noqa: ARG001
    """批量删除用户（忽略系统用户），当前用户 ID 仅预留扩展。"""
    delete_users_soft(ids)


def reset_password_service(
    user_id: int,
    req: UserPasswordResetReq,
    current_uid: int,
) -> None:
    """重置用户密码服务实现。"""
    if not (req.newPassword or "").strip():
        raise ValueError("密码不能为空")

    decryptor = _get_rsa_decryptor()
    try:
        raw_pwd = decryptor.decrypt_base64(req.newPassword)
    except Exception as exc:  # noqa: BLE001
        raise ValueError("密码解密失败") from exc

    if not (8 <= len(raw_pwd) <= 32):
        raise ValueError("密码长度为 8-32 个字符，至少包含字母和数字")
    has_letter = any(ch.isalpha() for ch in raw_pwd)
    has_digit = any(ch.isdigit() for ch in raw_pwd)
    if not (has_letter and has_digit):
        raise ValueError("密码长度为 8-32 个字符，至少包含字母和数字")

    encoded_pwd = PasswordHasher.hash(raw_pwd)
    reset_user_password(user_id, encoded_pwd, current_uid)


def update_user_role_service(
    user_id: int,
    req: UserRoleUpdateReq,
    current_uid: int,  # noqa: ARG001
) -> None:
    """仅调整用户角色绑定。"""
    update_user_roles_only(user_id, req.roleIds)


def export_user_service() -> str:
    """导出用户 CSV 内容。"""
    rows = export_user_rows()
    lines = ["username,nickname,gender,email,phone"]
    for r in rows:
        line = f'{r["username"]},{r["nickname"]},{int(r["gender"])},{r["email"]},{r["phone"]}'
        lines.append(line)
    content = "\n".join(lines) + "\n"
    return content

