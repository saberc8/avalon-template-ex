use serde::Serialize;
use serde_json::Value;

use axum::Json;

/// 通用 API 响应结构，与前端 ApiRes<T> 结构保持一致。
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub data: T,
    pub msg: String,
    pub success: bool,
    pub timestamp: String,
}

/// 分页结果结构，与 Go 版 PageResult 一致。
#[derive(Serialize)]
pub struct PageResult<T> {
    pub list: Vec<T>,
    pub total: i64,
}

/// 简化类型别名：所有接口统一返回 Json<ApiResponse<Value>>。
pub type ApiJson = Json<ApiResponse<Value>>;

/// 获取当前毫秒时间戳字符串，与 Java/Go 前端约定保持一致。
fn now_millis_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    now.to_string()
}

/// 构造成功响应。
pub fn api_ok<T: Serialize>(data: T) -> ApiJson {
    let value = serde_json::to_value(data).unwrap_or(Value::Null);
    Json(ApiResponse {
        code: "200".to_string(),
        data: value,
        msg: "操作成功".to_string(),
        success: true,
        timestamp: now_millis_string(),
    })
}

/// 构造失败响应。
pub fn api_fail(code: &str, msg: &str) -> ApiJson {
    Json(ApiResponse {
        code: code.to_string(),
        data: Value::Null,
        msg: msg.to_string(),
        success: false,
        timestamp: now_millis_string(),
    })
}

