from __future__ import annotations

from typing import List, Optional

from fastapi import APIRouter, Body, Header, Patch, Put, Query
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


class OptionResp(BaseModel):
    """系统配置响应结构。"""

    id: int
    name: str
    code: str
    value: str
    description: str


class OptionUpdateItem(BaseModel):
    """系统配置更新项。"""

    id: int
    code: str
    value: object


class OptionResetReq(BaseModel):
    """系统配置恢复默认请求体。"""

    code: List[str] | None = Field(default=None, description="配置编码列表")
    category: str | None = Field(default=None, description="配置类别")


def _normalize_multi_value(raw: str | List[str] | None) -> List[str]:
    """解析 code 查询参数，兼容逗号分隔与重复参数形式。"""
    if raw is None:
        return []
    items = raw if isinstance(raw, list) else [raw]
    result: List[str] = []
    for v in items:
        for part in v.split(","):
            p = part.strip()
            if p:
                result.append(p)
    return result


def _to_option_value_string(value: object) -> str:
    """将任意值转成字符串，保持与 Java/Go 写入 sys_option 的逻辑一致。"""
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        try:
            return str(int(value))
        except Exception:
            return ""
    # 其余情况序列化为 JSON 字符串
    try:
        import json

        return json.dumps(value, ensure_ascii=False)
    except Exception:
        return ""


@router.get("/system/option")
def list_option(
    code: str | List[str] | None = Query(default=None),
    category: str | None = Query(default=None),
):
    """
    查询系统配置列表，支持 code 多值与 category 筛选。
    与 Node/Java 版 `/system/option` 行为保持一致。
    """
    codes = _normalize_multi_value(code)
    category_filter = (category or "").strip()

    where = ["1=1"]
    params: List[object] = []
    if codes:
        placeholders = ", ".join(["%s"] * len(codes))
        where.append(f"code IN ({placeholders})")
        params.extend(codes)
    if category_filter:
        where.append("category = %s")
        params.append(category_filter)
    where_sql = " WHERE " + " AND ".join(where)

    sql = f"""
SELECT id,
       name,
       code,
       COALESCE(value, default_value, '') AS value,
       COALESCE(description, '')          AS description
FROM sys_option
{where_sql}
ORDER BY id ASC;
"""

    with get_db_cursor() as cur:
        cur.execute(sql, params)
        rows = cur.fetchall()

    data = [
        OptionResp(
            id=int(r["id"]),
            name=r["name"],
            code=r["code"],
            value=r["value"] or "",
            description=r["description"] or "",
        ).dict()
        for r in rows
    ]
    return ok(data)


@router.put("/system/option")
def update_option(
    body: List[OptionUpdateItem] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """
    批量保存系统配置值：PUT /system/option。
    """
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if not isinstance(body, list) or not body:
        return fail("400", "请求参数不正确")

    now_value_params: List[tuple] = []
    for item in body:
        try:
            id_num = int(item.id)
        except Exception:
            return fail("400", "请求参数不正确")
        code = (item.code or "").strip()
        if id_num <= 0 or not code:
            return fail("400", "请求参数不正确")
        value_str = _to_option_value_string(item.value)
        now_value_params.append((value_str, current_uid, id_num, code))

    with get_db_cursor() as cur:
        try:
            for value_str, uid, oid, code in now_value_params:
                cur.execute(
                    """
UPDATE sys_option
   SET value = %s,
       update_user = %s,
       update_time = NOW()
 WHERE id = %s AND code = %s;
""",
                    (value_str, uid, oid, code),
                )
        except Exception:
            return fail("500", "保存系统配置失败")

    return ok(True)


@router.patch("/system/option/value")
def reset_option_value(
    body: OptionResetReq = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """
    恢复默认值：PATCH /system/option/value。
    - 若提供 category，则按类别重置；
    - 否则按 code 列表重置。
    """
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if body is None:
        return fail("400", "请求参数不正确")

    category = (body.category or "").strip()
    codes = [c.strip() for c in (body.code or []) if c and c.strip()]
    if not category and not codes:
        return fail("400", "键列表或类别不能为空")

    with get_db_cursor() as cur:
        try:
            if category:
                cur.execute(
                    """
UPDATE sys_option
   SET value = NULL
 WHERE category = %s;
""",
                    (category,),
                )
            else:
                placeholders = ", ".join(["%s"] * len(codes))
                cur.execute(
                    f"""
UPDATE sys_option
   SET value = NULL
 WHERE code IN ({placeholders});
""",
                    tuple(codes),
                )
        except Exception:
            return fail("500", "恢复默认配置失败")

    return ok(True)

