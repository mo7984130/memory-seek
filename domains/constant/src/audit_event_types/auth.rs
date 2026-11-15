use crate::audit_event_types::macros::audit_event_types;

audit_event_types! {
    Login => "auth.login",
    Register => "auth.register"
}
