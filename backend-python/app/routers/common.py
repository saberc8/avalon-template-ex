from typing import List

from fastapi import APIRouter

from ..api_response import fail, ok
from ..db import get_db_cursor
from ..models.common import DeptTreeNode, LabelValue, MenuTreeNode

router = APIRouter()


@router.get("/common/dict/option/site")
def list_site_options():
    """网站基础配置字典数据，来源于 sys_option SITE 类别。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT code,
       COALESCE(value, default_value) AS value
FROM sys_option
WHERE category = 'SITE'
ORDER BY id ASC;
"""
        )
        rows = cur.fetchall()

    data = [LabelValue(label=r["code"], value=r["value"]).dict() for r in rows]
    return ok(data)


@router.get("/common/tree/menu")
def list_menu_tree():
    """菜单树，仅包含目录/菜单（type in (1,2)）。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id, title, parent_id, sort, status
FROM sys_menu
WHERE type IN (1, 2)
ORDER BY sort ASC, id ASC;
"""
        )
        rows = cur.fetchall()

    class MenuRow:
        def __init__(self, r: dict):
            self.id = int(r["id"])
            self.title = r["title"]
            self.parent_id = int(r["parent_id"])
            self.sort = int(r["sort"])
            self.status = int(r["status"])

    flat = [MenuRow(r) for r in rows]
    if not flat:
        return ok([])

    node_map: dict[int, MenuTreeNode] = {}
    for m in flat:
        node_map[m.id] = MenuTreeNode(
            key=m.id,
            title=m.title,
            disabled=m.status != 1,
            children=[],
        )

    roots: List[MenuTreeNode] = []
    for m in flat:
        node = node_map[m.id]
        if m.parent_id == 0:
            roots.append(node)
            continue
        parent = node_map.get(m.parent_id)
        if not parent:
            roots.append(node)
            continue
        parent.children.append(node)

    return ok([r.dict() for r in roots])


@router.get("/common/tree/dept")
def list_dept_tree():
    """部门树，兼容前端 TreeNodeData。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id, name, parent_id, sort, status, is_system
FROM sys_dept
ORDER BY sort ASC, id ASC;
"""
        )
        rows = cur.fetchall()

    class DeptRow:
        def __init__(self, r: dict):
            self.id = int(r["id"])
            self.name = r["name"]
            self.parent_id = int(r["parent_id"])
            self.sort = int(r["sort"])
            self.status = int(r["status"])
            self.is_system = bool(r["is_system"])

    flat = [DeptRow(r) for r in rows]
    if not flat:
        return ok([])

    node_map: dict[int, DeptTreeNode] = {}
    for d in flat:
        node_map[d.id] = DeptTreeNode(
            key=d.id,
            title=d.name,
            disabled=False,
            children=[],
        )

    roots: List[DeptTreeNode] = []
    for d in flat:
        node = node_map[d.id]
        if d.parent_id == 0:
            roots.append(node)
            continue
        parent = node_map.get(d.parent_id)
        if not parent:
            roots.append(node)
            continue
        parent.children.append(node)

    return ok([r.dict() for r in roots])


@router.get("/common/dict/user")
def list_user_dict(status: int | None = None):
    """用户字典：label=昵称/用户名，value=用户ID，extra=用户名。"""
    base_sql = """
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '')           AS username
FROM sys_user
WHERE status = 1
"""
    params: list = []
    if status and status > 0:
        base_sql = """
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '')           AS username
FROM sys_user
WHERE status = %s
"""
        params.append(status)
    base_sql += " ORDER BY id ASC;"

    with get_db_cursor() as cur:
        cur.execute(base_sql, params)
        rows = cur.fetchall()

    data = [
        LabelValue(label=r["nickname"], value=int(r["id"]), extra=r["username"]).dict()
        for r in rows
    ]
    return ok(data)


@router.get("/common/dict/role")
def list_role_dict():
    """角色字典：label=角色名，value=角色ID，extra=角色编码。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id, name, code
FROM sys_role
ORDER BY sort ASC, id ASC;
"""
        )
        rows = cur.fetchall()

    data = [
        LabelValue(label=r["name"], value=int(r["id"]), extra=r["code"]).dict()
        for r in rows
    ]
    return ok(data)


@router.get("/common/dict/{code}")
def list_dict_by_code(code: str):
    """通用字典项查询，根据 sys_dict / sys_dict_item。"""
    code = (code or "").strip()
    if not code:
        return ok([])

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT t1.label,
       t1.value,
       COALESCE(t1.color, '') AS extra
FROM sys_dict_item AS t1
LEFT JOIN sys_dict AS t2 ON t1.dict_id = t2.id
WHERE t1.status = 1
  AND t2.code = %s
ORDER BY t1.sort ASC, t1.id ASC;
""",
            (code,),
        )
        rows = cur.fetchall()

    data = [LabelValue(label=r["label"], value=r["value"], extra=r["extra"]).dict() for r in rows]
    return ok(data)


