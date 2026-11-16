use std::net::IpAddr;

use common_core::contextual::{ContextualError, Reason, ext::ResultExt};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WebServerConfig {
    #[serde(default)]
    pub server: ServerConfig,
}
impl WebServerConfig {
    pub fn load(source: &server::config::ConfigSource) -> Result<Self, ContextualError> {
        source
            .clone()
            .try_deserialize()
            .contextual_error(Reason::BuildFail, "解析配置失败")
    }
}

#[derive(Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}
impl ServerConfig {
    pub fn server_addr(&self) -> String {
        let host = self.host.trim_matches(|c: char| c == '[' || c == ']');
        let port = self.port;
        match host.parse::<IpAddr>() {
            Ok(IpAddr::V6(ip)) => format!("[{ip}]:{port}"),
            Ok(IpAddr::V4(ip)) => format!("{ip}:{port}"),
            Err(_) => format!("{host}:{port}"),
        }
    }
}
fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    7984
}
