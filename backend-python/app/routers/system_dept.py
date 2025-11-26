from __future__ import annotations

from typing import List, Optional

from fastapi import APIRouter, Body, Header, Path, Query
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..config import get_settings
from ..db import get_db_cursor
from ..security.jwt_token import TokenService

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _current_user_id(authorization: str | None) -> Optional[int]:
    """从 Authorization 头解析当前登录用户 ID，未授权返回 None。"""
    if not authorization:
        return None
    token_svc = _get_token_service()
    claims = token_svc.parse(authorization)
    if not claims:
        return None
    return claims.user_id


class DeptTreeQuery(BaseModel):
    """部门树查询条件。"""

    description: str | None = None
    status: str | None = None


class DeptResp(BaseModel):
    """部门响应结构，与前端 DeptResp 对齐。"""

    id: int
    name: str
    sort: int
    status: int
    isSystem: bool
    description: str
    createUserString: str
    createTime: str
    updateUserString: str
    updateTime: str
    parentId: int
    children: List["DeptResp"] = []


DeptResp.update_forward_refs()


class DeptReq(BaseModel):
    """创建/修改部门请求体。"""

    name: str = Field(..., description="部门名称")
    parentId: int = Field(..., description="上级部门 ID")
    sort: int = Field(1, description="排序")
    status: int = Field(1, description="状态 1=启用 2=禁用")
    description: str = Field("", description="描述")


class DeleteDeptReq(BaseModel):
    """删除部门请求体。"""

    ids: List[int]


def _format_time(dt) -> str:
    """统一时间格式化。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


@router.get("/system/dept/tree")
def list_dept_tree(
    description: str | None = Query(default=None),
    status: str | None = Query(default=None),
):
    """部门树：GET /system/dept/tree。"""
    desc = (description or "").strip()
    status_val = 0
    if status:
        try:
            v = int(status.strip())
            if v > 0:
                status_val = v
        except ValueError:
            status_val = 0

    where = ["1=1"]
    params: list = []
    if desc:
        where.append("(d.name ILIKE %s OR COALESCE(d.description,'') ILIKE %s)")
        like = f"%{desc}%"
        params.extend([like, like])
    if status_val:
        where.append("d.status = %s")
        params.append(status_val)
    where_sql = " WHERE " + " AND ".join(where)

    sql = f"""
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, '') AS description,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
{where_sql}
ORDER BY d.sort ASC, d.id ASC;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, params)
        rows = cur.fetchall()

    if not rows:
        return ok([])

    node_map: dict[int, DeptResp] = {}
    for r in rows:
        node = DeptResp(
            id=int(r["id"]),
            name=r["name"],
            sort=int(r["sort"]),
            status=int(r["status"]),
            isSystem=bool(r["is_system"]),
            description=r["description"],
            createUserString=r["create_user_string"],
            createTime=_format_time(r["create_time"]),
            updateUserString=r["update_user_string"],
            updateTime=_format_time(r["update_time"]),
            parentId=int(r["parent_id"]),
            children=[],
        )
        node_map[node.id] = node

    roots: List[DeptResp] = []
    for node in node_map.values():
        if node.parentId == 0:
            roots.append(node)
            continue
        parent = node_map.get(node.parentId)
        if not parent:
            roots.append(node)
            continue
        parent.children.append(node)

    return ok([d.dict() for d in roots])


@router.get("/system/dept/{dept_id}")
def get_dept(dept_id: int = Path(..., alias="dept_id")):
    """获取部门详情：GET /system/dept/{id}。"""
    if dept_id <= 0:
        return fail("400", "无效的部门 ID")

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, '') AS description,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = %s
LIMIT 1;
""",
            (dept_id,),
        )
        r = cur.fetchone()

    if not r:
        return fail("404", "部门不存在")

    resp = DeptResp(
        id=int(r["id"]),
        name=r["name"],
        sort=int(r["sort"]),
        status=int(r["status"]),
        isSystem=bool(r["is_system"]),
        description=r["description"],
        createUserString=r["create_user_string"],
        createTime=_format_time(r["create_time"]),
        updateUserString=r["update_user_string"],
        updateTime=_format_time(r["update_time"]),
        parentId=int(r["parent_id"]),
        children=[],
    )
    return ok(resp.dict())


@router.post("/system/dept")
def create_dept(
    body: DeptReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """新增部门：POST /system/dept。"""
    name = (body.name or "").strip()
    if not name:
        return fail("400", "名称不能为空")
    if not body.parentId:
        return fail("400", "上级部门不能为空")

    sort = body.sort if body.sort > 0 else 1
    status = body.status or 1

    with get_db_cursor() as cur:
        # 名称唯一性
        cur.execute(
            """
SELECT EXISTS(
  SELECT 1 FROM sys_dept WHERE name = %s AND parent_id = %s
) AS exists;
""",
            (name, body.parentId),
        )
        row = cur.fetchone()
        if row and row["exists"]:
            return fail("400", "新增失败，该名称在当前上级下已存在")

        # 上级是否存在
        cur.execute(
            "SELECT EXISTS(SELECT 1 FROM sys_dept WHERE id = %s) AS exists;",
            (body.parentId,),
        )
        parent_row = cur.fetchone()
        if not parent_row or not parent_row["exists"]:
            return fail("400", "上级部门不存在")

    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
INSERT INTO sys_dept (
    id, name, parent_id, sort, status, is_system, description,
    create_user, create_time
) VALUES (
    nextval('sys_dept_id_seq'), %s, %s, %s, %s, FALSE, %s,
    %s, NOW()
);
""",
                (
                    name,
                    body.parentId,
                    sort,
                    status,
                    (body.description or "").strip(),
                    current_uid,
                ),
            )
        except Exception:
            return fail("500", "新增部门失败")

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

    name = (body.name or "").strip()
    if not name:
        return fail("400", "名称不能为空")
    if not body.parentId:
        return fail("400", "上级部门不能为空")
    sort = body.sort if body.sort > 0 else 1
    status = body.status or 1

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id, name, parent_id, status, is_system
FROM sys_dept
WHERE id = %s;
""",
            (dept_id,),
        )
        old = cur.fetchone()
        if not old:
            return fail("404", "部门不存在")

        if old["is_system"]:
            if status == 2:
                return fail("400", f"[{old['name']}] 是系统内置部门，不允许禁用")
            if int(body.parentId) != int(old["parent_id"]):
                return fail("400", f"[{old['name']}] 是系统内置部门，不允许变更上级部门")

        # 名称唯一性（排除自身）
        cur.execute(
            """
SELECT EXISTS(
  SELECT 1 FROM sys_dept
  WHERE name = %s AND parent_id = %s AND id <> %s
) AS exists;
""",
            (name, body.parentId, dept_id),
        )
        dup = cur.fetchone()
        if dup and dup["exists"]:
            return fail("400", "修改失败，该名称在当前上级下已存在")

    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
UPDATE sys_dept
   SET name = %s,
       parent_id = %s,
       sort = %s,
       status = %s,
       description = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id = %s;
""",
                (
                    name,
                    body.parentId,
                    sort,
                    status,
                    (body.description or "").strip(),
                    current_uid,
                    dept_id,
                ),
            )
        except Exception:
            return fail("500", "修改部门失败")

    return ok(True)


@router.delete("/system/dept")
def delete_dept(body: DeleteDeptReq = Body(...)):
    """删除部门：DELETE /system/dept。"""
    ids = [i for i in (body.ids or []) if i and i > 0]
    if not ids:
        return fail("400", "参数错误")

    with get_db_cursor() as cur:
        ids_tuple = tuple(ids)

        # 系统内置校验
        cur.execute(
            """
SELECT name
FROM sys_dept
WHERE id = ANY(%s) AND is_system = TRUE
LIMIT 1;
""",
            (list(ids),),
        )
        row = cur.fetchone()
        if row:
            return fail(
                "400",
                f"所选部门 [{row['name']}] 是系统内置部门，不允许删除",
            )

        # 子部门校验
        cur.execute(
            """
SELECT EXISTS(
  SELECT 1 FROM sys_dept WHERE parent_id = ANY(%s)
) AS exists;
""",
            (list(ids),),
        )
        child = cur.fetchone()
        if child and child["exists"]:
            return fail("400", "所选部门存在下级部门，不允许删除")

        # 用户关联校验
        cur.execute(
            """
SELECT EXISTS(
  SELECT 1 FROM sys_user WHERE dept_id = ANY(%s)
) AS exists;
""",
            (list(ids),),
        )
        user = cur.fetchone()
        if user and user["exists"]:
            return fail("400", "所选部门存在用户关联，请解除关联后重试")

        # 删除角色部门关系
        cur.execute(
            "DELETE FROM sys_role_dept WHERE dept_id = ANY(%s);",
            (list(ids),),
        )
        # 删除部门
        cur.execute(
            "DELETE FROM sys_dept WHERE id = ANY(%s);",
            (list(ids),),
        )

    return ok(True)


@router.get("/system/dept/export")
def export_dept(
    description: str | None = Query(default=None),
    status: str | None = Query(default=None),
):
    """导出部门 CSV：GET /system/dept/export。"""
    desc = (description or "").strip()
    status_val = 0
    if status:
        try:
            v = int(status.strip())
            if v > 0:
                status_val = v
        except ValueError:
            status_val = 0

    where = ["1=1"]
    params: list = []
    if desc:
        where.append("(d.name ILIKE %s OR COALESCE(d.description,'') ILIKE %s)")
        like = f"%{desc}%"
        params.extend([like, like])
    if status_val:
        where.append("d.status = %s")
        params.append(status_val)
    where_sql = " WHERE " + " AND ".join(where)

    sql = f"""
SELECT d.id,
       d.name,
       d.parent_id,
       d.status,
       d.sort,
       d.is_system,
       COALESCE(d.description, '') AS description,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
{where_sql}
ORDER BY d.sort ASC, d.id ASC;
"""

    with get_db_cursor() as cur:
        cur.execute(sql, params)
        rows = cur.fetchall()

    header = "ID,名称,上级部门ID,状态,排序,系统内置,描述,创建时间,创建人,修改时间,修改人"
    lines = [header]
    for r in rows:
        line = ",".join(
            [
                str(r["id"]),
                str(r["name"]),
                str(r["parent_id"]),
                str(r["status"]),
                str(r["sort"]),
                "true" if r["is_system"] else "false",
                r["description"],
                _format_time(r["create_time"]),
                r["create_user_string"],
                _format_time(r["update_time"]),
                r["update_user_string"],
            ]
        )
        lines.append(line)

    return "\n".join(lines)

