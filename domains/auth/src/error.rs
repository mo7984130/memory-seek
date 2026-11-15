use common_core::contextual::{ContextualError, from::validator::format_validation_error_msg};
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
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

    #[error(transparent)]
    Internal(#[from] ContextualError),
}

impl From<ValidationErrors> for AuthError {
    fn from(value: ValidationErrors) -> Self {
        Self::Validation(format_validation_error_msg(&value))
    }
}
