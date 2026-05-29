use serde::{Deserialize, Serialize};

pub const SUCCESS_CODE: &str = "200";
pub const SUCCESS_MESSAGE: &str = "成功";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(code: impl Into<String>, msg: impl Into<String>, data: T) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            data,
        }
    }

    pub fn ok(data: T) -> Self {
        Self::new(SUCCESS_CODE, SUCCESS_MESSAGE, data)
    }
}

impl ApiResponse<()> {
    pub fn fail(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(code, msg, ())
    }
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
    }

    #[test]
    fn fail_response_uses_existing_envelope() {
        let res: ApiResponse<()> = ApiResponse::fail("403", "没有访问权限，请联系管理员授权");
        assert_eq!(res.code, "403");
        assert_eq!(res.msg, "没有访问权限，请联系管理员授权");
    }
}
