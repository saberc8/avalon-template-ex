from datetime import datetime
from typing import List

from fastapi import APIRouter, Depends, Header

from ..api_response import fail, ok
from ..config import get_settings
from ..db import get_db_cursor
from ..models.auth import LoginRequest, LoginResponse, RouteItem, UserInfo
from ..security.jwt_token import TokenClaims, TokenService
from ..security.password import PasswordVerifier
from ..security.rsa import RSADecryptor

router = APIRouter()


def get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def get_rsa_decryptor() -> RSADecryptor:
    """构造 RSA 解密器实例。"""
    s = get_settings()
    return RSADecryptor.from_base64_key(s.rsa_private_key_b64)


def parse_token(auth_header: str | None, token_svc: TokenService) -> TokenClaims | None:
    """从 Authorization 头中解析用户信息。"""
    if not auth_header:
        return None
    return token_svc.parse(auth_header)


@router.post("/auth/login")
def login(
    req: LoginRequest,
    token_svc: TokenService = Depends(get_token_service),
    decryptor: RSADecryptor = Depends(get_rsa_decryptor),
):
    """账号密码登录，保持与 Java/Go 逻辑一致。"""
    auth_type = (req.authType or "").strip().upper()
    if auth_type and auth_type != "ACCOUNT":
        return fail("400", "暂不支持该认证方式")

    if not req.clientId.strip():
        return fail("400", "客户端ID不能为空")
    if not req.username.strip():
        return fail("400", "用户名不能为空")
    if not req.password.strip():
        return fail("400", "密码不能为空")

    # RSA 解密密码
    try:
        raw_password = decryptor.decrypt_base64(req.password)
    except Exception:
        return fail("400", "密码解密失败")

    # 从 sys_user 查询用户
    from ..security.password import PasswordVerifier  # 延迟导入避免循环
    from ..db import get_db_cursor

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_user,
    create_time,
    update_user,
    update_time
FROM sys_user
WHERE username = %s
LIMIT 1;
""",
            (req.username,),
        )
        row = cur.fetchone()

    if not row:
        return fail("400", "用户名或密码不正确")

    encoded_pwd: str = row["password"]
    if not PasswordVerifier.verify(raw_password, encoded_pwd):
        return fail("400", "用户名或密码不正确")

    if row["status"] != 1:
        return fail("400", "此账号已被禁用，如有疑问，请联系管理员")

    token = token_svc.generate(int(row["id"]))
    resp = LoginResponse(token=token)
    return ok(resp.dict())


@router.get("/auth/user/info")
def get_user_info(
    authorization: str | None = Header(default=None, alias="Authorization"),
    token_svc: TokenService = Depends(get_token_service),
):
    """获取当前登录用户信息。"""
    claims = parse_token(authorization, token_svc)
    if not claims:
        return fail("401", "未授权，请重新登录")

    user_id = claims.user_id
    with get_db_cursor() as cur:
        # 加载用户
        cur.execute(
            """
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_user,
    create_time,
    update_user,
    update_time
FROM sys_user
WHERE id = %s
LIMIT 1;
""",
            (user_id,),
        )
        u = cur.fetchone()
        if not u:
            return fail("401", "未授权，请重新登录")

        # 角色
        cur.execute(
            """
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = %s;
""",
            (user_id,),
        )
        roles = cur.fetchall()
        role_codes: List[str] = [r["code"] for r in roles]

        # 权限
        cur.execute(
            """
SELECT DISTINCT m.permission
FROM sys_menu AS m
LEFT JOIN sys_role_menu AS rm ON rm.menu_id = m.id
LEFT JOIN sys_role AS r ON r.id = rm.role_id
LEFT JOIN sys_user_role AS ur ON ur.role_id = r.id
LEFT JOIN sys_user AS u ON u.id = ur.user_id
WHERE u.id = %s
  AND m.status = 1
  AND m.permission IS NOT NULL;
""",
            (user_id,),
        )
        perms_rows = cur.fetchall()
        permissions: List[str] = [p["permission"] for p in perms_rows]

        # 部门名称
        dept_name = ""
        if u["dept_id"]:
            cur.execute(
                "SELECT name FROM sys_dept WHERE id = %s LIMIT 1;",
                (u["dept_id"],),
            )
            dept_row = cur.fetchone()
            if dept_row:
                dept_name = dept_row["name"]

    pwd_reset_time = ""
    if u["pwd_reset_time"]:
        pwd_reset_time = u["pwd_reset_time"].strftime("%Y-%m-%d %H:%M:%S")
    registration_date = u["create_time"].strftime("%Y-%m-%d")

    def _opt(v) -> str:
        return v or ""

    user_info = UserInfo(
        id=u["id"],
        username=u["username"],
        nickname=u["nickname"],
        gender=u["gender"],
        email=_opt(u["email"]),
        phone=_opt(u["phone"]),
        avatar=_opt(u["avatar"]),
        description=_opt(u["description"]),
        pwdResetTime=pwd_reset_time,
        pwdExpired=False,
        registrationDate=registration_date,
        deptName=dept_name,
        roles=role_codes,
        permissions=permissions,
    )
    return ok(user_info.dict())


@router.get("/auth/user/route")
def get_user_route(
    authorization: str | None = Header(default=None, alias="Authorization"),
    token_svc: TokenService = Depends(get_token_service),
):
    """获取当前用户的路由树结构。"""
    claims = parse_token(authorization, token_svc)
    if not claims:
        return fail("401", "未授权，请重新登录")
    user_id = claims.user_id

    with get_db_cursor() as cur:
        # 角色
        cur.execute(
            """
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = %s;
""",
            (user_id,),
        )
        roles = cur.fetchall()
        if not roles:
            return ok([])
        role_codes: List[str] = [r["code"] for r in roles]

        # 所有角色关联的菜单
        role_ids = [r["id"] for r in roles]
        cur.execute(
            """
SELECT
  m.id,
  m.parent_id,
  m.title,
  m.type,
  m.path,
  m.name,
  m.component,
  m.redirect,
  m.icon,
  COALESCE(m.is_external, false) AS is_external,
  COALESCE(m.is_cache, false)    AS is_cache,
  COALESCE(m.is_hidden, false)   AS is_hidden,
  m.permission,
  COALESCE(m.sort, 0)            AS sort,
  m.status
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
WHERE rm.role_id = ANY(%s);
""",
            (role_ids,),
        )
        menu_rows = cur.fetchall()

    # 过滤掉按钮类型
    menus = [m for m in menu_rows if int(m["type"]) != 3]
    if not menus:
        return ok([])

    # 排序：sort 升序，其次 id 升序
    menus.sort(key=lambda m: (int(m["sort"]), int(m["id"])))

    # 构建 id -> RouteItem
    node_map: dict[int, RouteItem] = {}
    for m in menus:
        node_map[int(m["id"])] = RouteItem(
            id=int(m["id"]),
            title=m["title"],
            parentId=int(m["parent_id"]),
            type=int(m["type"]),
            path=m.get("path") or "",
            name=m.get("name") or "",
            component=m.get("component") or "",
            redirect=m.get("redirect") or "",
            icon=m.get("icon") or "",
            isExternal=bool(m["is_external"]),
            isHidden=bool(m["is_hidden"]),
            isCache=bool(m["is_cache"]),
            permission=m.get("permission") or "",
            roles=role_codes,
            sort=int(m["sort"]),
            status=int(m["status"]),
            children=[],
        )

    # 组装树结构
    roots: list[RouteItem] = []
    for item in node_map.values():
        if item.parentId == 0:
            roots.append(item)
            continue
        parent = node_map.get(item.parentId)
        if parent is None:
            roots.append(item)
            continue
        parent.children.append(item)

    # 递归对子节点排序
    def sort_children(nodes: list[RouteItem]) -> None:
        for node in nodes:
            if node.children:
                node.children.sort(key=lambda x: (x.sort, x.id))
                sort_children(node.children)

    sort_children(roots)

    # 返回 Pydantic 可序列化结构
    return ok([r.dict() for r in roots])


