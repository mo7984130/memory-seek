use common_core::contextual::ContextualError;
use memory_seek_api::client::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum E2eError {
    #[error("Api错误: {0}")]
    Api(#[from] ApiError),

    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Io错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("内部错误: {0}")]
    Internal(#[from] ContextualError),
}
