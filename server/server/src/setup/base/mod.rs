use common_core::{contextual::ContextualError, type_registry::TypeRegistry};

use crate::config::AppConfig;

pub mod log;

#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "redis")]
pub mod redis;

#[cfg(any(feature = "db", feature = "redis"))]
pub async fn init(config: &AppConfig) -> Result<TypeRegistry, ContextualError> {
    let mut registry = TypeRegistry::new();

    #[cfg(feature = "db")]
    db::init(config, &mut registry).await?;

    #[cfg(feature = "redis")]
    redis::init(config, &mut registry).await?;

    Ok(registry)
}

#[cfg(not(any(feature = "db", feature = "redis")))]
pub async fn init(_: &AppConfig) -> Result<TypeRegistry, ContextualError> {
    Ok(TypeRegistry::new())
}
