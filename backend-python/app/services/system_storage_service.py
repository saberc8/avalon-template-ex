from __future__ import annotations

from typing import Any, Dict, List

from pydantic import BaseModel, Field

from ..db import get_db_cursor
from ..id_generator import next_id


def _format_time(dt) -> str:
    """统一时间格式化：YYYY-MM-DD HH:MM:SS。"""
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


class StorageReq(BaseModel):
    """存储创建或修改请求体，对齐前端表单字段。"""

    name: str = Field(..., description="名称")
    code: str = Field(..., description="编码")
    type: int = Field(1, description="类型（1 本地；2 对象存储）")
    accessKey: str | None = Field(None, description="Access Key")
    secretKey: str | None = Field(None, description="Secret Key（加密后字符串）")
    endpoint: str | None = Field(None, description="Endpoint")
    region: str | None = Field(None, description="Region")
    bucketName: str = Field(..., description="Bucket/存储路径")
    domain: str | None = Field(None, description="域名/访问路径")
    sort: int = Field(999, description="排序")
    description: str | None = Field(None, description="描述")
    status: int = Field(1, description="状态（1 启用，2 禁用）")
    isDefault: bool = Field(False, description="是否为默认存储")


class StatusReq(BaseModel):
    """仅修改状态请求体。"""

    status: int = Field(..., description="状态（1 启用，2 禁用）")


def list_storage_service(
    description: str | None,
    type_value: int | None,
    sort: List[str] | None,
) -> List[Dict[str, Any]]:
    """查询存储列表服务实现。"""
    where: List[str] = ["1=1"]
    params: List[Any] = []

    desc_kw = (description or "").strip()
    if desc_kw:
        like = f"%{desc_kw}%"
        where.append(
            "(s.name ILIKE %s OR s.code ILIKE %s OR COALESCE(s.description,'') ILIKE %s)"
        )
        params.extend([like, like, like])

    if type_value and type_value > 0:
        where.append("s.type = %s")
        params.append(type_value)

    where_sql = " WHERE " + " AND ".join(where)

    order_by = "ORDER BY s.create_time DESC, s.id DESC"
    sort_values: List[str] = []
    for s in sort or []:
        if not s:
            continue
        for part in s.split(","):
            val = part.strip()
            if val:
                sort_values.append(val)
    if sort_values:
        field = sort_values[0]
        direction = sort_values[1] if len(sort_values) > 1 else "desc"
        column_map = {
            "createTime": "s.create_time",
            "id": "s.id",
            "sort": "s.sort",
        }
        col = column_map.get(field)
        if col:
            direction_sql = "DESC" if direction.lower() == "desc" else "ASC"
            order_by = f"ORDER BY {col} {direction_sql}"

    sql = f"""
SELECT s.id,
       s.name,
       s.code,
       COALESCE(s.type, 1)              AS type,
       COALESCE(s.access_key, '')       AS access_key,
       COALESCE(s.secret_key, '')       AS secret_key,
       COALESCE(s.endpoint, '')         AS endpoint,
       COALESCE(s.region, '')           AS region,
       COALESCE(s.bucket_name, '')      AS bucket_name,
       COALESCE(s.domain, '')           AS domain,
       COALESCE(s.description, '')      AS description,
       COALESCE(s.is_default, FALSE)    AS is_default,
       COALESCE(s.sort, 999)            AS sort,
       COALESCE(s.status, 1)            AS status,
       s.create_time,
       COALESCE(cu.nickname, '')        AS create_user_name,
       s.update_time,
       COALESCE(uu.nickname, '')        AS update_user_name
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
{where_sql}
{order_by};
"""
    with get_db_cursor() as cur:
        cur.execute(sql, params)
        rows = cur.fetchall()

    data: List[Dict[str, Any]] = []
    for r in rows:
        raw_secret = r["secret_key"]
        secret_mask = "********" if raw_secret else ""
        item = {
            "id": int(r["id"]),
            "name": r["name"],
            "code": r["code"],
            "type": int(r["type"]),
            "accessKey": r["access_key"],
            "secretKey": secret_mask,
            "endpoint": r["endpoint"],
            "region": r["region"],
            "bucketName": r["bucket_name"],
            "domain": r["domain"],
            "description": r["description"],
            "isDefault": bool(r["is_default"]),
            "sort": int(r["sort"]),
            "status": int(r["status"]),
            "createUserString": r["create_user_name"],
            "createTime": _format_time(r["create_time"]),
            "updateUserString": r["update_user_name"],
            "updateTime": _format_time(r["update_time"]),
        }
        data.append(item)

    return data


def get_storage_service(storage_id: int) -> Dict[str, Any] | None:
    """查询存储详情服务实现。"""
    sql = """
SELECT s.id,
       s.name,
       s.code,
       COALESCE(s.type, 1)              AS type,
       COALESCE(s.access_key, '')       AS access_key,
       COALESCE(s.secret_key, '')       AS secret_key,
       COALESCE(s.endpoint, '')         AS endpoint,
       COALESCE(s.region, '')           AS region,
       COALESCE(s.bucket_name, '')      AS bucket_name,
       COALESCE(s.domain, '')           AS domain,
       COALESCE(s.description, '')      AS description,
       COALESCE(s.is_default, FALSE)    AS is_default,
       COALESCE(s.sort, 999)            AS sort,
       COALESCE(s.status, 1)            AS status,
       s.create_time,
       COALESCE(cu.nickname, '')        AS create_user_name,
       s.update_time,
       COALESCE(uu.nickname, '')        AS update_user_name
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
WHERE s.id = %s;
"""
    with get_db_cursor() as cur:
        cur.execute(sql, (storage_id,))
        r = cur.fetchone()

    if not r:
        return None

    raw_secret = r["secret_key"]
    secret_mask = "********" if raw_secret else ""
    data = {
        "id": int(r["id"]),
        "name": r["name"],
        "code": r["code"],
        "type": int(r["type"]),
        "accessKey": r["access_key"],
        "secretKey": secret_mask,
        "endpoint": r["endpoint"],
        "region": r["region"],
        "bucketName": r["bucket_name"],
        "domain": r["domain"],
        "description": r["description"],
        "isDefault": bool(r["is_default"]),
        "sort": int(r["sort"]),
        "status": int(r["status"]),
        "createUserString": r["create_user_name"],
        "createTime": _format_time(r["create_time"]),
        "updateUserString": r["update_user_name"],
        "updateTime": _format_time(r["update_time"]),
    }
    return data


def create_storage_service(req: StorageReq, current_uid: int) -> int:
    """新增存储服务实现。"""
    name = (req.name or "").strip()
    code = (req.code or "").strip()
    if not name or not code:
        raise ValueError("名称和编码不能为空")

    t = req.type or 1
    bucket = (req.bucketName or "").strip()
    domain = (req.domain or "").strip() if req.domain is not None else ""

    if t == 1:
        if not bucket or not domain:
            raise ValueError("存储路径和访问路径不能为空")
    else:
        if not req.accessKey or not req.secretKey or not req.endpoint or not bucket:
            raise ValueError("对象存储配置不完整")

    status = req.status or 1
    sort = req.sort or 999
    is_default = bool(req.isDefault)

    new_id = next_id()

    with get_db_cursor() as cur:
        try:
            if is_default:
                cur.execute("UPDATE sys_storage SET is_default = FALSE;")

            cur.execute(
                """
INSERT INTO sys_storage (
    id, name, code, type,
    access_key, secret_key, endpoint, region,
    bucket_name, domain, description,
    is_default, sort, status,
    create_user, create_time
) VALUES (
    %s, %s, %s, %s,
    %s, %s, %s, %s,
    %s, %s, %s,
    %s, %s, %s,
    %s, NOW()
);
""",
                (
                    new_id,
                    name,
                    code,
                    t,
                    req.accessKey or "",
                    req.secretKey or "",
                    req.endpoint or "",
                    req.region or "",
                    bucket,
                    domain,
                    req.description or "",
                    is_default,
                    sort,
                    status,
                    current_uid,
                ),
            )
        except Exception as exc:  # noqa: BLE001
            raise RuntimeError("保存存储配置失败") from exc

    return new_id


def update_storage_service(
    storage_id: int,
    req: StorageReq,
    current_uid: int,
) -> None:
    """修改存储服务实现。"""
    name = (req.name or "").strip()
    if not name:
        raise ValueError("名称不能为空")

    t = req.type or 1
    bucket = (req.bucketName or "").strip()
    domain = (req.domain or "").strip() if req.domain is not None else ""

    if t == 1:
        if not bucket or not domain:
            raise ValueError("存储路径和访问路径不能为空")
    else:
        if not bucket:
            raise ValueError("Bucket 不能为空")

    status = req.status or 1
    sort = req.sort or 999

    with get_db_cursor() as cur:
        if req.isDefault:
            cur.execute("UPDATE sys_storage SET is_default = FALSE;")

        cur.execute(
            """
UPDATE sys_storage
   SET name        = %s,
       type        = %s,
       access_key  = %s,
       secret_key  = CASE WHEN %s IS NULL THEN secret_key ELSE %s END,
       endpoint    = %s,
       region      = %s,
       bucket_name = %s,
       domain      = %s,
       description = %s,
       is_default  = %s,
       sort        = %s,
       status      = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
            (
                name,
                t,
                req.accessKey or "",
                req.secretKey,
                req.secretKey or "",
                req.endpoint or "",
                req.region or "",
                bucket,
                domain,
                req.description or "",
                bool(req.isDefault),
                sort,
                status,
                current_uid,
                storage_id,
            ),
        )
        if cur.rowcount <= 0:
            raise ValueError("存储不存在")


def delete_storage_service(ids: List[int]) -> None:
    """删除存储服务实现。"""
    ids = [i for i in (ids or []) if i > 0]
    if not ids:
        raise ValueError("ID 列表不能为空")

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id
FROM sys_storage
WHERE id = ANY(%s::bigint[]) AND is_default = TRUE;
""",
            (ids,),
        )
        default_rows = cur.fetchall()
        default_ids = {int(r["id"]) for r in default_rows}
        if default_ids:
            raise ValueError("不允许删除默认存储，请先取消默认")

        cur.execute(
            """
DELETE FROM sys_storage
 WHERE id = ANY(%s::bigint[]);
""",
            (ids,),
        )


def update_storage_status_service(
    storage_id: int,
    req: StatusReq,
    current_uid: int,
) -> None:
    """修改存储状态服务实现。"""
    new_status = req.status
    if new_status not in (1, 2):
        raise ValueError("状态值不正确")

    with get_db_cursor() as cur:
        cur.execute(
            "SELECT is_default FROM sys_storage WHERE id = %s;",
            (storage_id,),
        )
        row = cur.fetchone()
        if not row:
            raise ValueError("存储不存在")
        if bool(row["is_default"]) and new_status == 2:
            raise ValueError("不允许禁用默认存储")

        cur.execute(
            """
UPDATE sys_storage
   SET status      = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
            (new_status, current_uid, storage_id),
        )


def set_default_storage_service(storage_id: int, current_uid: int) -> None:
    """设为默认存储服务实现。"""
    with get_db_cursor() as cur:
        cur.execute(
            "SELECT status FROM sys_storage WHERE id = %s;",
            (storage_id,),
        )
        row = cur.fetchone()
        if not row:
            raise ValueError("存储不存在")
        if int(row["status"]) != 1:
            raise ValueError("请先启用该存储后再设为默认")

        cur.execute("UPDATE sys_storage SET is_default = FALSE;")
        cur.execute(
            """
UPDATE sys_storage
   SET is_default  = TRUE,
       update_user = %s,
       update_time = NOW()
 WHERE id          = %s;
""",
            (current_uid, storage_id),
        )

