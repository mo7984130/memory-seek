use common_core::{
    contextual::{ContextualError, ContextualResult, Reason},
    time::{self, DateTime},
};
use serde_json::Value;
use type_core::{AdminId, UserId};

pub enum AuditActor {
    System,
    Admin(AdminId),
    User(UserId),
}
impl AuditActor {
    pub fn into_parts(self) -> (String, Option<i64>) {
        match self {
            Self::System => ("system".to_string(), None),
            Self::Admin(id) => ("admin".to_string(), Some(id.inner().0)),
            Self::User(id) => ("user".to_string(), Some(id.0)),
        }
    }

    pub fn from_parts(actor_type: &str, actor_id: Option<i64>) -> ContextualResult<Self> {
        match (actor_type, actor_id) {
            ("system", None) => Ok(Self::System),

            ("admin", Some(id)) => {
                let user_id = UserId(id);
                let admin_id = AdminId::new(user_id).ok_or_else(|| {
                    ContextualError::error_without_source(
                        Reason::DbDataError,
                        "数据库的审计数据管理员Id错误",
                    )
                })?;

                Ok(Self::Admin(admin_id))
            }

            ("user", Some(id)) => Ok(Self::User(UserId(id))),

            _ => Err(ContextualError::error_without_source(
                Reason::DbDataError,
                "数据库的审计数据错误",
            )),
        }
    }
}

pub struct AuditEvent {
    pub event_type: String,
    pub actor: AuditActor,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub detail: Option<Value>,
    pub occurred_at: DateTime,
}
impl AuditEvent {
    pub fn new(event_type: impl Into<String>, actor: AuditActor) -> Self {
        Self {
            event_type: event_type.into(),
            actor,
            target_type: None,
            target_id: None,
            detail: None,
            occurred_at: time::now(),
        }
    }

    pub fn with_target(
        mut self,
        target_type: impl Into<String>,
        target_id: impl Into<Option<i64>>,
    ) -> Self {
        self.target_type = Some(target_type.into());
        self.target_id = target_id.into();
        self
    }
}

#[cfg(feature = "orm")]
mod entity {
    use common_core::ext::result::ToOkExt;
    use sea_orm::{
        ActiveValue::{NotSet, Set},
        prelude::*,
    };
    use type_core::ids::AuditId;

    use super::*;

    common_db::scheme_register!();

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "audit_event")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub event_id: AuditId,

        #[sea_orm(indexed)]
        pub event_type: String,

        /// 操作者类型
        #[sea_orm(indexed)]
        pub actor_type: String,

        /// 操作者 ID，System 为 NULL
        #[sea_orm(indexed)]
        pub actor_id: Option<i64>,

        /// 目标类型
        pub target_type: Option<String>,

        /// 目标 ID
        pub target_id: Option<i64>,

        #[sea_orm(column_type = "JsonBinary")]
        pub detail: Option<Json>,

        #[sea_orm(indexed)]
        #[sea_orm(default_expr = "sea_orm::sea_query::Expr::current_timestamp()")]
        pub created_at: common_core::time::DateTime,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}

    pub struct AuditRecord {
        pub id: AuditId,
        pub event_type: String,
        pub actor: AuditActor,
        pub target_type: Option<String>,
        pub target_id: Option<i64>,
        pub detail: Option<serde_json::Value>,
        pub occurred_at: common_core::time::DateTime,
    }
    impl TryFrom<Model> for AuditRecord {
        type Error = ContextualError;
        fn try_from(v: Model) -> Result<Self, Self::Error> {
            Self {
                id: v.event_id,
                event_type: v.event_type,
                actor: AuditActor::from_parts(&v.actor_type, v.actor_id)?,
                target_type: v.target_type,
                target_id: v.target_id,
                detail: v.detail,
                occurred_at: v.created_at,
            }
            .to_ok()
        }
    }

    impl From<AuditEvent> for ActiveModel {
        fn from(v: AuditEvent) -> Self {
            let (actor_type, actor_id) = v.actor.into_parts();
            Self {
                event_id: NotSet,
                event_type: Set(v.event_type),
                actor_type: Set(actor_type),
                actor_id: Set(actor_id),
                target_type: Set(v.target_type),
                target_id: Set(v.target_id),
                detail: Set(v.detail),
                created_at: Set(v.occurred_at),
            }
        }
    }
}
#[cfg(feature = "orm")]
pub use entity::*;
