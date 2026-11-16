pub mod auth;
pub mod user;

pub(crate) mod macros {
    macro_rules! audit_event_types {
        ($($variant:ident => $value:literal),* $(,)?) => {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Types {
                $($variant),*
            }

            impl From<Types> for String {
                fn from(v: Types) -> Self {
                    match v {
                        $(Types::$variant => $value),*
                    }.to_string()
                }
            }

            pub use Types::*;
        };
    }

    pub(crate) use audit_event_types;
}
