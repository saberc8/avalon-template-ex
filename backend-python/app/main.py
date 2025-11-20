from fastapi import FastAPI

from .routers import auth, captcha, common


def create_app() -> FastAPI:
    """创建 FastAPI 应用并注册所有路由。"""
    app = FastAPI(title="Voc Admin Backend (Python/FastAPI)")

    # 按模块注册路由，路径与 Java/Go 版本保持一致
    app.include_router(auth.router)
    app.include_router(captcha.router)
    app.include_router(common.router)

    return app


app = create_app()


