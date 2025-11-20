import time

from fastapi import APIRouter

from ..api_response import ok

router = APIRouter()


@router.get("/captcha/image")
def get_captcha_image():
    """
    返回简化版图形验证码结构。

    与 Go 版保持一致：始终返回 isEnabled=false，前端因此隐藏验证码输入框。
    """
    now = int(time.time() * 1000)
    data = {
        "uuid": "",
        "img": "",
        "expireTime": now,
        "isEnabled": False,
    }
    return ok(data)


