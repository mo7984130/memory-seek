use common_core::{contextual::ContextualError, type_registry::TypeRegistry};

use crate::config::AppConfig;

pub mod base;
pub mod libs;

pub async fn init(config: &AppConfig) -> Result<TypeRegistry, ContextualError> {
    let mut registry = base::init(config).await?;
    libs::init(config, &mut registry)?;
    Ok(registry)
}
