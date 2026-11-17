use validator::ValidationError;

/// 校验登录账号符合用户名或邮箱格式
pub fn validate_account(value: &str) -> Result<(), ValidationError> {
    if value.contains('@') {
        crate::validators::email::validate_email(value)?;
    } else {
        crate::validators::username::validate_username(value)?;
    }
    Ok(())
}
