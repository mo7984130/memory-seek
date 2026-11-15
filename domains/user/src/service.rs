use std::sync::Arc;

use bytes::Bytes;
use common_core::{
    contextual::ext::{EmitIfErrExt, IntoContextualExt},
    ext::result::ResultInspectErrAsync,
    time,
};
use common_crypto::rand_util;
use constant::RedisKeys;
use media_validator::{MediaMetadata, MediaValidator};
use share::visual_token::VisualToken;
use type_core::{UserId, viusal::VisualTokenStr};
use type_user::{InviterCodeResponse, UpdateNicknameRequest, UserInfo};

use crate::{
    constant::{GENERATE_INVITER_CODE_MAX_RETRY, INVITER_CODE_LEN, INVITER_CODE_TTL},
    error::UserError,
    state::UserState,
};

pub struct UserService;

// 创建
impl UserService {
    pub async fn generate_inviter_code(
        state: Arc<UserState>,
        user_id: UserId,
    ) -> Result<InviterCodeResponse, UserError> {
        // 循环生成邀请码, 防止冲突
        // 最大生成次数为3
        let mut conn = state.redis.get().await.into_contextual()?;
        for _ in 0..GENERATE_INVITER_CODE_MAX_RETRY {
            let code: String = rand_util::generate_random_uppercase_str(INVITER_CODE_LEN);
            let key = RedisKeys::auth::inviter_code(&code);

            let success: bool = redis::cmd("SET")
                .arg(&key)
                .arg(user_id.0)
                .arg("EX")
                .arg(INVITER_CODE_TTL.as_secs())
                .arg("NX")
                .query_async(&mut conn)
                .await
                .into_contextual()?;

            if success {
                return Ok(InviterCodeResponse {
                    inviter_code: code,
                    expire_at: time::now_after(INVITER_CODE_TTL),
                });
            }
        }

        Err(UserError::InviterCodeGenerateExhausted)
    }
}

// 修改
impl UserService {
    #[tracing::instrument(
        skip_all,
        fields(
            user_id = %user_id,
            new_nickname = %req.new_nickname
        )
    )]
    pub async fn update_nickname(
        state: Arc<UserState>,
        user_id: UserId,
        req: UpdateNicknameRequest,
    ) -> Result<(), UserError> {
        state
            .repo
            .update_nickname(user_id, req.new_nickname)
            .await?;
        Ok(())
    }

    #[tracing::instrument(
            skip_all,
            fields(
                user_id = %user_id
            )
        )]
    pub async fn upload_avatar(
        state: Arc<UserState>,
        user_id: UserId,
        avatar: Bytes,
    ) -> Result<VisualTokenStr, UserError> {
        // 效验图片
        let metadata = MediaValidator::validate_image(avatar.clone())?;

        // 上传图片
        let avatar_key = Self::avatar_file_id(user_id, &metadata);
        state
            .s3_client
            .upload(&avatar_key, &avatar, metadata.kind.mime_type())
            .await
            .into_contextual()?;

        // 更新数据库
        // 如果失败的话, 删除刚才上传的文件
        let old_key = state
            .repo
            .update_avatar(user_id, avatar_key.clone())
            .await
            .inspect_err_async(|_| async {
                state
                    .s3_client
                    .delete(&avatar_key)
                    .await
                    .into_contextual()
                    .emit_if_err();
            })
            .await?;

        // 删除旧头像
        // 错误不返回
        if let Some(old_key) = old_key {
            state
                .s3_client
                .delete(old_key)
                .await
                .into_contextual()
                .emit_if_err();
        }

        // 生成头像token
        VisualToken::image_thumbnail(avatar_key, user_id)
            .encrypt(&state.token_cipher)
            .map_err(UserError::from)
    }
}

// 查询
impl UserService {
    #[tracing::instrument(skip_all, fields(
        user_id = %user_id
    ))]
    pub async fn get_user_self_info(
        state: Arc<UserState>,
        user_id: UserId,
    ) -> Result<UserInfo, UserError> {
        match state.repo.get_user_self_info(user_id).await? {
            Some(info) => Ok(info),
            None => Err(UserError::UserNotFound),
        }
    }
}

// 删除
impl UserService {}

impl UserService {
    fn avatar_file_id(user_id: UserId, metadata: &MediaMetadata) -> String {
        format!(
            "avatars/{}/{}.{}",
            user_id,
            uuid::Uuid::new_v4(),
            metadata.kind.ext()
        )
    }
}
