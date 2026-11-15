use validator::ValidationError;

/// 密码验证配置，定义长度范围、复杂性模式和错误提示信息
pub struct PasswordValidConfig;
impl PasswordValidConfig {
    pub const MIN: usize = 8;
    pub const MAX: usize = 64;
    /// 判断密码是否非空且同时包含字母与数字
    pub fn is_valid_password(password: &str) -> bool {
        if password.is_empty() {
            return false;
        }

        let mut has_letter = false;
        let mut has_digit = false;

        for c in password.chars() {
            if c.is_whitespace() {
                return false;
            }

            has_letter |= c.is_ascii_alphabetic();
            has_digit |= c.is_ascii_digit();
        }

        has_letter && has_digit
    }
    pub const LEN_MSG: &'static str = "密码长度需在 8 到 64 位之间";
    pub const PATTERN_MSG: &'static str = "密码必须包含字母和数字";
}

/// 校验密码长度和字符要求
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    // 1. 非空检查 (NotBlank)
    if password.trim().is_empty() {
        return Err(ValidationError::new("required").with_message("密码不能为空".into()));
    }

    // 2. 长度检查 (Length)
    let len = password.chars().count();
    if !(PasswordValidConfig::MIN..=PasswordValidConfig::MAX).contains(&len) {
        return Err(ValidationError::new("invalid_length")
            .with_message(PasswordValidConfig::LEN_MSG.into()));
    }

    // 3. 复杂性检查 (Pattern: 字母 + 数字)
    match PasswordValidConfig::is_valid_password(password) {
        true => Ok(()),
        false => Err(ValidationError::new("invalid_password")
            .with_message(PasswordValidConfig::PATTERN_MSG.into())),
    }
}
