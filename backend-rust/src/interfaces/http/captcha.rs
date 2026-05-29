use axum::{extract::Query, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::{interfaces::http::AppState, shared::response::ApiResponse};

const SVG_1X1_BASE64: &str =
    "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxIiBoZWlnaHQ9IjEiLz4=";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/captcha/image", get(image))
        .route("/captcha/behavior", get(behavior).post(check_behavior))
        .route("/captcha/mail", get(mail))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageCaptchaResp {
    uuid: String,
    img: String,
    expire_time: i64,
    is_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BehaviorCaptchaResp {
    original_image_base64: String,
    point: CaptchaPoint,
    jigsaw_image_base64: String,
    token: String,
    secret_key: String,
    word_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckBehaviorCaptchaResp {
    rep_code: String,
    rep_msg: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MailQuery {
    #[serde(default)]
    email: String,
    #[serde(default)]
    captcha_verification: String,
}

async fn image() -> Json<ApiResponse<ImageCaptchaResp>> {
    Json(ApiResponse::ok(ImageCaptchaResp {
        uuid: uuid::Uuid::new_v4().to_string(),
        img: SVG_1X1_BASE64.to_owned(),
        expire_time: 120,
        is_enabled: false,
    }))
}

async fn behavior() -> Json<ApiResponse<BehaviorCaptchaResp>> {
    Json(ApiResponse::ok(BehaviorCaptchaResp {
        original_image_base64: SVG_1X1_BASE64.to_owned(),
        point: CaptchaPoint { x: 0, y: 0 },
        jigsaw_image_base64: SVG_1X1_BASE64.to_owned(),
        token: uuid::Uuid::new_v4().to_string(),
        secret_key: uuid::Uuid::new_v4().simple().to_string(),
        word_list: Vec::new(),
    }))
}

async fn check_behavior() -> Json<ApiResponse<CheckBehaviorCaptchaResp>> {
    Json(ApiResponse::ok(CheckBehaviorCaptchaResp {
        rep_code: "0000".to_owned(),
        rep_msg: "成功".to_owned(),
    }))
}

async fn mail(Query(query): Query<MailQuery>) -> Json<ApiResponse<bool>> {
    let _ = (query.email, query.captcha_verification);
    Json(ApiResponse::ok(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_captcha_response_uses_vue_field_names() {
        let value = serde_json::to_value(ImageCaptchaResp {
            uuid: "u".to_owned(),
            img: "img".to_owned(),
            expire_time: 120,
            is_enabled: false,
        })
        .unwrap();

        assert_eq!(value["expireTime"], 120);
        assert_eq!(value["isEnabled"], false);
    }

    #[test]
    fn behavior_check_response_uses_vue_field_names() {
        let value = serde_json::to_value(CheckBehaviorCaptchaResp {
            rep_code: "0000".to_owned(),
            rep_msg: "成功".to_owned(),
        })
        .unwrap();

        assert_eq!(value["repCode"], "0000");
        assert_eq!(value["repMsg"], "成功");
    }
}
