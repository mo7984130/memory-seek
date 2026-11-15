use common_core::type_registry::FromRegistry;

pub struct AppState {
    #[cfg(feature = "db")]
    pub db: sea_orm::DatabaseConnection,
    #[cfg(feature = "redis")]
    pub redis: deadpool_redis::Pool,
    #[cfg(feature = "email")]
    pub email: email_client::EmailClient,
    #[cfg(feature = "s3_client")]
    pub s3_client: s3_client::S3Client,
    #[cfg(feature = "token_cipher")]
    pub token_cipher: common_crypto::token_cipher::SharedTokenCipher,
}
impl FromRegistry for AppState {
    #[cfg(any(
        feature = "db",
        feature = "redis",
        feature = "email",
        feature = "s3_client",
        feature = "token_cipher"
    ))]
    fn from_registry(
        registry: &common_core::type_registry::TypeRegistry,
    ) -> Result<Self, common_core::contextual::ContextualError> {
        use common_core::{ext::result::ToOkExt, type_registry::GetDepExt};
        Self {
            #[cfg(feature = "db")]
            db: registry.get_dep("App", "Db")?,
            #[cfg(feature = "redis")]
            redis: registry.get_dep("App", "Redis")?,
            #[cfg(feature = "email")]
            email: registry.get_dep("App", "Email")?,
            #[cfg(feature = "s3_client")]
            s3_client: registry.get_dep("App", "S3Client")?,
            #[cfg(feature = "token_cipher")]
            token_cipher: registry.get_dep("App", "TokenCipher")?,
        }
        .to_ok()
    }

    #[cfg(not(any(
        feature = "db",
        feature = "redis",
        feature = "email",
        feature = "s3_client",
        feature = "token_cipher"
    )))]
    fn from_registry(
        _registry: &common_core::type_registry::TypeRegistry,
    ) -> Result<Self, common_core::contextual::ContextualError> {
        Ok(Self {})
    }
}
