use std::{path::PathBuf, sync::Arc};

use common_core::{
    contextual::ContextualError,
    type_registry::{FromRegistry, TypeRegistry},
};
use tracing::info;

use crate::{config::ConfigSource, state::AppState};

pub mod config;
mod setup;
pub mod state;

pub struct InitBundle {
    pub registry: TypeRegistry,
    pub state: Arc<AppState>,
    pub config_source: ConfigSource,
    pub config_path: PathBuf,
}
pub async fn init(config_path: Option<String>) -> Result<InitBundle, ContextualError> {
    // 日志应当最先初始化
    setup::base::log::init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        commit = env!("MEMORY_SEEK_GIT_COMMIT"),
        "寻忆服务启动中"
    );

    let (config, config_source, config_path) = config::AppConfig::load(config_path)?;

    let registry = setup::init(&config).await?;
    let state = AppState::new_arc(&registry)?;
    info!("服务状态初始化完成");

    Ok(InitBundle {
        registry,
        state,
        config_source,
        config_path,
    })
}
