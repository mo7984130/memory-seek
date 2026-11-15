use std::sync::LazyLock;

use validator::ValidationError;

/// 常规字符验证配置，定义允许的字符模式和错误提示信息
pub struct CommonValidConfig;
impl CommonValidConfig {
    pub const NORMAL_CHAR_PATTERN: &'static str = r#"^[^<>/\\"'&@]+$"#;
    pub const NORMAL_CHAR_MSG: &'static str = "不能包含 < > / \\ \" ' & @等特殊符号";
}

static NORMAL_CHAR_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(CommonValidConfig::NORMAL_CHAR_PATTERN).expect("Invalid Normal Char Regex")
});

/// 校验字符串仅包含允许的普通字符
pub fn validate_normal_char(value: &str) -> Result<(), ValidationError> {
    // 空字符串或只包含空格的字符串直接拒绝
    if value.is_empty() || value.trim().is_empty() {
        return Err(ValidationError::new("invalid_characters")
            .with_message(CommonValidConfig::NORMAL_CHAR_MSG.into()));
    }

    if !NORMAL_CHAR_REGEX.is_match(value) {
        return Err(ValidationError::new("invalid_characters")
            .with_message(CommonValidConfig::NORMAL_CHAR_MSG.into()));
    }
    Ok(())
}
