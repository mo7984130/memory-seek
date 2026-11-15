use validator::ValidationError;

use crate::validators::{
    email::EMAIL_REGEX,
    username::{USERNAME_REGEX, UsernameValidConfig},
};

/// 校验登录账号符合用户名或邮箱格式
pub fn validate_account(value: &str) -> Result<(), ValidationError> {
    if value.contains('@') {
        if !EMAIL_REGEX.is_match(value) {
            return Err(
                ValidationError::new("invalid_email").with_message("请输入正确的邮箱地址".into())
            );
        }
    } else {
        if !USERNAME_REGEX.is_match(value) {
            return Err(ValidationError::new("invalid_username")
                .with_message(UsernameValidConfig::CHAR_ERROR_MSG.into()));
        }
        let len = value.chars().count();
        if !(UsernameValidConfig::MIN_LENGTH..=UsernameValidConfig::MAX_LENGTH).contains(&len) {
            return Err(ValidationError::new("invalid_length")
                .with_message(UsernameValidConfig::LEN_ERROR_MSG.into()));
        }
    }
    Ok(())
}
