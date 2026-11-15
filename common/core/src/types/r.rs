use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// 统一 API 响应格式
///
/// 所有接口使用此结构体作为响应载体，JSON 序列化后字段名为驼峰命名。
/// 成功时包含 `code` 和 `data`，失败时包含 `code` 和 `msg`。
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct R<T> {
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SucR<T> {
    pub code: u16,
    pub data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrR {
    pub code: u16,
    pub msg: String,
}
impl Display for ErrR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.msg)
    }
}

impl<T> R<T>
where
    T: Serialize,
{
    /// 构造成功响应；`code` 固定为 200
    pub fn ok(data: T) -> Self {
        Self {
            code: 200,
            msg: None,
            data: Some(data),
        }
    }
}

impl R<()> {
    /// 构造错误响应；`code` 为 HTTP 状态码
    pub fn err(code: u16, msg: &str) -> Self {
        Self {
            code,
            msg: Some(msg.to_string()),
            data: None,
        }
    }
}

#[cfg(feature = "axum")]
impl<T> axum::response::IntoResponse for R<T>
where
    T: Serialize,
{
    /// 按 `code` 转为 HTTP 状态码，非法值回退为 500
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::from_u16(self.code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        (status, axum::Json(self)).into_response()
    }
}
