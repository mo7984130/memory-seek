use std::time::Duration;

pub const ACCESS_TOKEN_LEN: usize = 32;
pub const ACCESS_TOKEN_TTL: Duration = Duration::from_hours(2);
pub const ACCESS_TOKEN_REFRESH_TTL: Duration = Duration::from_secs(2);
pub const CACHE_REFRESH_TOKEN_TTL: Duration = Duration::from_hours(4);
pub const REFRESH_TOKEN_LEN: usize = 32;
pub const REFRESH_TOKEN_TTL: Duration = Duration::from_hours(30 * 24);
pub const EMAIL_CODE_LEN: usize = 6;
pub const EMAIL_CODE_EXPIRATION: Duration = Duration::from_mins(10);
