use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    interfaces::http::AppState,
    shared::{error::AppError, response::ApiResponse},
};

const SVG_1X1_BASE64: &str =
    "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxIiBoZWlnaHQ9IjEiLz4=";
const IMAGE_CAPTCHA_EXPIRATION_SECONDS: i64 = 120;
const IMAGE_CAPTCHA_CHARS: &[u8] = b"0123456789";
const LOGIN_CAPTCHA_ENABLED_CODE: &str = "LOGIN_CAPTCHA_ENABLED";
const CAPTCHA_ERROR_MESSAGE: &str = "验证码错误或已过期";

#[derive(Debug, Clone, Default)]
pub struct CaptchaStore {
    challenges: Arc<Mutex<HashMap<String, CaptchaChallenge>>>,
}

#[derive(Debug, Clone)]
struct CaptchaChallenge {
    code: String,
    expire_time: i64,
}

impl CaptchaStore {
    pub fn insert(&self, uuid: String, code: String, expire_time: i64) -> Result<(), AppError> {
        self.challenges
            .lock()
            .map_err(|_| AppError::bad_request("验证码状态异常"))?
            .insert(uuid, CaptchaChallenge { code, expire_time });
        Ok(())
    }

    pub fn verify(&self, uuid: &str, code: &str, now_millis: i64) -> Result<(), AppError> {
        let uuid = uuid.trim();
        let code = code.trim();
        if uuid.is_empty() || code.is_empty() {
            return Err(AppError::bad_request(CAPTCHA_ERROR_MESSAGE));
        }

        let mut challenges = self
            .challenges
            .lock()
            .map_err(|_| AppError::bad_request("验证码状态异常"))?;
        let Some(challenge) = challenges.get(uuid) else {
            return Err(AppError::bad_request(CAPTCHA_ERROR_MESSAGE));
        };
        if challenge.expire_time <= now_millis {
            challenges.remove(uuid);
            return Err(AppError::bad_request(CAPTCHA_ERROR_MESSAGE));
        }
        if !challenge.code.eq_ignore_ascii_case(code) {
            return Err(AppError::bad_request(CAPTCHA_ERROR_MESSAGE));
        }

        challenges.remove(uuid);
        Ok(())
    }
}

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

async fn image(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ImageCaptchaResp>>, AppError> {
    let expire_time = image_captcha_expire_time();
    if !is_login_captcha_enabled(&state.db).await? {
        return Ok(Json(ApiResponse::ok(disabled_image_captcha_response(
            expire_time,
        ))));
    }

    let code = random_image_captcha_code();
    let uuid = uuid::Uuid::new_v4().to_string();
    state
        .captcha
        .insert(uuid.clone(), code.clone(), expire_time)?;
    Ok(Json(ApiResponse::ok(enabled_image_captcha_response(
        &code,
        uuid,
        expire_time,
    ))))
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

async fn is_login_captcha_enabled(db: &PgPool) -> Result<bool, AppError> {
    let value = sqlx::query_scalar::<_, Option<String>>(
        r#"
SELECT COALESCE(value, default_value)
FROM sys_option
WHERE code = $1
"#,
    )
    .bind(LOGIN_CAPTCHA_ENABLED_CODE)
    .fetch_optional(db)
    .await?
    .flatten();

    Ok(parse_login_captcha_enabled(value.as_deref()))
}

pub async fn ensure_login_captcha(
    state: &AppState,
    uuid: Option<&str>,
    code: Option<&str>,
) -> Result<(), AppError> {
    if !is_login_captcha_enabled(&state.db).await? {
        return Ok(());
    }

    state.captcha.verify(
        uuid.unwrap_or_default(),
        code.unwrap_or_default(),
        current_time_millis(),
    )
}

fn parse_login_captcha_enabled(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "y" | "on")
    )
}

fn disabled_image_captcha_response(expire_time: i64) -> ImageCaptchaResp {
    ImageCaptchaResp {
        uuid: String::new(),
        img: String::new(),
        expire_time,
        is_enabled: false,
    }
}

fn enabled_image_captcha_response(code: &str, uuid: String, expire_time: i64) -> ImageCaptchaResp {
    ImageCaptchaResp {
        uuid,
        img: svg_data_url(&image_captcha_svg(code)),
        expire_time,
        is_enabled: true,
    }
}

fn random_image_captcha_code() -> String {
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| {
            let index = rng.gen_range(0..IMAGE_CAPTCHA_CHARS.len());
            IMAGE_CAPTCHA_CHARS[index] as char
        })
        .collect()
}

fn image_captcha_svg(code: &str) -> String {
    let code = escape_svg_text(code);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="40" viewBox="0 0 120 40">
<rect width="120" height="40" rx="6" fill="#f8fafc"/>
<path d="M8 30 C28 12, 48 36, 70 18 S104 10, 112 28" fill="none" stroke="#94a3b8" stroke-width="2"/>
<line x1="12" y1="12" x2="108" y2="31" stroke="#cbd5e1" stroke-width="1"/>
<text x="60" y="28" text-anchor="middle" font-family="Arial, Helvetica, sans-serif" font-size="24" font-weight="700" letter-spacing="4" fill="#0f172a">{code}</text>
</svg>"##
    )
}

fn svg_data_url(svg: &str) -> String {
    format!(
        "data:image/svg+xml;base64,{}",
        general_purpose::STANDARD.encode(svg.as_bytes())
    )
}

fn image_captcha_expire_time() -> i64 {
    current_time_millis() + IMAGE_CAPTCHA_EXPIRATION_SECONDS * 1000
}

fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn escape_svg_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
    fn disabled_image_captcha_response_has_no_placeholder_image() {
        let response = disabled_image_captcha_response(120);

        assert_eq!(response.uuid, "");
        assert_eq!(response.img, "");
        assert!(!response.is_enabled);
    }

    #[test]
    fn enabled_image_captcha_response_contains_visible_svg_data_url() {
        let response = enabled_image_captcha_response("1234", "uuid-1".to_owned(), 120);

        assert_eq!(response.uuid, "uuid-1");
        assert!(response.is_enabled);
        assert!(response.img.starts_with("data:image/svg+xml;base64,"));
        assert_ne!(response.img, SVG_1X1_BASE64);
    }

    #[test]
    fn image_captcha_svg_contains_visible_code() {
        let svg = image_captcha_svg("1234");

        assert!(svg.contains("width=\"120\""));
        assert!(svg.contains("height=\"40\""));
        assert!(svg.contains(">1234<"));
        assert!(!svg.contains("width=\"1\" height=\"1\""));
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

    #[test]
    fn captcha_store_requires_matching_code_once() {
        let store = CaptchaStore::default();
        let now = current_time_millis();
        let expire_time = now + IMAGE_CAPTCHA_EXPIRATION_SECONDS * 1000;
        store
            .insert("uuid-1".to_owned(), "1234".to_owned(), expire_time)
            .unwrap();

        assert!(store.verify("uuid-1", "0000", now).is_err());
        assert!(store.verify("uuid-1", "1234", now).is_ok());
        assert!(store.verify("uuid-1", "1234", now).is_err());
    }
}
