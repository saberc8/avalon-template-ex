from fastapi import FastAPI

from .routers import (
    auth,
    captcha,
    common,
    monitor_online,
    system_dict,
    system_dept,
    system_option,
    system_menu,
    system_role,
    system_user,
)


def create_app() -> FastAPI:
    """创建 FastAPI 应用并注册所有路由。"""
    app = FastAPI(title="Voc Admin Backend (Python/FastAPI)")

    # 按模块注册路由，路径与 Java/Go 版本保持一致
    app.include_router(auth.router)
    app.include_router(captcha.router)
    app.include_router(common.router)
    app.include_router(system_user.router)
    app.include_router(system_role.router)
    app.include_router(system_menu.router)
    app.include_router(system_dept.router)
    app.include_router(system_option.router)
    app.include_router(monitor_online.router)

    return app


app = create_app()
