use common_core::{contextual::ContextualError, time::DateTime};
use constant::RedisKeys;
use deadpool_redis::Pool;
use multi_level_cache::{CacheConfig, MultiLevelCache};
use sea_orm::DatabaseConnection;
use tracing::debug;
use type_core::UserId;
use type_user::UserRecord;

use crate::{constant::CACHE_REFRESH_TOKEN_TTL, mapper::AuthMapper};

pub struct AuthRepo {
    db: DatabaseConnection,
    cache_user_refresh_token: MultiLevelCache<Option<String>, ContextualError>,
}

impl AuthRepo {
    pub fn new(db: DatabaseConnection, redis: Pool, config: CacheConfig) -> Self {
        let cache_user_refresh_token =
            MultiLevelCache::new_with_name("user_refresh_token", redis.clone(), config.l2_only());
        Self {
            db: db.clone(),
            cache_user_refresh_token,
        }
    }

    pub async fn update_refresh_token(
        &self,
        user_id: UserId,
        refresh_token: &str,
        refresh_token_expire_at: DateTime,
    ) -> Result<UserRecord, ContextualError> {
        let user = AuthMapper::update_refresh_token(
            &self.db,
            user_id,
            refresh_token.to_owned(),
            refresh_token_expire_at,
        )
        .await?;
        self.cache_user_refresh_token
            .put(
                &RedisKeys::auth::cache_user_refresh_token(user_id),
                Some(refresh_token.to_owned()),
                CACHE_REFRESH_TOKEN_TTL,
            )
            .await?;
        Ok(user)
    }

    pub async fn get_refresh_token(
        &self,
        user_id: UserId,
    ) -> Result<Option<String>, ContextualError> {
        self.cache_user_refresh_token
            .get_or_load(
                &RedisKeys::auth::cache_user_refresh_token(user_id),
                CACHE_REFRESH_TOKEN_TTL,
                || async move {
                    let refresh_token = AuthMapper::query_refresh_token(&self.db, user_id).await?;
                    debug!("{} 的 refresh_token db数据为 {:?}", user_id, refresh_token);
                    Ok(refresh_token)
                },
            )
            .await
    }
}
