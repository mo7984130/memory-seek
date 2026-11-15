use type_core::UserId;

#[inline]
pub fn user_access_token(user_id: UserId) -> String {
    format!("a:u:at:{}", user_id)
}

#[inline]
pub fn user_access_token_refresh(user_id: UserId) -> String {
    format!("a:u:at:r:{}", user_id)
}

#[inline]
pub fn cache_user_refresh_token(user_id: UserId) -> String {
    format!("a:c:rt:{}", user_id)
}

#[inline]
pub fn inviter_code(code: &str) -> String {
    //auth:inviter:code
    format!("a:i:c:{}", code)
}

#[inline]
pub fn email_verify_code(email: &str) -> String {
    //auth:verify:email
    format!("a:v:e:{}", email)
}
