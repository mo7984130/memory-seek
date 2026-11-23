mod transport;
use std::ops::Deref;

use crate::{
    config::dir,
    error::{Error, Result},
};
use api::{client::ApiClient, config::ApiConfig, storage::impls::FileStorage};
use gpui_kit::{App, Global};
use tracing::error;
pub use transport::TokioRuntimeTransport;

#[derive(Clone)]
pub struct Client(ApiClient<TokioRuntimeTransport, FileStorage>);
impl Global for Client {}
impl Deref for Client {
    type Target = ApiClient<TokioRuntimeTransport, FileStorage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Client {
    pub fn init(cx: &mut App) -> Result<()> {
        let client = ApiClient::new(
            ApiConfig::default(),
            TokioRuntimeTransport::new(
                reqwest::Client::builder()
                    .proxy(reqwest::Proxy::all("http://localhost:8000").map_err(|e| {
                        error!("代理错误: {:?}", e);
                        Error::Start(Box::new(e))
                    })?)
                    .build()
                    .map_err(|e| {
                        error!("构建请求器错误: {:?}", e);
                        Error::Start(Box::new(e))
                    })?,
            ),
            FileStorage::new(dir::config_dir().join("cred.json"))?,
        );
        let this = Self(client);
        cx.set_global(this);
        Ok(())
    }
}
