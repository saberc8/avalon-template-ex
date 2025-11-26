from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional, Sequence

from ..db import get_db_cursor
from ..id_generator import next_id


def list_menus_for_tree() -> List[Dict[str, Any]]:
    """查询构建菜单树所需的全部菜单行数据。"""
    sql = """
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
ORDER BY m.sort ASC, m.id ASC;
"""
    with get_db_cursor() as cur:
        cur.execute(sql)
        rows = cur.fetchall()
    return rows


def get_menu_row(menu_id: int) -> Optional[Dict[str, Any]]:
    """根据 ID 查询单个菜单行数据。"""
    sql = """
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
WHERE m.id = %s
LIMIT 1;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (menu_id,))
        row = cur.fetchone()
    return row


def insert_menu_row(
    title: str,
    parent_id: int,
    type_: int,
    path: str,
    name: str,
    component: str,
    redirect: str,
    icon: str,
    is_external: bool,
    is_cache: bool,
    is_hidden: bool,
    permission: str,
    sort: int,
    status: int,
    create_user: int,
) -> int:
    """插入菜单记录，返回新菜单 ID。"""
    new_id = next_id()
    with get_db_cursor() as cur:
        cur.execute(
            """
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect,
    icon, is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s, %s, %s,
    %s, %s, %s, %s, %s, %s, %s,
    %s, NOW()
);
""",
            (
                new_id,
                title,
                parent_id,
                type_,
                path,
                name,
                component,
                redirect,
                icon,
                is_external,
                is_cache,
                is_hidden,
                permission,
                sort,
                status,
                create_user,
            ),
        )
    return new_id


def update_menu_row(
    menu_id: int,
    title: str,
    parent_id: int,
    type_: int,
    path: str,
    name: str,
    component: str,
    redirect: str,
    icon: str,
    is_external: bool,
    is_cache: bool,
    is_hidden: bool,
    permission: str,
    sort: int,
    status: int,
    update_user: int,
) -> None:
    """根据 ID 更新菜单记录。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
UPDATE sys_menu
   SET title       = %s,
       parent_id   = %s,
       type        = %s,
       path        = %s,
       name        = %s,
       component   = %s,
       redirect    = %s,
       icon        = %s,
       is_external = %s,
       is_cache    = %s,
       is_hidden   = %s,
       permission  = %s,
       sort        = %s,
       status      = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
            (
                title,
                parent_id,
                type_,
                path,
                name,
                component,
                redirect,
                icon,
                is_external,
                is_cache,
                is_hidden,
                permission,
                sort,
                status,
                update_user,
                menu_id,
            ),
        )


def list_menu_parent_relations() -> List[Dict[str, Any]]:
    """查询全部菜单的 ID 与父子关系。"""
    with get_db_cursor() as cur:
        cur.execute("SELECT id, parent_id FROM sys_menu;")
        rows = cur.fetchall()
    return rows


def delete_menus_with_ids(ids: Sequence[int]) -> None:
    """删除指定菜单及其在角色关联表中的记录。"""
    if not ids:
        return
    with get_db_cursor() as cur:
        cur.execute(
            "DELETE FROM sys_role_menu WHERE menu_id = ANY(%s::bigint[]);",
            (list(ids),),
        )
        cur.execute(
            "DELETE FROM sys_menu WHERE id = ANY(%s::bigint[]);",
            (list(ids),),
        )

