use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use common_crypto::token_cipher::SharedTokenCipher;
use tracing::{debug, info};

use crate::config::AppConfig;

pub fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化TokenCipher");
    let cfg = &config.token_cipher;
    let token_cipher = common_crypto::token_cipher::TokenCipher::new(cfg);
    let shared = SharedTokenCipher::from(token_cipher);
    registry.insert(shared);
    info!("初始化TokenCipher成功");
    Ok(())
}
