use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use common_core::contextual::ContextualError;
use futures::{Stream, StreamExt};
use reqwest::StatusCode;
use s3::Bucket;
use s3::{Region, creds::Credentials};

use crate::retry::retry_429;

mod retry;

const CHUNK_SIZE: usize = 256;
const CONCURRENCY: usize = 16;
const DEFAULT_TTL: Duration = Duration::from_hours(1);

struct Inner {
    client: reqwest::Client,
    bucket: Bucket,
    public_url: String,
}

#[derive(Clone)]
pub struct S3Client {
    inner: Arc<Inner>,
}

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("Io错误: {}", 0)]
    Io(#[from] std::io::Error),
    #[error("Oss错误 {}", 0)]
    Oss(#[from] s3::error::S3Error),
    #[error("凭证错误 {}", 0)]
    Credential(#[from] s3::creds::error::CredentialsError),
    #[error("请求错误 {}", 0)]
    Reqwest(#[from] reqwest::Error),
    #[error("请求错误 {}, {}", status, url)]
    Http { url: String, status: StatusCode },
}
impl S3Error {
    /// 判断错误是否为 OSS 限流(HTTP 429 Too Many Requests)；命中后可按 `RETRY_DELAYS` 退避重试
    pub fn is_rate_limited(&self) -> bool {
        match self {
            Self::Http { status, .. } => *status == StatusCode::TOO_MANY_REQUESTS,
            Self::Oss(s3::error::S3Error::HttpFailWithBody(status, _)) => *status == 429,
            _ => false,
        }
    }

    /// 判断错误是否为对象不存在(HTTP 404)
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::Http { status, .. } => *status == StatusCode::NOT_FOUND,
            Self::Oss(s3::error::S3Error::HttpFailWithBody(status, _)) => *status == 404,
            _ => false,
        }
    }
}
impl From<S3Error> for ContextualError {
    #[track_caller]
    fn from(value: S3Error) -> Self {
        ContextualError::error(common_core::contextual::Reason::OssError, "oss错误", value)
    }
}
type Result<T> = std::result::Result<T, S3Error>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub public_url: Option<String>,
    pub force_path_style: bool,
}

/// 流式下载响应: 状态码、范围信息与字节流
pub struct DownloadStream {
    /// 对象存储返回的状态码(200 全量 / 206 部分内容 / 416 范围越界)
    pub status: StatusCode,
    /// 底层响应，调用方从中读取 `Content-Type` / `Content-Range` / `Content-Length`
    /// 以及通过 `bytes_stream()` 获取响应体流。
    pub response: reqwest::Response,
}

impl DownloadStream {
    /// 便捷方法：读取 `Content-Type` 头。
    pub fn content_type(&self) -> Option<&str> {
        self.response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
    }

    /// 便捷方法：读取 `Content-Range` 头(仅 206 / 416 时存在)。
    pub fn content_range(&self) -> Option<&str> {
        self.response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
    }

    /// 便捷方法：读取 `Content-Length`。
    pub fn content_length(&self) -> Option<u64> {
        self.response.content_length()
    }

    /// 将响应体转为字节流。
    pub fn bytes_stream(self) -> impl Stream<Item = Result<Bytes>> + Send + 'static {
        self.response
            .bytes_stream()
            .map(|item| item.map_err(S3Error::from))
    }
}

impl S3Client {
    pub fn new(cfg: &S3Config) -> Result<Self> {
        let region = Region::Custom {
            region: cfg.region.clone(),
            endpoint: cfg.endpoint.clone(),
        };
        let credentials = Credentials::new(
            Some(&cfg.access_key),
            Some(&cfg.secret_key),
            None,
            None,
            None,
        )?;
        let bucket = Bucket::new(&cfg.bucket, region, credentials)?;
        let bucket = if cfg.force_path_style {
            *bucket.with_path_style()
        } else {
            *bucket
        };

        Ok(Self {
            inner: Arc::new(Inner {
                client: reqwest::Client::new(),
                bucket,
                public_url: cfg
                    .public_url
                    .clone()
                    .unwrap_or_else(|| cfg.endpoint.clone())
                    .trim_end_matches('/')
                    .to_string(),
            }),
        })
    }

    pub async fn upload(
        &self,
        key: impl AsRef<str>,
        data: &Bytes,
        content_type: impl AsRef<str>,
    ) -> Result<()> {
        retry_429(|| async {
            self.inner
                .bucket
                .put_object_with_content_type(key.as_ref(), data, content_type.as_ref())
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn upload_file(
        &self,
        key: impl AsRef<str>,
        file_path: &Path,
        content_type: impl AsRef<str>,
    ) -> Result<()> {
        let path = file_path.to_owned();
        retry_429(|| async {
            let mut file = tokio::fs::File::open(&path).await?;
            self.inner
                .bucket
                .put_object_stream_with_content_type(&mut file, key.as_ref(), content_type.as_ref())
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn delete(&self, key: impl AsRef<str>) -> Result<()> {
        retry_429(|| async {
            self.inner.bucket.delete_object(key.as_ref()).await?;
            Ok(())
        })
        .await
    }

    pub async fn delete_batch(&self, keys: Vec<impl AsRef<str>>) -> Result<()> {
        async {
            for concurrent_chunks in keys.chunks(CHUNK_SIZE * CONCURRENCY) {
                let futures: Vec<_> = concurrent_chunks
                    .chunks(CHUNK_SIZE)
                    .map(|chunk| async move {
                        for key in chunk {
                            retry_429(|| async {
                                self.inner.bucket.delete_object(key.as_ref()).await?;
                                Ok(())
                            })
                            .await?;
                        }
                        Ok::<_, S3Error>(())
                    })
                    .collect();

                futures::future::try_join_all(futures).await?;
            }

            Ok(())
        }
        .await
    }

    pub async fn get_url(&self, key: impl AsRef<str>) -> String {
        format!("{}{}", self.inner.public_url, key.as_ref())
    }

    pub async fn get_signed_url(&self, key: impl AsRef<str>, ttl: Duration) -> Result<String> {
        self.get_signed_url_with_param(key, ttl, None).await
    }

    pub async fn get_signed_url_with_param(
        &self,
        key: impl AsRef<str>,
        ttl: Duration,
        process: Option<String>,
    ) -> Result<String> {
        let custom_queries = if let Some(p) = process {
            let mut queries = HashMap::new();
            queries.insert("x-oss-process".to_string(), p);
            Some(queries)
        } else {
            None
        };

        Ok(self
            .inner
            .bucket
            .presign_get(key.as_ref(), ttl.as_secs() as u32, custom_queries)
            .await?)
    }

    pub async fn download(&self, key: impl AsRef<str>) -> Result<Bytes> {
        retry_429(|| async {
            let resp = self.inner.bucket.get_object(key.as_ref()).await?;
            Ok(resp.into_bytes())
        })
        .await
    }

    pub async fn exist(&self, key: impl AsRef<str>) -> Result<bool> {
        let existed = self
            .inner
            .bucket
            .list(key.as_ref().to_string(), None)
            .await?
            .iter()
            .flat_map(|r| r.contents.iter())
            .any(|obj| obj.key.as_str() == key.as_ref());
        Ok(existed)
    }

    pub async fn get_download_stream(&self, key: impl AsRef<str>) -> Result<DownloadStream> {
        self.get_download_stream_with_range(key, None).await
    }

    pub async fn get_download_stream_with_range(
        &self,
        key: impl AsRef<str>,
        range: Option<String>,
    ) -> Result<DownloadStream> {
        let url = self.get_signed_url(key, DEFAULT_TTL).await?;
        let resp = retry_429(|| async {
            let mut req = self.inner.client.get(&url);
            if let Some(range) = range.as_deref() {
                req = req.header(reqwest::header::RANGE, range);
            }
            let resp = req.send().await?;
            Ok(resp)
        })
        .await?;

        // 非 2xx 除 416 外都视为存储错误, 消费响应体后返回完整错误
        if !resp.status().is_success()
            && resp.status() != reqwest::StatusCode::RANGE_NOT_SATISFIABLE
        {
            return Err(S3Error::Http {
                url,
                status: resp.status(),
            });
        }

        Ok(DownloadStream {
            status: resp.status(),
            response: resp,
        })
    }

    pub async fn download_with_process(
        &self,
        key: impl AsRef<str>,
        process: String,
    ) -> Result<Bytes> {
        let url = self
            .get_signed_url_with_param(key, DEFAULT_TTL, Some(process))
            .await?;
        retry_429(|| async {
            let resp = self.inner.client.get(&url).send().await?;
            if !resp.status().is_success() {
                return Err(S3Error::Http {
                    url: url.clone(),
                    status: resp.status(),
                });
            }
            Ok(resp.bytes().await?)
        })
        .await
    }

    /// 获取应用图片处理参数后的流式下载响应
    pub async fn download_stream_with_process(
        &self,
        key: impl AsRef<str>,
        process: String,
    ) -> Result<DownloadStream> {
        let url = self
            .get_signed_url_with_param(key, DEFAULT_TTL, Some(process))
            .await?;

        retry_429(|| async {
            let resp = self.inner.client.get(&url).send().await?;
            if !resp.status().is_success() {
                return Err(S3Error::Http {
                    url: url.clone(),
                    status: resp.status(),
                });
            }
            Ok(DownloadStream {
                status: resp.status(),
                response: resp,
            })
        })
        .await
    }
}
