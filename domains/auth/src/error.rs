use common_core::contextual::{ContextualError, from::validator::format_validation_error_msg};
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("生成邀请码达到最大尝试次数")]
    InviterCodeGenerateExhausted,

    #[error("邀请码错误")]
    InviterCodeInvalid,

    #[error("邮箱验证码错误")]
    EmailVerifyCodeInvalid,

    #[error("用户名已存在")]
    UsernameExists,

    #[error("邮箱已存在")]
    EmailExists,

    #[error("账号或密码错误")]
    InvalidCredentials,

    #[error("登录已过期，请重新登录")]
    RefreshTokenInvalid,

    #[error("{0}")]
    Validation(String),

    #[error("用户不存在")]
    UserNotFound,

    #[error("图片效验失败: {}", 0)]
    ImageInvalid(#[from] media_validator::MediaValidatorError),

    #[error(transparent)]
    Internal(#[from] ContextualError),
}
pub type Result<T> = std::result::Result<T, AuthError>;

impl From<ValidationErrors> for AuthError {
    fn from(value: ValidationErrors) -> Self {
        Self::Validation(format_validation_error_msg(&value))
    }
}
