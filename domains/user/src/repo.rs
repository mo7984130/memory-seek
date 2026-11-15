use crate::error::{Result, UserError};
use audit::{AuditActor, AuditEvent, recorder::AuditRecorder};
use common_core::{contextual::ext::IntoContextualExt, ext::result::ToOkExt};
use common_crypto::token_cipher::SharedTokenCipher;
use common_db::db_transaction;
use constant::{AuditEventTypes, RedisKeys};
use deadpool_redis::Pool;
use multi_level_cache::MultiLevelCache;
use sea_orm::DatabaseConnection;
use share::visual_token::VisualToken;
use type_core::UserId;
use type_user::UserInfo;

use crate::mapper::UserMapper;

pub struct UserRepo {
    pub db: DatabaseConnection,
    pub redis: Pool,
    pub token_cipher: SharedTokenCipher,
    pub cache_user_self_info: MultiLevelCache<Option<UserInfo>, UserError>,
}

// 创建
impl UserRepo {}

// 修改
impl UserRepo {
    pub async fn update_nickname(&self, user_id: UserId, new_nickname: String) -> Result<()> {
        db_transaction!(&self.db, |txn| {
            UserMapper::update_nickname(txn, user_id, new_nickname).await?;

            AuditRecorder::append(
                txn,
                AuditEvent::new(
                    AuditEventTypes::user::UpdateNickname,
                    AuditActor::User(user_id),
                ),
            )
            .await?;

            Ok::<_, UserError>(())
        })
        .await?;
        self.invalidate_user_info(user_id).await?;
        Ok(())
    }

    /// 更新头像
    /// 返回旧头像key
    pub async fn update_avatar(&self, user_id: UserId, new_key: String) -> Result<Option<String>> {
        let old_key = db_transaction!(&self.db, |txn| {
            let old_key = UserMapper::update_avatar(txn, user_id, new_key).await?;
            AuditRecorder::append(
                txn,
                AuditEvent::new(
                    AuditEventTypes::user::UpdateAvatar,
                    AuditActor::User(user_id),
                ),
            )
            .await?;

            Ok::<_, UserError>(old_key)
        })
        .await?;

        self.invalidate_user_info(user_id).await?;

        Ok(old_key)
    }
}

// 查询
impl UserRepo {
    pub async fn get_user_self_info(&self, user_id: UserId) -> Result<Option<UserInfo>> {
        self.cache_user_self_info
            .get_or_load(
                &RedisKeys::user::cache_user_self_info(user_id),
                RedisKeys::user::CACHE_USER_SELF_INFO_TTL,
                || async move {
                    let user = UserMapper::query_by_id(&self.db, user_id).await?;
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
            .await?
            .to_ok()
    }
}

// 删除
impl UserRepo {
    async fn invalidate_user_info(&self, user_id: UserId) -> Result<()> {
        self.cache_user_self_info
            .invalidate(&RedisKeys::user::cache_user_self_info(user_id))
            .await
            .into_contextual()?;
        Ok(())
    }
}
