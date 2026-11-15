use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use tracing::debug;

use crate::config::AppConfig;

pub fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化 MultiLevelCache");
    let cfg = &config.multi_level_cache;
    registry.insert(*cfg);
    Ok(())
}
