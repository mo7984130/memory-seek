use common_core::{
    contextual::ContextualError,
    type_registry::{FromRegistry, GetDepExt, TypeRegistry},
};
use common_crypto::token_cipher::SharedTokenCipher;
use deadpool_redis::Pool;
use email_client::EmailClient;
use sea_orm::DatabaseConnection;

use crate::repo::AuthRepo;

/// Auth服务状态
pub struct AuthState {
    pub db: DatabaseConnection,
    pub redis: Pool,
    pub email_client: EmailClient,
    pub token_cipher: SharedTokenCipher,
    pub repo: AuthRepo,
}
impl FromRegistry for AuthState {
    fn from_registry(registry: &TypeRegistry) -> Result<Self, ContextualError> {
        let db: DatabaseConnection = registry.get_dep("Auth", "Db")?;
        let redis: Pool = registry.get_dep("Auth", "Redis")?;

        Ok(Self {
            db: db.clone(),
            redis: redis.clone(),
            email_client: registry.get_dep("Auth", "Email")?,
            token_cipher: registry.get_dep("Auth", "TokenCipher")?,
            repo: AuthRepo::new(db, redis, registry.get_dep("Auth", "CacheConfig")?),
        })
    }
}
