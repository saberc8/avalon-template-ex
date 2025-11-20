use crate::interfaces::http::response::{api_ok, ApiJson};

use serde::Serialize;

/// Captcha 配置返回结构，对齐现有各后端实现：默认关闭验证码。
#[derive(Serialize)]
struct CaptchaResp {
    #[serde(rename = "isEnabled")]
    is_enabled: bool,
    #[serde(rename = "type")]
    captcha_type: String,
    uuid: String,
    image: String,
}

/// GET /captcha/image
/// 目前按照 Go/Python/PHP 版本的约定，直接返回关闭验证码的配置。
pub async fn get_captcha_image() -> ApiJson {
    let resp = CaptchaResp {
        is_enabled: false,
        captcha_type: "ARITHMETIC".to_string(),
        uuid: "".to_string(),
        image: "".to_string(),
    };
    api_ok(resp)
}

