use common_core::{
    ext::result::ToOkExt,
    type_registry::{FromRegistry, GetDepExt},
};
use common_crypto::token_cipher::SharedTokenCipher;
use deadpool_redis::Pool;
use multi_level_cache::MultiLevelCache;
use s3_client::S3Client;
use sea_orm::DatabaseConnection;

use crate::repo::UserRepo;

pub struct UserState {
    pub db: DatabaseConnection,
    pub redis: Pool,
    pub s3_client: S3Client,
    pub token_cipher: SharedTokenCipher,
    pub repo: UserRepo,
}
impl FromRegistry for UserState {
    fn from_registry(
        registry: &common_core::type_registry::TypeRegistry,
    ) -> Result<Self, common_core::contextual::ContextualError> {
        let db: DatabaseConnection = registry.get_dep("User", "db")?;
        let redis: Pool = registry.get_dep("User", "redis")?;
        Self {
            db: db.clone(),
            redis: redis.clone(),
            s3_client: registry.get_dep("User", "S3Client")?,
            token_cipher: registry.get_dep("User", "TokenCipher")?,
            repo: UserRepo {
                db: db.clone(),
                redis: redis.clone(),
                token_cipher: registry.get_dep("User", "TokenCipher")?,
                cache_user_self_info: MultiLevelCache::new(
                    redis.clone(),
                    registry.get_dep("User", "CacheConfig")?,
                ),
            },
        }
        .to_ok()
    }
}
