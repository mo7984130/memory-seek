use common_core::contextual::ContextualError;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("生成邀请码达到最大尝试次数")]
    InviterCodeGenerateExhausted,

    #[error("用户不存在")]
    UserNotFound,

    #[error("图片效验失败: {}", 0)]
    ImageInvalid(#[from] media_validator::MediaValidatorError),

    #[error(transparent)]
    Internal(#[from] ContextualError),
}
pub type Result<T> = std::result::Result<T, UserError>;
