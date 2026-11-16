use audit::{AuditActor, AuditEvent, recorder::AuditRecorder};
use common_core::{
    contextual::{ContextualError, ext::IntoContextualExt},
    ext::result::ToOkExt,
    time::DateTime,
    type_registry::{GetDepExt, TypeRegistry},
};
use common_crypto::token_cipher::SharedTokenCipher;
use common_db::db_transaction;
use constant::{AuditEventTypes, RedisKeys};
use deadpool_redis::Pool;
use multi_level_cache::{CacheConfig, MultiLevelCache};
use sea_orm::DatabaseConnection;
use share::visual_token::VisualToken;
use type_auth::{UserInfo, UserRecord};
use type_core::UserId;

use crate::{constant::CACHE_REFRESH_TOKEN_TTL, error::AuthError, mapper::AuthMapper};

pub struct AuthRepo {
    db: DatabaseConnection,
    cache_user_self_info: MultiLevelCache<Option<UserInfo>, AuthError>,
    cache_user_refresh_token: MultiLevelCache<Option<String>, AuthError>,
    token_cipher: SharedTokenCipher,
}
impl TryFrom<&TypeRegistry> for AuthRepo {
    type Error = ContextualError;

    fn try_from(registry: &TypeRegistry) -> Result<Self, Self::Error> {
        let redis: Pool = registry.get_dep("AuthRepo", "Redis")?;
        let cache_config: CacheConfig = registry.get_dep("AuthRepo", "CacheConfig")?;
        let cache_user_refresh_token = MultiLevelCache::new_with_name(
            "user_refresh_token",
            redis.clone(),
            cache_config.l2_only(),
        );
        let cache_user_self_info =
            MultiLevelCache::new_with_name("user_self_info", redis.clone(), cache_config);
        Ok(Self {
            db: registry.get_dep("AuthRepo", "Db")?,
            cache_user_self_info,
            cache_user_refresh_token,
            token_cipher: registry.get_dep("AuthRepo", "SharedTokenCipher")?,
        })
    }
}

// 创建
impl AuthRepo {}

// 修改
impl AuthRepo {
    pub async fn update_refresh_token(
        &self,
        user_id: UserId,
        refresh_token: &str,
        refresh_token_expire_at: DateTime,
    ) -> Result<UserRecord, AuthError> {
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
            .await
            .into_contextual()?;
        Ok(user)
    }

    pub async fn update_nickname(
        &self,
        user_id: UserId,
        new_nickname: String,
    ) -> Result<(), AuthError> {
        db_transaction!(&self.db, |txn| {
            AuthMapper::update_nickname(txn, user_id, new_nickname).await?;

            AuditRecorder::append(
                txn,
                AuditEvent::new(
                    AuditEventTypes::user::UpdateNickname,
                    AuditActor::User(user_id),
                ),
            )
            .await?;

            Ok::<_, AuthError>(())
        })
        .await?;
        self.invalidate_user_info(user_id).await?;
        Ok(())
    }

    /// 返回旧头像key
    pub async fn update_avatar(
        &self,
        user_id: UserId,
        new_key: String,
    ) -> Result<Option<String>, AuthError> {
        let old_key = db_transaction!(&self.db, |txn| {
            let old_key = AuthMapper::update_avatar(txn, user_id, new_key).await?;
            AuditRecorder::append(
                txn,
                AuditEvent::new(
                    AuditEventTypes::user::UpdateAvatar,
                    AuditActor::User(user_id),
                ),
            )
            .await?;

            Ok::<_, AuthError>(old_key)
        })
        .await?;

        self.invalidate_user_info(user_id).await?;

        Ok(old_key)
    }
}

// 查询
impl AuthRepo {
    pub async fn get_refresh_token(&self, user_id: UserId) -> Result<Option<String>, AuthError> {
        self.cache_user_refresh_token
            .get_or_load(
                &RedisKeys::auth::cache_user_refresh_token(user_id),
                CACHE_REFRESH_TOKEN_TTL,
                || async move {
                    AuthMapper::query_refresh_token(&self.db, user_id)
                        .await?
                        .to_ok()
                },
            )
            .await
    }

    pub async fn get_user_self_info(&self, user_id: UserId) -> Result<Option<UserInfo>, AuthError> {
        self.cache_user_self_info
            .get_or_load(
                &RedisKeys::user::cache_user_self_info(user_id),
                RedisKeys::user::CACHE_USER_SELF_INFO_TTL,
                || async move {
                    let user = AuthMapper::query_by_id(&self.db, user_id).await?;
                    match user {
                        None => None,
                        Some(user) => Some(UserInfo::from_record(user, |file_id| {
                            VisualToken::image_thumbnail(file_id, user_id)
                                .encrypt(&self.token_cipher)
                        })?),
                    }
                    .to_ok()
                },
            )
            .await
    }
}

// 删除
impl AuthRepo {
    async fn invalidate_user_info(&self, user_id: UserId) -> Result<(), AuthError> {
        self.cache_user_self_info
            .invalidate(&RedisKeys::user::cache_user_self_info(user_id))
            .await
            .into_contextual()?;
        Ok(())
    }
}
