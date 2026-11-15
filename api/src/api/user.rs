use crate::Result;
use crate::{client::ApiClient, storage::Storage, transport::Transport};
use bytes::Bytes;
use type_core::viusal::VisualTokenStr;
use type_user::{InviterCodeResponse, UpdateNicknameRequest, UserInfo};
use validator::Validate;

impl<T: Transport, S: Storage> ApiClient<T, S> {
    pub async fn generate_inviter_code(&self) -> Result<InviterCodeResponse> {
        self.post_empty("/user/inviter-code").await
    }

    pub async fn get_self_info(&self) -> Result<UserInfo> {
        self.get("/user/me").await
    }

    pub async fn update_nickname(&self, req: UpdateNicknameRequest) -> Result<()> {
        req.validate()?;
        self.post("/user/nickname", req).await
    }

    pub async fn upload_avatar(&self, avatar: Bytes) -> Result<VisualTokenStr> {
        self.post_raw("/user/avatar", avatar).await
    }
}
