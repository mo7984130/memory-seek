use common_core::contextual::Result;
use sea_orm::DatabaseTransaction;
use type_audit::AuditEvent;

use crate::mapper::AuditMapper;

pub struct AuditRecorder;

// 创建
impl AuditRecorder {
    pub async fn append(txn: &DatabaseTransaction, event: AuditEvent) -> Result<()> {
        Self::appends(txn, [event]).await
    }
    pub async fn appends(
        txn: &DatabaseTransaction,
        events: impl IntoIterator<Item = AuditEvent>,
    ) -> Result<()> {
        AuditMapper::insert_many(txn, events).await
    }
}

// 修改
impl AuditRecorder {}

// 查询
impl AuditRecorder {}

// 删除
impl AuditRecorder {}
