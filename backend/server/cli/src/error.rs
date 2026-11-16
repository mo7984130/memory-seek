#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Auth错误: {0}")]
    Auth(#[from] auth::error::AuthError),

    #[error("内部错误: {0}")]
    Internal(#[from] common_core::contextual::ContextualError),
}
