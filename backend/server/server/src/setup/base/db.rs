use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use common_db::SCHEMA_PREFIXES;
use sea_orm::{ConnectOptions, Database};
use serde::Deserialize;
use tracing::{debug, info};

use crate::config::AppConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}
fn default_max_connections() -> u32 {
    64
}

pub async fn init(config: &AppConfig, registry: &mut TypeRegistry) -> Result<(), ContextualError> {
    debug!("初始化数据库");
    let cfg = &config.database;
    let mut opt = ConnectOptions::new(&cfg.url);
    opt.max_connections(cfg.max_connections);
    let db = Database::connect(opt).await?;

    let mut synced = false;
    for prefix in SCHEMA_PREFIXES {
        synced = true;
        db.get_schema_registry(prefix.0).sync(&db).await?;
    }
    if !synced {
        // 没有实体 crate 登记前缀说明链接图里没有实体(或忘记登记), 表结构不会被同步。
        tracing::warn!("没有实体 crate 登记 schema 前缀, 表结构不会被同步");
    }

    registry.insert(db);
    info!("数据库初始化成功");
    Ok(())
}
