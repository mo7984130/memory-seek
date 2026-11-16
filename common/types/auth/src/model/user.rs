use crate::validators::normal::validate_normal_char;
use common_core::time::DateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviterCodeResponse {
    pub inviter_code: String,
    pub expire_at: DateTime,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct UpdateNicknameRequest {
    #[validate(
        length(min = 1, max = 32, message = "昵称长度在 1 到 32 个字符"),
        custom(function = "validate_normal_char")
    )]
    pub new_nickname: String,
}
