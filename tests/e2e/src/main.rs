use std::error::Error;

use api::{config::ApiConfig, storage::impls::MemoryStorage, transport::impls::ReqwestTransport};

type ApiClient = api::client::ApiClient<ReqwestTransport, MemoryStorage>;
mod error;
mod once;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error:#}");
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let state = server::init(None).await?.state;

    let mut builder = reqwest::Client::builder();
    if let Ok(proxy) = std::env::var("E2E_PROXY") {
        builder = builder.proxy(reqwest::Proxy::all(proxy)?);
    }

    let base_url =
        std::env::var("E2E_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:7984".to_owned());

    let client = ApiClient::new(
        ApiConfig::new(base_url)?,
        ReqwestTransport::new(builder.build()?),
        MemoryStorage::new(),
    );
    once::run(&state, &client).await?;

    Ok(())
}
