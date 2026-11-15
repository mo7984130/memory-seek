use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use tracing::{debug, info};

use crate::config::AppConfig;

pub fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化S3Client");
    let cfg = &config.s3_client;
    let client = s3_client::S3Client::new(cfg)?;
    registry.insert(client);
    info!("初始化S3Client成功");
    Ok(())
}
