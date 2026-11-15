use std::sync::LazyLock;

use validator::ValidationError;

/// 邮箱正则表达式，匹配 `local@domain.tld` 格式，支持子域名和 `+` 标签
pub static EMAIL_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"^[a-zA-Z0-9_+-]+(\.[a-zA-Z0-9_+-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)*\.[a-zA-Z0-9-]+$",
    )
    .unwrap()
});
/// 校验邮箱格式
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::new("invalid_email").with_message("邮箱格式不正确".into()));
    }
    Ok(())
}
