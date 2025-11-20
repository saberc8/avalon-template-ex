from typing import List, Optional

from pydantic import BaseModel, Field


class LoginRequest(BaseModel):
    """与前端 POST /auth/login 请求体结构一致。"""

    clientId: str = Field("", description="客户端 ID")
    authType: str = Field("ACCOUNT", description="认证方式，当前仅支持 ACCOUNT")
    username: str
    password: str
    captcha: str = ""
    uuid: str = ""


class LoginResponse(BaseModel):
    """登录成功返回 token。"""

    token: str


class UserInfo(BaseModel):
    """与前端 UserInfo 类型对齐。"""

    id: int
    username: str
    nickname: str
    gender: int
    email: str
    phone: str
    avatar: str
    description: str
    pwdResetTime: str
    pwdExpired: bool
    registrationDate: str
    deptName: str
    roles: List[str]
    permissions: List[str]


class RouteItem(BaseModel):
    """与前端 RouteItem 类型对齐。"""

    id: int
    title: str
    parentId: int
    type: int
    path: str
    name: str
    component: str
    redirect: str
    icon: str
    isExternal: bool
    isHidden: bool
    isCache: bool
    permission: str
    roles: List[str]
    sort: int
    status: int
    children: List["RouteItem"] = []
    activeMenu: str = ""
    alwaysShow: bool = False
    breadcrumb: bool = True
    showInTabs: bool = True
    affix: bool = False


RouteItem.update_forward_refs()


