from fastapi import FastAPI

from .routers import auth, captcha, common, system_user, system_role


def create_app() -> FastAPI:
    """创建 FastAPI 应用并注册所有路由。"""
    app = FastAPI(title="Voc Admin Backend (Python/FastAPI)")

    # 按模块注册路由，路径与 Java/Go 版本保持一致
    app.include_router(auth.router)
    app.include_router(captcha.router)
    app.include_router(common.router)
    app.include_router(system_user.router)
    app.include_router(system_role.router)

    return app


app = create_app()
