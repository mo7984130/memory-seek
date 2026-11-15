use std::error::Error;

use memory_seek_api::{
    config::ApiConfig, storage::impls::FileStorage, transport::impls::ReqwestTransport,
};

type ApiClient = memory_seek_api::client::ApiClient<ReqwestTransport, FileStorage>;
pub const LOGIN_CRED: (&str, &str) = ("Test123456", "Test123456");
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

    let client = ApiClient::new(
        ApiConfig::new("http://localhost:7984")?,
        ReqwestTransport::new(
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all("http://127.0.0.1:8000").unwrap())
                .build()?,
        ),
        FileStorage::new("./tests/e2e/cred.json")?,
    );
    once::run(&state, &client).await?;

    Ok(())
}
