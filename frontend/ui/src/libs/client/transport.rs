use std::{
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};

use api::transport::{Transport, TransportError, impls::ReqwestTransport};
use bytes::Bytes;
use http_body::{Frame, SizeHint};
use tokio::runtime::{Handle, Runtime};

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();
fn runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("初始化Tokio运行时失败")
    })
}

struct RuntimeBody {
    inner: api::transport::Body,
    handle: Handle,
}

impl http_body::Body for RuntimeBody {
    type Data = Bytes;
    type Error = api::transport::TransportError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.get_mut();
        let _guard = this.handle.enter();
        Pin::new(&mut this.inner).poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

pub struct TokioRuntimeTransport {
    inner: Arc<ReqwestTransport>,
    handle: Handle,
}

impl Default for TokioRuntimeTransport {
    fn default() -> Self {
        let handle = runtime().handle().clone();

        let inner = {
            let _guard = handle.enter();
            Arc::new(ReqwestTransport::default())
        };

        Self { inner, handle }
    }
}

impl TokioRuntimeTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            inner: Arc::new(ReqwestTransport::new(client)),
            handle: runtime().handle().clone(),
        }
    }
}

impl Transport for TokioRuntimeTransport {
    async fn send(
        &self,
        request: api::http::Request<Bytes>,
    ) -> Result<api::http::Response<api::transport::Body>, api::transport::TransportError> {
        let inner = self.inner.clone();

        let resp = self
            .handle
            .spawn(async move { inner.send(request).await })
            .await
            .map_err(|e| TransportError::Other(Box::new(e)))??;

        Ok(resp.map(|body| {
            api::transport::Body::stream(RuntimeBody {
                inner: body,
                handle: self.handle.clone(),
            })
        }))
    }
}
