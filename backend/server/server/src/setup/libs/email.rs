use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use tracing::debug;

use crate::config::AppConfig;

pub fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化 EmailClient");
    let cfg = &config.email;
    let client = email_client::EmailClient::new(cfg)?;
    registry.insert(client);
    Ok(())
}
