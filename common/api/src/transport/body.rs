use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use http_body::{Frame, SizeHint};
use http_body_util::BodyExt;

use crate::transport::TransportError;

pub enum Body {
    Empty,
    Bytes(Option<bytes::Bytes>),
    Stream(
        Pin<Box<dyn http_body::Body<Data = bytes::Bytes, Error = TransportError> + Send + 'static>>,
    ),
}

impl Body {
    #[inline]
    pub fn empty() -> Self {
        Self::Empty
    }

    #[inline]
    pub fn from_bytes(bytes: impl Into<bytes::Bytes>) -> Self {
        Self::Bytes(Some(bytes.into()))
    }

    #[inline]
    pub fn stream<B>(body: B) -> Self
    where
        B: http_body::Body<Data = bytes::Bytes, Error = TransportError> + Send + 'static,
    {
        Self::Stream(Box::pin(body))
    }

    /// 将响应体完整读取为字节。
    ///
    /// 流式响应体会被全量缓冲,大响应或事件流请改用逐帧读取。
    pub async fn collect(self) -> Result<bytes::Bytes, TransportError> {
        Ok(BodyExt::collect(self).await?.to_bytes())
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Body::Empty => f.write_str("Body::Empty"),
            Body::Bytes(bytes) => f
                .debug_tuple("Body::Bytes")
                .field(&bytes.as_ref().map(|bytes| bytes.len()))
                .finish(),
            Body::Stream(_) => f.write_str("Body::Stream"),
        }
    }
}

impl http_body::Body for Body {
    type Data = bytes::Bytes;
    type Error = TransportError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Body::Empty => Poll::Ready(None),
            Body::Bytes(bytes) => Poll::Ready(bytes.take().map(|bytes| Ok(Frame::data(bytes)))),
            Body::Stream(body) => body.as_mut().poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Body::Empty => true,
            Body::Bytes(bytes) => bytes.is_none(),
            Body::Stream(body) => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Body::Empty => SizeHint::with_exact(0),
            Body::Bytes(bytes) => {
                SizeHint::with_exact(bytes.as_ref().map_or(0, |bytes| bytes.len() as u64))
            }
            Body::Stream(body) => body.size_hint(),
        }
    }
}
