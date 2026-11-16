use crate::audit_event_types::macros::audit_event_types;

audit_event_types! {
    UpdateNickname => "user.update_nickname",
    UpdateAvatar => "user.update_avatar"
}
