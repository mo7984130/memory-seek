use std::{borrow::Cow, error::Error};

use crate::contextual::{ContextualError, ContextualResult, Reason};

pub trait ResultExt<T> {
    /// 为 `Ok` 返回值，`Err` 时包装为 error 级上下文错误
    #[track_caller]
    fn contextual_error(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T>;

    /// 为 `Ok` 返回值，`Err` 时包装为 warn 级上下文错误
    #[track_caller]
    fn contextual_warn(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T>;
}

impl<T, Err> ResultExt<T> for std::result::Result<T, Err>
where
    Err: Error + Send + Sync + 'static,
{
    /// 为 `Ok` 返回值，`Err` 时包装为 error 级上下文错误
    #[track_caller]
    fn contextual_error(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(ContextualError::error(reason, context, source)),
        }
    }

    /// 为 `Ok` 返回值，`Err` 时包装为 warn 级上下文错误
    #[track_caller]
    fn contextual_warn(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(ContextualError::warn(reason, context, source)),
        }
    }
}
