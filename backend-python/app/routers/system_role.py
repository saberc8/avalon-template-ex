from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Body, Header
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..config import get_settings
from ..db import get_db_cursor
from ..id_generator import next_id
from ..security.jwt_token import TokenService

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _current_user_id(authorization: str | None) -> int | None:
    """从 Authorization 头解析当前登录用户 ID。"""
    if not authorization:
        return None
    token_svc = _get_token_service()
    claims = token_svc.parse(authorization)
    if not claims:
        return None
    return claims.user_id


def _format_time(dt) -> str:
    """统一时间格式化：YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


class RoleReq(BaseModel):
    """创建/修改角色请求体，字段与前端 RoleReq 对齐。"""

    name: str = Field(..., description="角色名称")
    code: str | None = Field(None, description="角色编码（仅新增必填）")
    sort: int = Field(999, description="排序")
    description: str | None = Field(None, description="描述")
    dataScope: int = Field(4, description="数据范围")
    deptIds: List[int] = Field(default_factory=list, description="部门 ID 列表")
    deptCheckStrictly: bool = Field(True, description="部门数据权限是否严格模式")


class RolePermissionReq(BaseModel):
    """更新角色菜单权限请求体。"""

    menuIds: List[int] = Field(default_factory=list, description="菜单 ID 列表")
    menuCheckStrictly: bool = Field(True, description="菜单数据权限是否严格模式")


class IdsBody(BaseModel):
    """通用 ID 列表请求体。"""

    ids: List[int]


@router.get("/system/role/list")
def list_role(description: str | None = None):
    """查询角色列表，结构与前端 RoleResp[] 对齐。"""
    sql = """
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999)               AS sort,
       COALESCE(r.description, '')         AS description,
       COALESCE(r.data_scope, 4)           AS data_scope,
       COALESCE(r.is_system, FALSE)        AS is_system,
       r.create_time,
       COALESCE(cu.nickname, '')           AS create_user_name,
       r.update_time,
       COALESCE(uu.nickname, '')           AS update_user_name
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
ORDER BY r.sort ASC, r.id ASC;
"""
    desc = (description or "").strip()
    with get_db_cursor() as cur:
        cur.execute(sql)
        rows = cur.fetchall()

    data: List[Dict[str, Any]] = []
    for r in rows:
        name = r["name"] or ""
        desc_text = r["description"] or ""
        if desc and (desc not in name) and (desc not in desc_text):
            continue
        is_system = bool(r["is_system"])
        code = r["code"] or ""
        item = {
            "id": int(r["id"]),
            "name": name,
            "code": code,
            "sort": int(r["sort"]),
            "description": desc_text,
            "dataScope": int(r["data_scope"]),
            "isSystem": is_system,
            "createUserString": r["create_user_name"],
            "createTime": _format_time(r["create_time"]),
            "updateUserString": r["update_user_name"],
            "updateTime": _format_time(r["update_time"]),
            "disabled": is_system and code == "admin",
        }
        data.append(item)
    return ok(data)


@router.get("/system/role/{role_id}")
def get_role(role_id: int):
    """获取角色详情，返回 RoleDetailResp 结构。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")

    sql = """
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999)               AS sort,
       COALESCE(r.description, '')         AS description,
       COALESCE(r.data_scope, 4)           AS data_scope,
       COALESCE(r.is_system, FALSE)        AS is_system,
       COALESCE(r.menu_check_strictly, TRUE) AS menu_check_strictly,
       COALESCE(r.dept_check_strictly, TRUE) AS dept_check_strictly,
       r.create_time,
       COALESCE(cu.nickname, '')           AS create_user_name,
       r.update_time,
       COALESCE(uu.nickname, '')           AS update_user_name
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (role_id,))
        r = cur.fetchone()
        if not r:
            return fail("404", "角色不存在")

        is_system = bool(r["is_system"])
        code = r["code"] or ""
        base = {
            "id": int(r["id"]),
            "name": r["name"],
            "code": code,
            "sort": int(r["sort"]),
            "description": r["description"],
            "dataScope": int(r["data_scope"]),
            "isSystem": is_system,
            "createUserString": r["create_user_name"],
            "createTime": _format_time(r["create_time"]),
            "updateUserString": r["update_user_name"],
            "updateTime": _format_time(r["update_time"]),
            "disabled": is_system and code == "admin",
        }

        # 角色菜单 ID
        cur.execute(
            "SELECT menu_id FROM sys_role_menu WHERE role_id = %s;",
            (role_id,),
        )
        menu_ids = [int(x["menu_id"]) for x in cur.fetchall()]

        # 角色部门 ID
        cur.execute(
            "SELECT dept_id FROM sys_role_dept WHERE role_id = %s;",
            (role_id,),
        )
        dept_ids = [int(x["dept_id"]) for x in cur.fetchall()]

    resp = {
        **base,
        "menuIds": menu_ids,
        "deptIds": dept_ids,
        "menuCheckStrictly": bool(r["menu_check_strictly"]),
        "deptCheckStrictly": bool(r["dept_check_strictly"]),
    }
    return ok(resp)


@router.post("/system/role")
def create_role(
    req: RoleReq,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增角色。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    name = (req.name or "").strip()
    code = (req.code or "").strip()
    if not name or not code:
        return fail("400", "名称和编码不能为空")

    sort = req.sort or 999
    data_scope = req.dataScope or 4
    dept_check_strict = bool(req.deptCheckStrictly)

    with get_db_cursor() as cur:
        new_id = next_id()
        cur.execute(
            """
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s,
    FALSE, TRUE, %s,
    %s, NOW()
);
""",
            (
                new_id,
                name,
                code,
                data_scope,
                req.description,
                sort,
                dept_check_strict,
                current_uid,
            ),
        )

        for did in req.deptIds or []:
            cur.execute(
                """
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (%s, %s)
ON CONFLICT DO NOTHING;
""",
                (new_id, did),
            )

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

    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    name = (req.name or "").strip()
    if not name:
        return fail("400", "名称不能为空")

    sort = req.sort or 999
    data_scope = req.dataScope or 4
    dept_check_strict = bool(req.deptCheckStrictly)

    with get_db_cursor() as cur:
        cur.execute(
            """
UPDATE sys_role
   SET name                = %s,
       description         = %s,
       sort                = %s,
       data_scope          = %s,
       dept_check_strictly = %s,
       update_user         = %s,
       update_time         = NOW()
 WHERE id                  = %s;
""",
            (
                name,
                req.description,
                sort,
                data_scope,
                dept_check_strict,
                current_uid,
                role_id,
            ),
        )

        cur.execute("DELETE FROM sys_role_dept WHERE role_id = %s;", (role_id,))
        for did in req.deptIds or []:
            cur.execute(
                """
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (%s, %s)
ON CONFLICT DO NOTHING;
""",
                (role_id, did),
            )

    return ok(True)


@router.delete("/system/role")
def delete_role(
    body: IdsBody = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除角色（跳过系统内置角色）。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (body.ids or []) if i > 0]
    if not ids:
        return fail("400", "ID 列表不能为空")

    with get_db_cursor() as cur:
        for rid in ids:
            cur.execute(
                "SELECT is_system, code FROM sys_role WHERE id = %s;",
                (rid,),
            )
            row = cur.fetchone()
            if not row:
                continue
            if row["is_system"] and row["code"] == "admin":
                # 系统内置 admin 角色不允许删除
                continue
            cur.execute("DELETE FROM sys_role_menu WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_role_dept WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_user_role WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_role WHERE id = %s;", (rid,))

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
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    with get_db_cursor() as cur:
        cur.execute("DELETE FROM sys_role_menu WHERE role_id = %s;", (role_id,))
        for mid in req.menuIds or []:
            cur.execute(
                """
INSERT INTO sys_role_menu (role_id, menu_id)
VALUES (%s, %s)
ON CONFLICT DO NOTHING;
""",
                (role_id, mid),
            )

        cur.execute(
            """
UPDATE sys_role
   SET menu_check_strictly = %s,
       update_user         = %s,
       update_time         = NOW()
 WHERE id                  = %s;
""",
            (bool(req.menuCheckStrictly), current_uid, role_id),
        )

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
    if page <= 0:
        page = 1
    if size <= 0:
        size = 10

    desc = (description or "").strip()

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT ur.id,
       ur.role_id,
       u.id                 AS user_id,
       u.username,
       u.nickname,
       u.gender,
       u.status,
       u.is_system,
       COALESCE(u.description, '') AS description,
       u.dept_id,
       COALESCE(d.name, '')        AS dept_name
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE ur.role_id = %s
ORDER BY ur.id DESC;
""",
            (role_id,),
        )
        rows = cur.fetchall()

    all_items: List[Dict[str, Any]] = []
    for r in rows:
        username = r["username"]
        nickname = r["nickname"]
        desc_text = r["description"]
        if desc and (desc not in username) and (desc not in nickname) and (desc not in desc_text):
            continue
        item = {
            "id": int(r["id"]),
            "roleId": int(r["role_id"]),
            "userId": int(r["user_id"]),
            "username": username,
            "nickname": nickname,
            "gender": int(r["gender"]),
            "status": int(r["status"]),
            "isSystem": bool(r["is_system"]),
            "description": desc_text,
            "deptId": int(r["dept_id"]) if r["dept_id"] is not None else 0,
            "deptName": r["dept_name"],
            "roleIds": [],
            "roleNames": [],
            "disabled": False,
        }
        all_items.append(item)

    # 补充每个用户的全部角色信息
    if all_items:
        user_ids = sorted({int(x["userId"]) for x in all_items})
        with get_db_cursor() as cur:
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

        for item in all_items:
            uid = int(item["userId"])
            info = tmp.get(uid)
            if info:
                item["roleIds"] = info["ids"]
                item["roleNames"] = info["names"]
            # admin 角色下的系统用户禁用操作
            item["disabled"] = item["isSystem"] and item["roleId"] == 1

    total = len(all_items)
    start = (page - 1) * size
    if start > total:
        start = total
    end = start + size
    if end > total:
        end = total
    page_list = all_items[start:end]

    return ok({"list": page_list, "total": total})


@router.post("/system/role/{role_id}/user")
def assign_to_users(
    role_id: int,
    user_ids: List[int] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """为角色分配用户。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = [i for i in (user_ids or []) if i > 0]
    if not ids:
        return fail("400", "用户ID列表不能为空")

    with get_db_cursor() as cur:
        for uid in ids:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), uid, role_id),
            )

    return ok(True)


@router.delete("/system/role/user")
def unassign_from_users(
    ids: List[int] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """取消用户与角色的关联（按 user_role 记录 ID）。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    valid_ids = [i for i in (ids or []) if i > 0]
    if not valid_ids:
        return fail("400", "用户角色ID列表不能为空")

    with get_db_cursor() as cur:
        cur.execute(
            "DELETE FROM sys_user_role WHERE id = ANY(%s::bigint[]);",
            (valid_ids,),
        )

    return ok(True)


@router.get("/system/role/{role_id}/user/id")
def list_role_user_ids(role_id: int):
    """查询角色下所有用户 ID 列表。"""
    if role_id <= 0:
        return fail("400", "ID 参数不正确")

    with get_db_cursor() as cur:
        cur.execute(
            "SELECT user_id FROM sys_user_role WHERE role_id = %s;",
            (role_id,),
        )
        rows = cur.fetchall()

    ids = [int(r["user_id"]) for r in rows]
    return ok(ids)

