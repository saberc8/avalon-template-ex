use std::time::Instant;

use axum::{
    body::{to_bytes, Body, Bytes},
    extract::State,
    http::{header, HeaderMap, Method, Request},
    middleware::Next,
    response::Response,
};
use serde_json::Value;

use crate::{
    application::monitor::log_service::{
        build_log_record, LogRecordInput, LogService, OPERATION_LOG_TYPE,
    },
    interfaces::http::AppState,
};

const REQUEST_BODY_LIMIT: usize = 64 * 1024;
const RESPONSE_BODY_LIMIT: usize = 512 * 1024;

pub async fn record_access_log(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !should_log_request(request.method(), request.uri().path()) {
        return next.run(request).await;
    }

    let started = Instant::now();
    let method = request.method().clone();
    let uri = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();
    let create_user = user_id_from_headers(&state, &headers);
    let ip = client_ip(&headers);
    let browser = browser_name(&headers);
    let os = os_name(&headers);
    let request_headers = safe_headers_json(&headers);

    let (request, request_body) = maybe_capture_request_body(request).await;
    let response = next.run(request).await;
    let status_code = response.status().as_u16() as i32;
    let response_headers = safe_headers_json(response.headers());
    let (response, response_body) = capture_response_body(response).await;
    let (status, error_msg) = response_status(&response_body, status_code);
    let (module, description) = operation_labels(&method, &path);
    let time_taken = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
    let response_body = truncate(&response_body, RESPONSE_BODY_LIMIT);

    let record = build_log_record(LogRecordInput {
        description: &description,
        module: &module,
        log_type: OPERATION_LOG_TYPE,
        request_url: &uri,
        request_method: method.as_str(),
        request_headers: &request_headers,
        request_body: &request_body,
        status_code,
        response_headers: &response_headers,
        response_body: &response_body,
        time_taken,
        ip: &ip,
        browser: &browser,
        os: &os,
        status,
        error_msg: &error_msg,
        create_user,
    });
    if let Err(error) = LogService::new(state.db).create(&record).await {
        tracing::warn!(?error, "failed to write access log");
    }

    response
}

pub fn should_log_request(method: &Method, path: &str) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    ) && !path.starts_with("/auth/login")
        && !path.starts_with("/captcha")
        && !path.starts_with("/system/log")
}

pub fn operation_labels(method: &Method, path: &str) -> (String, String) {
    let module = if path.starts_with("/system/user") {
        "用户管理"
    } else if path.starts_with("/system/role") {
        "角色管理"
    } else if path.starts_with("/system/dept") {
        "部门管理"
    } else if path.starts_with("/system/menu") {
        "菜单管理"
    } else if path.starts_with("/system/dict") {
        "字典管理"
    } else if path.starts_with("/system/storage") {
        "存储管理"
    } else if path.starts_with("/system/client") {
        "客户端管理"
    } else if path.starts_with("/system/file") || path.starts_with("/common/file") {
        "文件管理"
    } else if path.starts_with("/system/option") {
        "系统配置"
    } else if path.starts_with("/monitor/online") {
        "在线用户"
    } else if path.starts_with("/user/profile") {
        "个人中心"
    } else {
        "系统操作"
    };
    (module.to_owned(), format!("{} {}", method.as_str(), path))
}

pub fn response_status(response_body: &str, status_code: i32) -> (i16, String) {
    if !(200..300).contains(&status_code) {
        return (2, format!("HTTP {status_code}"));
    }
    if let Ok(value) = serde_json::from_str::<Value>(response_body) {
        let code = value.get("code").and_then(Value::as_str).unwrap_or("200");
        if code == "200" {
            return (1, String::new());
        }
        let msg = value
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("请求失败")
            .to_owned();
        return (2, msg);
    }
    (1, String::new())
}

async fn maybe_capture_request_body(request: Request<Body>) -> (Request<Body>, String) {
    if is_multipart(request.headers()) || is_large_body(request.headers(), REQUEST_BODY_LIMIT) {
        return (request, String::new());
    }

    let (parts, body) = request.into_parts();
    match to_bytes(body, REQUEST_BODY_LIMIT).await {
        Ok(bytes) => {
            let body_text = sanitize_body(&bytes, parts.uri.path());
            (Request::from_parts(parts, Body::from(bytes)), body_text)
        }
        Err(_) => (Request::from_parts(parts, Body::empty()), String::new()),
    }
}

async fn capture_response_body(response: Response) -> (Response, String) {
    let (parts, body) = response.into_parts();
    match to_bytes(body, RESPONSE_BODY_LIMIT).await {
        Ok(bytes) => {
            let body_text = String::from_utf8_lossy(&bytes).to_string();
            (Response::from_parts(parts, Body::from(bytes)), body_text)
        }
        Err(_) => (Response::from_parts(parts, Body::empty()), String::new()),
    }
}

fn user_id_from_headers(state: &AppState, headers: &HeaderMap) -> Option<i64> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| state.jwt.parse(value).ok())
        .map(|claims| claims.user_id)
}

pub fn bearer_token_value(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(strip_bearer)
        .filter(|value| !value.is_empty())
}

fn strip_bearer(value: &str) -> String {
    value
        .trim()
        .split_once(' ')
        .and_then(|(scheme, token)| {
            if scheme.eq_ignore_ascii_case("bearer") {
                Some(token.trim())
            } else {
                None
            }
        })
        .unwrap_or_else(|| value.trim())
        .to_owned()
}

pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_owned()
}

pub fn browser_name(headers: &HeaderMap) -> String {
    let ua = user_agent(headers);
    if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Chrome/") {
        "Chrome"
    } else if ua.contains("Firefox/") {
        "Firefox"
    } else if ua.contains("Safari/") {
        "Safari"
    } else {
        ""
    }
    .to_owned()
}

pub fn os_name(headers: &HeaderMap) -> String {
    let ua = user_agent(headers);
    if ua.contains("Windows") {
        "Windows"
    } else if ua.contains("Mac OS X") || ua.contains("Macintosh") {
        "macOS"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else if ua.contains("Linux") {
        "Linux"
    } else {
        ""
    }
    .to_owned()
}

fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned()
}

fn is_multipart(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().starts_with("multipart/"))
        .unwrap_or(false)
}

fn is_large_body(headers: &HeaderMap, limit: usize) -> bool {
    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .map(|length| length > limit)
        .unwrap_or(false)
}

fn sanitize_body(bytes: &Bytes, path: &str) -> String {
    let body = String::from_utf8_lossy(bytes).to_string();
    if path.to_ascii_lowercase().contains("password")
        || body.to_ascii_lowercase().contains("password")
    {
        "[redacted]".to_owned()
    } else {
        truncate(&body, REQUEST_BODY_LIMIT)
    }
}

fn safe_headers_json(headers: &HeaderMap) -> String {
    let mut values = serde_json::Map::new();
    for name in [header::CONTENT_TYPE, header::USER_AGENT] {
        if let Some(value) = headers.get(&name).and_then(|value| value.to_str().ok()) {
            values.insert(name.as_str().to_owned(), Value::String(value.to_owned()));
        }
    }
    Value::Object(values).to_string()
}

fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        value.to_owned()
    } else {
        value.chars().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn access_log_skips_reads_and_login() {
        assert!(!should_log_request(&Method::GET, "/system/user"));
        assert!(!should_log_request(&Method::POST, "/auth/login"));
        assert!(should_log_request(&Method::POST, "/system/user"));
    }

    #[test]
    fn response_status_reads_vue_envelope_code() {
        assert_eq!(
            response_status(r#"{"code":"200","msg":"成功"}"#, 200),
            (1, String::new())
        );
        assert_eq!(
            response_status(r#"{"code":"400","msg":"请求参数错误"}"#, 200),
            (2, "请求参数错误".to_owned())
        );
    }

    #[test]
    fn client_metadata_is_derived_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("10.0.0.1, proxy"),
        );
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X) AppleWebKit Chrome/120",
            ),
        );

        assert_eq!(client_ip(&headers), "10.0.0.1");
        assert_eq!(browser_name(&headers), "Chrome");
        assert_eq!(os_name(&headers), "macOS");
    }
}
