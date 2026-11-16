use common_core::{contextual::ContextualError, type_registry::TypeRegistry};

use crate::config::AppConfig;

#[cfg(feature = "email")]
pub mod email;
#[cfg(feature = "multi_level_cache")]
pub mod multi_level_cache;
#[cfg(feature = "s3_client")]
pub mod s3_client;
#[cfg(feature = "token_cipher")]
pub mod token_cipher;

pub fn init(
    #[allow(unused, reason = "当无lib需求时, config无用")] config: &AppConfig,
    #[allow(unused, reason = "当无lib需求时, registry无用")] registry: &mut TypeRegistry,
) -> Result<(), ContextualError> {
    #[cfg(feature = "email")]
    email::init(config, registry)?;
    #[cfg(feature = "s3_client")]
    s3_client::init(config, registry)?;
    #[cfg(feature = "token_cipher")]
    token_cipher::init(config, registry)?;
    #[cfg(feature = "multi_level_cache")]
    multi_level_cache::init(config, registry)?;

    Ok(())
}
