use bytes::Bytes;
use server::state::AppState;
use std::fs::{self};
use tracing::info;
use type_auth::UpdateNicknameRequest;

use crate::{ApiClient, error::E2eError};

pub async fn run(_state: &AppState, client: &ApiClient) -> Result<(), E2eError> {
    info!("生成邀请码: {:?}", client.generate_inviter_code().await?);
    info!("获取个人信息: {:?}", client.get_self_info().await?);
    info!(
        "修改昵称: {:?}\n{:?}",
        client
            .update_nickname(UpdateNicknameRequest {
                new_nickname: "NewNickname1".to_string()
            })
            .await?,
        client.get_self_info().await?
    );
    info!(
        "更新头像: {:?}\n{:?}",
        client
            .upload_avatar(Bytes::from_owner(fs::read(
                "./tests/e2e/assets/avatar.jpg"
            )?))
            .await?,
        client.get_self_info().await?
    );
    Ok(())
}
