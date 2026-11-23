mod body;
pub use body::Body;

pub mod impls;

use std::{error::Error, sync::Arc};

use bytes::Bytes;
use http::{Request, Response};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// 请求构建失败(URI、请求头等非法),与网络状态无关。
    #[error("请求构建失败")]
    Build(#[from] http::Error),

    /// 网络传输失败,流式响应体读取过程中的错误也归入此类。
    #[error("传输失败")]
    Io(#[from] reqwest::Error),

    #[error("其他错误")]
    Other(#[from] Box<dyn Error + Send + Sync>),
}

pub trait Transport: Send + Sync {
    fn send(
        &self,
        request: Request<Bytes>,
    ) -> impl Future<Output = Result<Response<Body>, TransportError>> + Send;
}

impl<T: Transport + ?Sized> Transport for Arc<T> {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Body>, TransportError> {
        (**self).send(request).await
    }
}
impl<T: Transport + ?Sized> Transport for Box<T> {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Body>, TransportError> {
        (**self).send(request).await
    }
}
