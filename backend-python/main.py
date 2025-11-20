"""
FastAPI 启动入口。

运行命令示例：

    cd backend-python
    uvicorn app.main:app --reload --port 4398

说明：
- 保持默认端口 4398 与 Go 版一致，便于前端切换；
- 数据库与认证依赖环境变量配置，参见 app/config.py。
"""

from app.main import app  # noqa: F401


