use std::time::Duration;

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::BodyExt;

use crate::transport::{Body, Transport, TransportError};

/// 基于 [`reqwest`] 的传输实现。
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    /// 使用外部提供的 [`reqwest::Client`],由调用方决定超时、代理等配置。
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// 返回内部的 [`reqwest::Client`]。
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                // 连接建立超时。
                .connect_timeout(Duration::from_secs(10))
                // 单次读取超时,不会中断长时间持续输出的流式响应。
                .read_timeout(Duration::from_secs(60))
                .build()
                .expect("构建 reqwest Client 失败"),
        }
    }
}

impl Transport for ReqwestTransport {
    async fn send(&self, request: Request<Bytes>) -> Result<Response<Body>, TransportError> {
        let request = reqwest::Request::try_from(request)?;
        let response = self.client.execute(request).await?;

        // 转成 `http::Response<reqwest::Body>`:保留状态码、版本与响应头,
        // 响应体保持流式,由调用方决定收集还是逐帧读取。
        let response: http::Response<reqwest::Body> = response.into();
        let (parts, body) = response.into_parts();

        Ok(Response::from_parts(
            parts,
            Body::stream(body.map_err(TransportError::Io)),
        ))
    }
}
