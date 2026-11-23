use http::StatusCode;
use http::header::InvalidHeaderValue;
use validator::ValidationErrors;

use crate::storage::StorageError;
use crate::transport::TransportError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// 请求构建失败(URI、请求头等非法),与网络状态无关。
    #[error("请求构建失败")]
    Build(#[from] http::Error),
    /// 传输失败,流式响应体读取过程中的错误也归入此类。
    #[error("网络错误")]
    Transport(#[from] TransportError),
    #[error("存储错误")]
    Storage(#[from] StorageError),
    #[error("序列化失败")]
    Decode(#[from] serde_json::Error),
    #[error("{msg}")]
    HttpStatus { status: StatusCode, msg: String },
    #[error("未登录")]
    NotLoggedIn,
    #[error("非法Url")]
    InvalidUrl(#[from] http::uri::InvalidUri),
    #[error("非法请求头值")]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("参数效验错误: {0}")]
    InvalidReq(#[from] ValidationErrors),
}
