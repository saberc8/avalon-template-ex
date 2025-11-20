import time
from typing import Any, Dict, Optional


def _now_millis_str() -> str:
    """返回当前时间的毫秒时间戳字符串，兼容前端 `Number(res.timestamp)`。"""
    return str(int(time.time() * 1000))


def ok(data: Any) -> Dict[str, Any]:
    """成功响应包装，与 Go 版 APIResponse 结构保持一致。"""
    return {
        "code": "200",
        "data": data,
        "msg": "操作成功",
        "success": True,
        "timestamp": _now_millis_str(),
    }


def fail(code: str, msg: str, data: Optional[Any] = None) -> Dict[str, Any]:
    """失败响应包装，HTTP 层仍返回 200 状态码。"""
    return {
        "code": code,
        "data": data,
        "msg": msg,
        "success": False,
        "timestamp": _now_millis_str(),
    }


