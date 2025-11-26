import base64
import io
import time
import uuid
from random import choices

from captcha.image import ImageCaptcha
from fastapi import APIRouter

from ..api_response import ok
from ..db import get_db_cursor
from ..redis_client import redis_set

router = APIRouter()


def _is_login_captcha_enabled() -> bool:
    """读取 sys_option 中 LOGIN_CAPTCHA_ENABLED 配置，判断是否启用登录验证码。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT COALESCE(value, default_value, '0') AS val
FROM sys_option
WHERE code = %s
LIMIT 1;
""",
            ("LOGIN_CAPTCHA_ENABLED",),
        )
        row = cur.fetchone()
    if not row:
        return False
    val = str(row["val"]).strip()
    return bool(val and val != "0")


@router.get("/captcha/image")
def get_captcha_image():
    """
    获取登录图片验证码。

    行为与 Java/Go 版本保持一致：
    - 当 LOGIN_CAPTCHA_ENABLED=0 或未配置时，仅返回 isEnabled=false；
    - 当 LOGIN_CAPTCHA_ENABLED!=0 时，生成 4 位数字验证码并返回 Base64 图片。
    """
    # 与 Go 版保持一致的过期时间配置：2 分钟
    expiration_minutes = 2
    now_ms = int(time.time() * 1000)
    expire_time = now_ms + expiration_minutes * 60 * 1000

    # 未启用登录验证码：前端隐藏验证码输入框
    if not _is_login_captcha_enabled():
        data = {
            "uuid": "",
            "img": "",
            "expireTime": expire_time,
            "isEnabled": False,
        }
        return ok(data)

    # 启用登录验证码：生成 4 位数字验证码图片（PNG，Base64 编码）
    code = "".join(choices("0123456789", k=4))
    image_captcha = ImageCaptcha(width=120, height=40)
    image = image_captcha.generate_image(code)

    buf = io.BytesIO()
    image.save(buf, format="PNG")
    img_b64 = base64.b64encode(buf.getvalue()).decode("utf-8")
    img_data_url = f"data:image/png;base64,{img_b64}"

    # 生成 uuid，与前端请求体中的 uuid 字段对齐
    captcha_uuid = str(uuid.uuid4())

    # 将验证码文本写入 Redis，键前缀与 Java 一致：CAPTCHA:
    # 过期时间采用与前端一致的过期时间（秒），这里使用 2 分钟。
    expire_seconds = expiration_minutes * 60
    redis_set(f"CAPTCHA:{captcha_uuid}", code, ex_seconds=expire_seconds)

    data = {
        "uuid": captcha_uuid,
        "img": img_data_url,
        "expireTime": expire_time,
        "isEnabled": True,
    }
    return ok(data)
