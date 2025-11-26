from __future__ import annotations

from typing import Any, Dict, List, Optional, Sequence

from ..db import get_db_cursor
from ..id_generator import next_id


def list_users_page(
    page: int,
    size: int,
    description: str | None,
    status: int | None,
    dept_id: int | None,
) -> Dict[str, Any]:
    """分页查询用户列表数据，不包含角色信息。"""
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
    if dept_id and dept_id > 0:
        where_clauses.append("u.dept_id = %s")
        params.append(dept_id)

    where_sql = " WHERE " + " AND ".join(where_clauses)

    with get_db_cursor() as cur:
        cur.execute("SELECT COUNT(*) AS cnt FROM sys_user AS u" + where_sql, params)
        row = cur.fetchone()
        total = int(row["cnt"]) if row and row["cnt"] is not None else 0
        if total == 0:
            return {"list": [], "total": 0}

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

    return {"list": rows, "total": total}


def list_users(user_ids: Optional[Sequence[int]] = None) -> List[Dict[str, Any]]:
    """查询全部用户或指定 ID 列表。"""
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
    if user_ids:
        base_sql += "WHERE u.id = ANY(%s::bigint[]) "
        params.append(list(user_ids))
    base_sql += "ORDER BY u.id DESC;"

    with get_db_cursor() as cur:
        cur.execute(base_sql, params)
        rows = cur.fetchall()

    return rows


def get_user_detail_row(user_id: int) -> Optional[Dict[str, Any]]:
    """查询单个用户详情原始行数据。"""
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
        row = cur.fetchone()
    return row


def fill_user_roles(cur, users: List[Dict[str, Any]]) -> None:
    """补充用户列表中的角色 ID/名称信息。"""
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


def create_user_row(
    username: str,
    nickname: str,
    encoded_pwd: str,
    gender: int,
    email: str | None,
    phone: str | None,
    avatar: str | None,
    description: str | None,
    status: int,
    dept_id: int,
    create_user: int,
    role_ids: Sequence[int] | None,
) -> int:
    """插入用户及角色关系，返回新用户 ID。"""
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
                encoded_pwd,
                gender,
                email,
                phone,
                avatar,
                description,
                status,
                dept_id,
                create_user,
            ),
        )

        if role_ids:
            for rid in role_ids:
                cur.execute(
                    """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                    (next_id(), new_id, rid),
                )

    return new_id


def update_user_row(
    user_id: int,
    username: str,
    nickname: str,
    gender: int,
    email: str | None,
    phone: str | None,
    avatar: str | None,
    description: str | None,
    status: int,
    dept_id: int,
    update_user: int,
    role_ids: Sequence[int] | None,
) -> None:
    """更新用户及角色绑定。"""
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
                gender,
                email,
                phone,
                avatar,
                description,
                status,
                dept_id,
                update_user,
                user_id,
            ),
        )

        cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (user_id,))
        for rid in role_ids or []:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), user_id, rid),
            )


def delete_users_soft(ids: Sequence[int]) -> None:
    """删除非系统用户及其角色关联。"""
    if not ids:
        return
    with get_db_cursor() as cur:
        for uid in ids:
            cur.execute("SELECT is_system FROM sys_user WHERE id = %s;", (uid,))
            row = cur.fetchone()
            if not row:
                continue
            if row["is_system"]:
                continue
            cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (uid,))
            cur.execute("DELETE FROM sys_user WHERE id = %s;", (uid,))


def reset_user_password(user_id: int, encoded_pwd: str, update_user: int) -> None:
    """重置用户密码。"""
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
            (encoded_pwd, update_user, user_id),
        )


def update_user_roles_only(user_id: int, role_ids: Sequence[int] | None) -> None:
    """仅更新用户角色绑定。"""
    with get_db_cursor() as cur:
        cur.execute("DELETE FROM sys_user_role WHERE user_id = %s;", (user_id,))
        for rid in role_ids or []:
            cur.execute(
                """
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (%s, %s, %s)
ON CONFLICT (user_id, role_id) DO NOTHING;
""",
                (next_id(), user_id, rid),
            )


def export_user_rows() -> List[Dict[str, Any]]:
    """导出用户 CSV 所需行数据。"""
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
    return rows

