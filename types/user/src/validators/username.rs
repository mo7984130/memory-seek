use std::sync::LazyLock;

use validator::ValidationError;

/// 用户名验证配置，定义长度范围、允许字符模式和错误提示信息
pub struct UsernameValidConfig;
impl UsernameValidConfig {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 32;
    pub const CHAR_ERROR_MSG: &str = "用户名只能包含字母、数字、下划线和短横线";
    pub const LEN_ERROR_MSG: &str = "账号长度需在 4-32 之间";
    pub const PATTERN: &str = r"^[a-zA-Z0-9_-]+$";
}

pub static USERNAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(UsernameValidConfig::PATTERN).unwrap());

/// 校验用户名格式和长度
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let len = username.chars().count();
    if !(UsernameValidConfig::MIN_LENGTH..=UsernameValidConfig::MAX_LENGTH).contains(&len) {
        return Err(ValidationError::new("invalid_length")
            .with_message(UsernameValidConfig::LEN_ERROR_MSG.into()));
    }
    if !USERNAME_REGEX.is_match(username) {
        return Err(ValidationError::new("invalid_username")
            .with_message(UsernameValidConfig::CHAR_ERROR_MSG.into()));
    }
    Ok(())
}
