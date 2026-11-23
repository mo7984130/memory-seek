use std::{borrow::Cow, sync::Arc};

use crate::{Result, error::ApiError, transport::Body};
use bytes::Bytes;
use common_core::{
    ext::result::ToOkExt,
    time::DateTime,
    types::{ErrR, SucR},
};
use http::{
    HeaderValue, Method, Request, Response, StatusCode,
    header::{self},
    request::Builder,
};
use serde::{Serialize, de::DeserializeOwned};
use type_auth::RefreshAccessTokenResponse;
use type_core::UserId;

use crate::{
    config::ApiConfig,
    storage::{AuthStorage, Storage},
    transport::Transport,
};

use tracing::error;

type OnUnauthCallback = Arc<dyn Fn() + Send + Sync + 'static>;

pub struct ApiClient<T: Transport, S: Storage> {
    pub inner: Arc<Inner<T, S>>,
}
impl<T: Transport, S: Storage> Clone for ApiClient<T, S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct Inner<T: Transport, S: Storage> {
    config: ApiConfig,
    pub transport: T,
    pub storage: AuthStorage<S>,
    on_unauth: std::sync::Mutex<Option<OnUnauthCallback>>,
    refresh_lock: tokio::sync::Mutex<()>,
}

impl<T: Transport, S: Storage> ApiClient<T, S> {
    pub fn new(config: ApiConfig, transport: T, storage: S) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                transport,
                storage: AuthStorage::new(storage),
                on_unauth: std::sync::Mutex::new(None),
                refresh_lock: tokio::sync::Mutex::new(()),
            }),
        }
    }

    pub fn register_unauth_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self.inner.on_unauth.lock().unwrap() = Some(Arc::new(callback));
    }

    fn notify_unauth(&self) {
        let callback = self.inner.on_unauth.lock().unwrap().clone();
        if let Some(callback) = callback {
            callback();
        }
    }

    pub async fn get<Res: Send + Sync + DeserializeOwned>(
        &self,
        uri: impl Into<Cow<'static, str>>,
    ) -> Result<Res> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::GET)
            .body(Bytes::new())?;
        Self::parse_envelope(self.exec(req).await?).await
    }

    pub async fn post<Req: Send + Sync + Serialize, Res: Send + Sync + DeserializeOwned>(
        &self,
        uri: impl Into<Cow<'static, str>>,
        body: Req,
    ) -> Result<Res> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::POST)
            .body(Self::serialize(&body)?)?;
        Self::parse_envelope(self.exec(req).await?).await
    }

    pub async fn post_raw<Res: Send + Sync + DeserializeOwned>(
        &self,
        uri: impl Into<Cow<'static, str>>,
        body: Bytes,
    ) -> Result<Res> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::POST)
            .body(body)?;
        Self::parse_envelope(self.exec(req).await?).await
    }

    pub async fn post_empty<Res: Send + Sync + DeserializeOwned>(
        &self,
        uri: impl Into<Cow<'static, str>>,
    ) -> Result<Res> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::POST)
            .body(Bytes::new())?;
        Self::parse_envelope(self.exec(req).await?).await
    }

    pub async fn post_no_auth<Req: Send + Sync + Serialize, Res: Send + Sync + DeserializeOwned>(
        &self,
        uri: impl Into<Cow<'static, str>>,
        body: Req,
    ) -> Result<Res> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::POST)
            .body(Self::serialize(&body)?)?;
        Self::parse_envelope(self.exec_no_auth(req).await?).await
    }

    /// 发起 GET 流式请求,响应体边到达边处理。
    pub async fn get_stream(&self, uri: impl Into<Cow<'static, str>>) -> Result<Response<Body>> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::GET)
            .body(Bytes::new())?;
        self.exec(req).await
    }

    /// 发起 POST 流式请求,响应体边到达边处理。
    pub async fn post_stream<Req: Send + Sync + Serialize>(
        &self,
        uri: impl Into<Cow<'static, str>>,
        body: Req,
    ) -> Result<Response<Body>> {
        let req = self
            .request_builder(&uri.into())
            .method(Method::POST)
            .body(Self::serialize(&body)?)?;
        self.exec(req).await
    }

    async fn exec(&self, mut req: Request<Bytes>) -> Result<Response<Body>> {
        let user_id = self.inner.storage.user_id()?.ok_or_else(|| {
            self.notify_unauth();
            ApiError::NotLoggedIn
        })?;
        let access_token = self.inner.storage.access_token()?;
        let access_token = match access_token {
            None => self.refresh(user_id, None).await?,
            Some(v) => Some(v),
        }
        .ok_or_else(|| {
            self.notify_unauth();
            ApiError::NotLoggedIn
        })?;
        Self::set_auth(&mut req, user_id, &access_token)?;

        let resp = self.inner.transport.send(req.clone()).await?;

        if resp.status() == StatusCode::UNAUTHORIZED {
            let access_token = self
                .refresh(user_id, Some(access_token.as_str()))
                .await?
                .ok_or_else(|| {
                    self.notify_unauth();
                    ApiError::NotLoggedIn
                })?;
            Self::set_auth(&mut req, user_id, &access_token)?;
            let resp = self.inner.transport.send(req).await?;
            if resp.status() == StatusCode::UNAUTHORIZED {
                Err(ApiError::NotLoggedIn)
            } else {
                Ok(resp)
            }
        } else {
            Ok(resp)
        }
    }

    /// 写入认证头。
    fn set_auth(req: &mut Request<Bytes>, user_id: UserId, access_token: &str) -> Result<()> {
        req.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {user_id} {access_token}"))?,
        );

        Ok(())
    }

    async fn exec_no_auth(&self, req: Request<Bytes>) -> Result<Response<Body>> {
        Ok(self.inner.transport.send(req).await?)
    }

    pub(crate) async fn refresh(
        &self,
        user_id: UserId,
        old_access_token: Option<&str>,
    ) -> Result<Option<String>> {
        let _guard = self.inner.refresh_lock.lock().await;

        // 再次检查 Token 是否已经被其他请求刷新
        let current_token = self.inner.storage.access_token()?;

        if current_token.as_deref() != old_access_token {
            return Ok(current_token);
        }

        let refresh_token = self.inner.storage.refresh_token()?.ok_or_else(|| {
            self.notify_unauth();
            ApiError::NotLoggedIn
        })?;

        let (access_token, access_token_expire_at) =
            self.fetch_new_access_token(user_id, refresh_token).await?;
        self.inner
            .storage
            .set_access_token(access_token.clone(), access_token_expire_at)?;

        Ok(Some(access_token))
    }

    async fn fetch_new_access_token(
        &self,
        user_id: UserId,
        refresh_token: String,
    ) -> Result<(String, DateTime)> {
        let req = self
            .request_builder("/auth/token")
            .method(Method::POST)
            .header("X-User-Id", user_id.to_string())
            .header("X-Refresh-Token", refresh_token)
            .body(Bytes::new())?;
        let resp = self.exec_no_auth(req).await?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            self.inner.storage.clear()?;
            self.notify_unauth();
            return Err(ApiError::NotLoggedIn);
        }
        let body = resp.into_body().collect().await?;
        Self::deserialize::<SucR<RefreshAccessTokenResponse>>(&body)?
            .data
            .into_pair()
            .to_ok()
    }

    fn request_builder(&self, uri: &str) -> Builder {
        Builder::new().uri(format!("{}{}", self.inner.config.base_url(), uri))
    }

    pub fn check_login(&self) -> Result<bool> {
        self.inner.storage.check_login()?.to_ok()
    }

    pub fn check_and_notify(&self) -> Result<bool> {
        let is_login_in = self.check_login()?;
        if !is_login_in {
            self.notify_unauth();
        }
        Ok(is_login_in)
    }

    #[inline]
    fn deserialize<V: DeserializeOwned>(bytes: impl AsRef<[u8]>) -> Result<V> {
        serde_json::from_slice::<V>(bytes.as_ref())?.to_ok()
    }

    #[inline]
    fn serialize<V: Serialize + ?Sized>(value: &V) -> Result<Bytes> {
        Ok(Bytes::from(serde_json::to_vec(value)?))
    }

    async fn parse_envelope<V: DeserializeOwned>(resp: Response<Body>) -> Result<V> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.into_body().collect().await?;
            let msg = match Self::deserialize::<ErrR>(&body) {
                Err(e) => {
                    error!("序列化服务器相应失败, 降级: {:?}", e);
                    String::from_utf8_lossy(&body).into_owned()
                }
                Ok(r) => r.msg,
            };
            Err(ApiError::HttpStatus { status, msg })
        } else {
            let body = resp.into_body().collect().await?;
            let r = Self::deserialize::<SucR<V>>(&body)?;
            Ok(r.data)
        }
    }
}
