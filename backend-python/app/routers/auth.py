from fastapi import APIRouter, Depends, Header, Request

from ..models.auth import LoginRequest
from ..security.jwt_token import TokenService
from ..security.rsa import RSADecryptor
from ..services.auth_service import (
    get_token_service,
    get_rsa_decryptor,
    login_service,
    logout_service,
    get_user_info_service,
    get_user_route_service,
)

router = APIRouter()


@router.post("/auth/login")
def login(
    req: LoginRequest,
    request: Request,
    token_svc: TokenService = Depends(get_token_service),
    decryptor: RSADecryptor = Depends(get_rsa_decryptor),
):
    """账号密码登录，保持与 Java/Go 逻辑一致。"""
    return login_service(req, request, token_svc, decryptor)


@router.post("/auth/logout")
def logout(
    authorization: str | None = Header(default=None, alias="Authorization"),
    token_svc: TokenService = Depends(get_token_service),
):
    """
    登出接口：POST /auth/logout。

    行为说明：
    - 解析当前请求的 Token，若无效则返回未授权；
    - 从在线用户存储中移除当前 token；
    - 返回当前登录用户 ID，兼容 Java/Node 版本。
    """
    return logout_service(authorization, token_svc)


@router.get("/auth/user/info")
def get_user_info(
    authorization: str | None = Header(default=None, alias="Authorization"),
    token_svc: TokenService = Depends(get_token_service),
):
    """获取当前登录用户信息。"""
    return get_user_info_service(authorization, token_svc)


@router.get("/auth/user/route")
def get_user_route(
    authorization: str | None = Header(default=None, alias="Authorization"),
    token_svc: TokenService = Depends(get_token_service),
):
    """获取当前用户的路由树结构。"""
    return get_user_route_service(authorization, token_svc)
