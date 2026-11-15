use std::time::Duration;

use type_core::UserId;

pub fn cache_user_self_info(user_id: UserId) -> String {
    // u:user_self_info:{}
    format!("u:u_s_i:{}", user_id)
}
pub const CACHE_USER_SELF_INFO_TTL: Duration = Duration::from_hours(24);
