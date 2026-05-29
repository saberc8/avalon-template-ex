use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const SUCCESS_CODE: &str = "200";
pub const SUCCESS_MESSAGE: &str = "成功";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub data: T,
    pub msg: String,
    pub success: bool,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    pub fn new(code: impl Into<String>, msg: impl Into<String>, data: T, success: bool) -> Self {
        Self {
            code: code.into(),
            data,
            msg: msg.into(),
            success,
            timestamp: now_millis_string(),
        }
    }

    pub fn ok(data: T) -> Self {
        Self::new(SUCCESS_CODE, SUCCESS_MESSAGE, data, true)
    }
}

impl ApiResponse<()> {
    pub fn fail(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(code, msg, (), false)
    }
}

fn now_millis_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ok_response_uses_existing_envelope() {
        let res = ApiResponse::ok(json!({"id": "1"}));
        assert_eq!(res.code, "200");
        assert_eq!(res.msg, "成功");
        assert!(res.success);
        assert!(!res.timestamp.is_empty());
    }

    #[test]
    fn fail_response_uses_existing_envelope() {
        let res: ApiResponse<()> = ApiResponse::fail("403", "没有访问权限，请联系管理员授权");
        assert_eq!(res.code, "403");
        assert_eq!(res.msg, "没有访问权限，请联系管理员授权");
        assert!(!res.success);
        assert!(!res.timestamp.is_empty());
    }
}
