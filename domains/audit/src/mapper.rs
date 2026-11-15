use common_core::contextual::Result;
use sea_orm::{ConnectionTrait, EntityTrait};
use type_audit::{ActiveModel, AuditEvent, Entity};

pub struct AuditMapper;

// 创建
impl AuditMapper {
    pub async fn insert_many(
        db: &impl ConnectionTrait,
        events: impl IntoIterator<Item = AuditEvent>,
    ) -> Result<()> {
        Entity::insert_many(events.into_iter().map(ActiveModel::from))
            .exec(db)
            .await?;
        Ok(())
    }
}

// 修改
impl AuditMapper {}

// 查询
impl AuditMapper {}

// 删除
impl AuditMapper {}
