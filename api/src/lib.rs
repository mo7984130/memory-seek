pub mod api;
pub mod client;
pub mod config;
pub mod storage;
pub mod transport;

pub type Result<T> = std::result::Result<T, client::ApiError>;
