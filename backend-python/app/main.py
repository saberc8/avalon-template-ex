from fastapi import FastAPI

from .logging_middleware import sys_log_middleware
from .routers import (
    auth,
    captcha,
    common,
    monitor_online,
    monitor_log,
    system_client,
    system_dict,
    system_dept,
    system_file,
    system_option,
    system_menu,
    system_storage,
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
    app.include_router(monitor_log.router)
    app.include_router(system_dict.router)
    app.include_router(system_file.router)
    app.include_router(system_storage.router)
    app.include_router(system_client.router)
    app.include_router(system_user.router)
    app.include_router(system_role.router)
    app.include_router(system_menu.router)
    app.include_router(system_dept.router)
    app.include_router(system_option.router)
    app.include_router(monitor_online.router)

    # 系统日志中间件：记录所有业务请求到 sys_log
    app.middleware("http")(sys_log_middleware)

    return app


app = create_app()
