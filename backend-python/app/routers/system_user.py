from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Body, File, Header, UploadFile
from fastapi.responses import Response
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..db import get_db_cursor
from ..id_generator import next_id
from ..models.auth import RouteItem  # 仅为类型兼容引入，避免循环依赖检查误报
from ..security.jwt_token import TokenService
from ..security.password import PasswordHasher, PasswordVerifier
from ..security.rsa import RSADecryptor
from ..config import get_settings

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例（与 auth 路由保持一致）。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _get_rsa_decryptor() -> RSADecryptor:
    """构造 RSA 解密器实例。"""
    s = get_settings()
    return RSADecryptor.from_base64_key(s.rsa_private_key_b64)


def _format_time(dt) -> str:
    """统一的时间格式化：`YYYY-MM-DD HH:MM:SS`。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def _current_user_id(
    authorization: str | None,
) -> Optional[int]:
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

    username: str = Field(..., description="用户名")
    nickname: str = Field(..., description="昵称")
    password: str = Field("", description="密码（仅新增时必填，RSA 加密）")
    gender: int = Field(1, description="性别")
    email: str | None = Field(None, description="邮箱")
    phone: str | None = Field(None, description="手机号")
    avatar: str | None = Field(None, description="头像地址")
    description: str | None = Field(None, description="描述")
    status: int = Field(1, description="状态 1=启用 2=禁用")
    deptId: int = Field(..., description="部门 ID")
    roleIds: List[int] = Field(default_factory=list, description="角色 ID 列表")


class UserPasswordResetReq(BaseModel):
    """重置密码请求体。"""

    newPassword: str = Field(..., description="新密码（RSA 加密）")


class UserRoleUpdateReq(BaseModel):
    """更新用户角色请求体。"""

    roleIds: List[int] = Field(default_factory=list, description="角色 ID 列表")


@router.get("/system/user")
def list_user_page(
    page: int = 1,
    size: int = 10,
    description: str | None = None,
    status: int | None = None,
    deptId: int | None = None,
):
    """分页查询用户列表，响应结构与 Java/Go UserResp PageResult 一致。"""
    if page <= 0:
        page = 1
    if size <= 0:
        size = 10

    where_clauses: List[str] = ["1=1"]
    params: List[Any] = []

    if description:
        like = f"%{description.strip()}%"
        where_clauses.append(
            "(u.username ILIKE %s OR u.nickname ILIKE %s OR COALESCE(u.description,'') ILIKE %s)"
        )
        params.extend([like, like, like])
    if status and status > 0:
        where_clauses.append("u.status = %s")
        params.append(status)
    if deptId and deptId > 0:
        where_clauses.append("u.dept_id = %s")
        params.append(deptId)

    where_sql = " WHERE " + " AND ".join(where_clauses)

    with get_db_cursor() as cur:
        cur.execute("SELECT COUNT(*) AS cnt FROM sys_user AS u" + where_sql, params)
        row = cur.fetchone()
        total = int(row["cnt"]) if row and row["cnt"] is not None else 0
        if total == 0:
            return ok({"list": [], "total": 0})

        offset = (page - 1) * size
        list_sql = f"""
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '')       AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '')  AS create_user_name,
       u.update_time,
       COALESCE(uu.nickname, '')  AS update_user_name
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
{where_sql}
ORDER BY u.id DESC
LIMIT %s OFFSET %s;
"""
        cur.execute(list_sql, [*params, size, offset])
        rows = cur.fetchall()

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
                    "createUserString": r["create_user_name"],
                    "createTime": _format_time(r["create_time"]),
                    "updateUserString": r["update_user_name"],
                    "updateTime": _format_time(r["update_time"]),
                    "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
                    "deptName": r["dept_name"],
                    "roleIds": [],
                    "roleNames": [],
                    "disabled": bool(r["is_system"]),
                }
            )

        _fill_user_roles(cur, users)

    return ok({"list": users, "total": total})


@router.get("/system/user/list")
def list_user_all(userIds: List[int] | None = None):
    """查询全部用户或指定 ID 列表，结构与 Java/Go 一致。"""
    base_sql = """
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '')       AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '')  AS create_user_name,
       u.update_time,
       COALESCE(uu.nickname, '')  AS update_user_name
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
"""
    params: List[Any] = []
    if userIds:
        base_sql += "WHERE u.id = ANY(%s::bigint[]) "
        params.append(userIds)
    base_sql += "ORDER BY u.id DESC;"

    with get_db_cursor() as cur:
        cur.execute(base_sql, params)
        rows = cur.fetchall()

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
                    "createUserString": r["create_user_name"],
                    "createTime": _format_time(r["create_time"]),
                    "updateUserString": r["update_user_name"],
                    "updateTime": _format_time(r["update_time"]),
                    "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
                    "deptName": r["dept_name"],
                    "roleIds": [],
                    "roleNames": [],
                    "disabled": bool(r["is_system"]),
                }
            )

        _fill_user_roles(cur, users)

    return ok(users)


def _fill_user_roles(cur, users: List[Dict[str, Any]]) -> None:
    """补充用户的角色 ID/名称信息。"""
    if not users:
        return
    user_ids = sorted({int(u["id"]) for u in users})
    cur.execute(
        """
SELECT ur.user_id,
       ur.role_id,
       r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY(%s::bigint[]);
""",
        (user_ids,),
    )
    rows = cur.fetchall()
    tmp: Dict[int, Dict[str, List[Any]]] = {}
    for r in rows:
        uid = int(r["user_id"])
        rid = int(r["role_id"])
        name = r["name"]
        entry = tmp.setdefault(uid, {"ids": [], "names": []})
        entry["ids"].append(rid)
        entry["names"].append(name)

    for u in users:
        uid = int(u["id"])
        info = tmp.get(uid)
        if info:
            u["roleIds"] = info["ids"]
            u["roleNames"] = info["names"]


@router.get("/system/user/{user_id}")
def get_user_detail(user_id: int):
    """查询单个用户详情，返回 UserDetailResp 结构。"""
    if user_id <= 0:
        return fail("400", "ID 参数不正确")

    sql = """
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, '') AS avatar,
       u.gender,
       COALESCE(u.email, '') AS email,
       COALESCE(u.phone, '') AS phone,
       COALESCE(u.description, '') AS description,
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '')       AS dept_name,
       u.pwd_reset_time,
       u.create_time,
       COALESCE(cu.nickname, '')  AS create_user_name,
       u.update_time,
       COALESCE(uu.nickname, '')  AS update_user_name
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (user_id,))
        r = cur.fetchone()
        if not r:
            return fail("404", "用户不存在")

        pwd_reset = r["pwd_reset_time"]
        user: Dict[str, Any] = {
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
            "createUserString": r["create_user_name"],
            "createTime": _format_time(r["create_time"]),
            "updateUserString": r["update_user_name"],
            "updateTime": _format_time(r["update_time"]),
            "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
            "deptName": r["dept_name"],
            "roleIds": [],
            "roleNames": [],
            "disabled": bool(r["is_system"]),
            "pwdResetTime": _format_time(pwd_reset),
        }

        _fill_user_roles(cur, [user])

    return ok(user)


@router.post("/system/user")
def create_user(
    req: UserReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增用户，逻辑与 Go/Java 保持一致（含 RSA 解密与密码强度校验）。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    username = (req.username or "").strip()
    nickname = (req.nickname or "").strip()
    if not username or not nickname:
        return fail("400", "用户名和昵称不能为空")
    if not req.deptId:
        return fail("400", "所属部门不能为空")
    if not (req.password or "").strip():
        return fail("400", "密码不能为空")

    # RSA 解密密码并进行强度校验
    decryptor = _get_rsa_decryptor()
    try:
        raw_pwd = decryptor.decrypt_base64(req.password)
    except Exception:
        return fail("400", "密码解密失败")

    if not (8 <= len(raw_pwd) <= 32):
        return fail("400", "密码长度为 8-32 个字符，至少包含字母和数字")
    has_letter = any(ch.isalpha() for ch in raw_pwd)
    has_digit = any(ch.isdigit() for ch in raw_pwd)
    if not (has_letter and has_digit):
        return fail("400", "密码长度为 8-32 个字符，至少包含字母和数字")

    encoded_pwd = PasswordHasher.hash(raw_pwd)

    with get_db_cursor() as cur:
        new_id = next_id()
        cur.execute(
            """
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s, %s, %s,
    %s, %s, FALSE, NOW(), %s,
    %s, NOW()
);
""",
            (
                new_id,
                username,
                nickname,
                req.gender,
                req.email,
                req.phone,
                req.avatar,
                req.description,
                req.status or 1,
                req.deptId,
                current_uid,
            ),
        )

        if req.roleIds:
            for rid in req.roleIds:
                cur.execute(
                    """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                    (next_id(), new_id, rid),
                )

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
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    username = (req.username or "").strip()
    nickname = (req.nickname or "").strip()
    if not username or not nickname:
        return fail("400", "用户名和昵称不能为空")
    if not req.deptId:
        return fail("400", "所属部门不能为空")

    with get_db_cursor() as cur:
        cur.execute(
            """
UPDATE sys_user
   SET username    = %s,
       nickname    = %s,
       gender      = %s,
       email       = %s,
       phone       = %s,
       avatar      = %s,
       description = %s,
       status      = %s,
       dept_id     = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
            (
                username,
                nickname,
                req.gender,
                req.email,
                req.phone,
                req.avatar,
                req.description,
                req.status or 1,
                req.deptId,
                current_uid,
                user_id,
            ),
        )

        # 先清空原有角色，再写入新角色
        cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (user_id,))
        for rid in req.roleIds or []:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), user_id, rid),
            )

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
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    if not ids:
        return fail("400", "ID 列表不能为空")

    with get_db_cursor() as cur:
        for uid in ids:
            cur.execute(
                "SELECT is_system FROM sys_user WHERE id = %s;", (uid,)
            )
            row = cur.fetchone()
            if not row:
                continue
            if row["is_system"]:
                continue
            cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (uid,))
            cur.execute("DELETE FROM sys_user WHERE id = %s;", (uid,))

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
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    if not (req.newPassword or "").strip():
        return fail("400", "密码不能为空")

    decryptor = _get_rsa_decryptor()
    try:
        raw_pwd = decryptor.decrypt_base64(req.newPassword)
    except Exception:
        return fail("400", "密码解密失败")

    if not (8 <= len(raw_pwd) <= 32):
        return fail("400", "密码长度为 8-32 个字符，至少包含字母和数字")
    has_letter = any(ch.isalpha() for ch in raw_pwd)
    has_digit = any(ch.isdigit() for ch in raw_pwd)
    if not (has_letter and has_digit):
        return fail("400", "密码长度为 8-32 个字符，至少包含字母和数字")

    encoded_pwd = PasswordHasher.hash(raw_pwd)

    with get_db_cursor() as cur:
        cur.execute(
            """
UPDATE sys_user
   SET password      = %s,
       pwd_reset_time= NOW(),
       update_user   = %s,
       update_time   = NOW()
 WHERE id            = %s;
""",
            (encoded_pwd, current_uid, user_id),
        )

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
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    with get_db_cursor() as cur:
        cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (user_id,))
        for rid in req.roleIds or []:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), user_id, rid),
            )

    return ok(True)


@router.get("/system/user/export")
def export_user():
    """
    导出用户为 CSV 文件。

    说明：与 Go 版一致，直接返回文件流，不包裹在统一响应结构中。
    """
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT username,
       nickname,
       gender,
       COALESCE(email, '') AS email,
       COALESCE(phone, '') AS phone
FROM sys_user
ORDER BY id ASC;
"""
        )
        rows = cur.fetchall()

    lines = ["username,nickname,gender,email,phone"]
    for r in rows:
        line = f'{r["username"]},{r["nickname"]},{int(r["gender"])},{r["email"]},{r["phone"]}'
        lines.append(line)
    content = "\n".join(lines) + "\n"

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

