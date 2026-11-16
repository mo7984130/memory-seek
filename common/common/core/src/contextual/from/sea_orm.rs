use crate::contextual::{ContextualError, Reason};

impl From<sea_orm::error::DbErr> for ContextualError {
    #[track_caller]
    fn from(value: sea_orm::error::DbErr) -> Self {
        ContextualError::error(Reason::DbError, "数据库错误", value)
    }
}
