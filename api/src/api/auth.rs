use type_user::{LoginRequest, LoginResponse, RegisterRequest, SendEmailRequest, UserInfo};
use validator::Validate;

use crate::{
    Result,
    client::{ApiClient, ApiError},
    storage::Storage,
    transport::Transport,
};

impl<T: Transport, S: Storage> ApiClient<T, S> {
    pub async fn register(&self, req: RegisterRequest) -> Result<UserInfo> {
        req.validate()?;
        self.post_no_auth("/auth/register", req).await
    }

    pub async fn send_email_code(&self, req: SendEmailRequest) -> Result<()> {
        req.validate()?;
        self.post_no_auth("/auth/email-code", req).await
    }

    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse> {
        req.validate()?;
        let resp = self.login_only(req).await?;
        self.inner.storage.login(&resp)?;
        Ok(resp)
    }

    // 仅发送登录请求, 无副作用
    pub async fn login_only(&self, req: LoginRequest) -> Result<LoginResponse> {
        let resp = self.post_no_auth("/auth/login", req).await?;
        Ok(resp)
    }

    // 移除access_token, 后续由client自动刷新, 供测试使用
    pub async fn remove_access_token(&self) -> Result<()> {
        self.inner.storage.remove_access_token()?;
        Ok(())
    }

    pub async fn refresh_access_token(&self) -> Result<String> {
        self.refresh(
            self.inner.storage.user_id()?.ok_or(ApiError::NotLoggedIn)?,
            self.inner.storage.access_token()?.as_deref(),
        )
        .await?
        .ok_or(ApiError::NotLoggedIn)
    }
}
