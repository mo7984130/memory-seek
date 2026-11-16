use crate::{
    UserInfo,
    validators::{
        account::validate_account, email::validate_email, normal::validate_normal_char,
        password::validate_password, username::validate_username,
    },
};
use common_core::time::DateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct LoginRequest {
    #[validate(custom(function = "validate_account"))]
    pub account: String,

    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserInfo,
    pub access_token: String,
    pub access_token_expire_at: DateTime,
    pub refresh_token: String,
    pub refresh_token_expire_at: DateTime,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterRequest {
    /// 用户名
    #[validate(custom(function = "validate_username"))]
    pub username: String,

    /// 邮箱
    #[validate(custom(function = "validate_email"))]
    pub email: String,

    /// 密码
    #[validate(
        custom(function = "validate_password"),
        must_match(other = "confirm_password", message = "两次输入的密码不一致")
    )]
    pub password: String,

    /// 确认密码
    pub confirm_password: String,

    /// 昵称
    #[validate(
        length(min = 1, max = 20, message = "昵称长度在 1 到 20 个字符"),
        custom(function = "validate_normal_char")
    )]
    pub nickname: String,

    /// 邀请码
    #[validate(length(min = 6, max = 6, message = "邀请码长度为6个字符"))]
    pub inviter_code: String,

    /// 邮箱验证码
    #[validate(length(min = 6, max = 6, message = "邮箱验证码长度为6个字符"))]
    pub email_verify_code: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct CreateAdminArgs {
    /// 用户名
    #[validate(custom(function = "validate_username"))]
    pub username: String,

    /// 邮箱
    #[validate(custom(function = "validate_email"))]
    pub email: String,

    /// 密码
    #[validate(
        custom(function = "validate_password"),
        must_match(other = "confirm_password", message = "两次输入的密码不一致")
    )]
    pub password: String,

    /// 确认密码
    pub confirm_password: String,

    /// 昵称
    #[validate(
        length(min = 1, max = 20, message = "昵称长度在 1 到 20 个字符"),
        custom(function = "validate_normal_char")
    )]
    pub nickname: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct SendEmailRequest {
    /// 邮箱
    #[validate(custom(function = "validate_email"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshAccessTokenResponse {
    /// 访问令牌
    pub access_token: String,
    /// 访问令牌过期时间
    pub access_token_expire_at: DateTime,
}
impl RefreshAccessTokenResponse {
    pub fn into_pair(self) -> (String, DateTime) {
        (self.access_token, self.access_token_expire_at)
    }
}
