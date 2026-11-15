use common_core::{
    contextual::{ContextualError, ext::ResultExt},
    type_registry::TypeRegistry,
};
use deadpool_redis::PoolConfig;
use serde::Deserialize;
use tracing::{debug, info};

use crate::config::AppConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            url: default_url(),
            max_connections: default_max_connections(),
        }
    }
}
fn default_url() -> String {
    "redis://127.0.0.1:6379".to_string()
}
fn default_max_connections() -> usize {
    16
}

pub async fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化Redis");
    let cfg = &config.redis;
    let mut redis_cfg = deadpool_redis::Config::from_url(&cfg.url);
    redis_cfg.pool = Some(PoolConfig::new(cfg.max_connections));

    let pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .contextual_warn(
            common_core::contextual::Reason::BuildFail,
            "Redis连接池创建失败",
        )?;

    // 测试连接
    let mut conn = pool.get().await?;
    redis::cmd("PING").query_async::<String>(&mut conn).await?;

    registry.insert(pool);
    info!("Redis连接成功");
    Ok(())
}
