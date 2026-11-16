pub struct SchemaPrefix(pub &'static str);

#[linkme::distributed_slice]
pub static SCHEMA_PREFIXES: [SchemaPrefix] = [..];

#[macro_export]
macro_rules! scheme_register {
    () => {
        #[linkme::distributed_slice(common_db::SCHEMA_PREFIXES)]
        static SCHEMA_PREFIX: $crate::SchemaPrefix = $crate::SchemaPrefix(env!("CARGO_PKG_NAME"));
    };
}
