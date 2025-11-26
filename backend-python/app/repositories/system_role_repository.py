from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional, Sequence

from ..db import get_db_cursor
from ..id_generator import next_id


def list_roles() -> List[Dict[str, Any]]:
    """查询全部角色行数据。"""
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
    with get_db_cursor() as cur:
        cur.execute(sql)
        rows = cur.fetchall()
    return rows


def get_role_row(role_id: int) -> Optional[Dict[str, Any]]:
    """查询单个角色详情行数据。"""
    sql = """
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999)                 AS sort,
       COALESCE(r.description, '')           AS description,
       COALESCE(r.data_scope, 4)             AS data_scope,
       COALESCE(r.is_system, FALSE)          AS is_system,
       COALESCE(r.menu_check_strictly, TRUE) AS menu_check_strictly,
       COALESCE(r.dept_check_strictly, TRUE) AS dept_check_strictly,
       r.create_time,
       COALESCE(cu.nickname, '')             AS create_user_name,
       r.update_time,
       COALESCE(uu.nickname, '')             AS update_user_name
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (role_id,))
        row = cur.fetchone()
    return row


def get_role_menu_ids(role_id: int) -> List[int]:
    """查询角色绑定的菜单 ID 列表。"""
    with get_db_cursor() as cur:
        cur.execute(
            "SELECT menu_id FROM sys_role_menu WHERE role_id = %s;",
            (role_id,),
        )
        rows = cur.fetchall()
    return [int(x["menu_id"]) for x in rows]


def get_role_dept_ids(role_id: int) -> List[int]:
    """查询角色绑定的部门 ID 列表。"""
    with get_db_cursor() as cur:
        cur.execute(
            "SELECT dept_id FROM sys_role_dept WHERE role_id = %s;",
            (role_id,),
        )
        rows = cur.fetchall()
    return [int(x["dept_id"]) for x in rows]


def create_role_row(
    name: str,
    code: str,
    sort: int,
    description: str | None,
    data_scope: int,
    dept_check_strict: bool,
    dept_ids: Iterable[int],
    create_user: int,
) -> int:
    """插入角色及其部门数据权限配置，返回新角色 ID。"""
    new_id = next_id()
    with get_db_cursor() as cur:
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
                description,
                sort,
                dept_check_strict,
                create_user,
            ),
        )

        for did in dept_ids or []:
            cur.execute(
                """
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (%s, %s)
ON CONFLICT DO NOTHING;
""",
                (new_id, did),
            )
    return new_id


def update_role_row(
    role_id: int,
    name: str,
    description: str | None,
    sort: int,
    data_scope: int,
    dept_check_strict: bool,
    dept_ids: Iterable[int],
    update_user: int,
) -> None:
    """更新角色基础信息及部门数据权限配置。"""
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
                description,
                sort,
                data_scope,
                dept_check_strict,
                update_user,
                role_id,
            ),
        )

        cur.execute("DELETE FROM sys_role_dept WHERE role_id = %s;", (role_id,))
        for did in dept_ids or []:
            cur.execute(
                """
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (%s, %s)
ON CONFLICT DO NOTHING;
""",
                (role_id, did),
            )


def delete_roles(ids: Sequence[int]) -> None:
    """删除指定角色并清理关联记录，跳过系统内置 admin 角色。"""
    if not ids:
        return
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
                continue
            cur.execute("DELETE FROM sys_role_menu WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_role_dept WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_user_role WHERE role_id = %s;", (rid,))
            cur.execute("DELETE FROM sys_role WHERE id = %s;", (rid,))


def update_role_permission_row(
    role_id: int,
    menu_ids: Iterable[int],
    menu_check_strictly: bool,
    update_user: int,
) -> None:
    """更新角色菜单权限与严格模式配置。"""
    with get_db_cursor() as cur:
        cur.execute("DELETE FROM sys_role_menu WHERE role_id = %s;", (role_id,))
        for mid in menu_ids or []:
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
            (bool(menu_check_strictly), update_user, role_id),
        )


def list_role_users_rows(role_id: int) -> List[Dict[str, Any]]:
    """查询角色关联用户的明细行数据。"""
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
    return rows


def fill_role_user_roles(items: List[Dict[str, Any]]) -> None:
    """补充角色关联用户的全部角色 ID/名称信息。"""
    if not items:
        return
    user_ids = sorted({int(x["userId"]) for x in items})
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

    for item in items:
        uid = int(item["userId"])
        info = tmp.get(uid)
        if info:
            item["roleIds"] = info["ids"]
            item["roleNames"] = info["names"]


def assign_role_to_users(role_id: int, user_ids: Iterable[int]) -> None:
    """为角色分配用户。"""
    with get_db_cursor() as cur:
        for uid in user_ids:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), uid, role_id),
            )


def delete_user_roles_by_ids(ids: Iterable[int]) -> None:
    """按 sys_user_role.ID 删除用户角色关联。"""
    ids = [i for i in ids if i > 0]
    if not ids:
        return
    with get_db_cursor() as cur:
        cur.execute(
            "DELETE FROM sys_user_role WHERE id = ANY(%s::bigint[]);",
            (ids,),
        )


def list_role_user_ids(role_id: int) -> List[int]:
    """查询角色下所有用户 ID 列表。"""
    with get_db_cursor() as cur:
        cur.execute(
            "SELECT user_id FROM sys_user_role WHERE role_id = %s;",
            (role_id,),
        )
        rows = cur.fetchall()
    return [int(r["user_id"]) for r in rows]

