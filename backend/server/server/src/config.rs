use std::path::PathBuf;

use common_core::{
    contextual::{ContextualError, ext::ResultExt},
    ext::result::ToErrExt,
};
use config::{Config, Environment, File};
use serde::Deserialize;
use tracing::{debug, info};

pub type ConfigSource = ::config::Config;
#[derive(Deserialize)]
pub struct AppConfig {
    #[cfg(feature = "db")]
    pub database: crate::setup::base::db::Config,
    #[cfg(feature = "redis")]
    #[serde(default)]
    pub redis: crate::setup::base::redis::Config,
    #[cfg(feature = "email")]
    pub email: email_client::EmailClientConfig,
    #[cfg(feature = "s3_client")]
    pub s3_client: s3_client::S3Config,
    #[cfg(feature = "token_cipher")]
    pub token_cipher: common_crypto::token_cipher::TokenCipherConfig,
    #[cfg(feature = "multi_level_cache")]
    #[serde(default)]
    pub multi_level_cache: multi_level_cache::CacheConfig,
}

impl AppConfig {
    pub fn load(
        config_path: Option<String>,
    ) -> Result<(Self, ConfigSource, PathBuf), ContextualError> {
        debug!("加载配置文件");

        let config_path = if let Some(path) = config_path {
            PathBuf::from(path)
        } else if let Ok(path) = std::env::var("MEMORY_SEEK_CONFIG_PATH") {
            PathBuf::from(path)
        } else {
            let local = PathBuf::from("config.yml");

            if local.exists() {
                local
            } else {
                PathBuf::from("/etc/memory-seek-server/config.yml")
            }
        };

        if !config_path.exists() {
            return ContextualError::error_without_source(
                common_core::contextual::Reason::BuildFail,
                format!("配置文件不存在: {}", config_path.display()),
            )
            .to_err();
        }

        let config_source = Config::builder()
            .add_source(File::from(config_path.clone()))
            .add_source(
                Environment::with_prefix("MEMORY_SEEK")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()
            .contextual_error(common_core::contextual::Reason::BuildFail, "构建配置失败")?;

        let config = config_source.clone().try_deserialize().contextual_error(
            common_core::contextual::Reason::BuildFail,
            "反序列化配置失败",
        )?;

        info!("加载配置文件成功: {}", config_path.display());

        Ok((config, config_source, config_path))
    }
}
