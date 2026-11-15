use std::borrow::Cow;

use crate::{
    contextual::{ContextualError, ContextualResult, Reason},
    ext::result::ToErrExt,
};

pub trait ContextualOptionExt<T> {
    /// 为 `Some` 返回值，`None` 时返回 warn 级上下文错误
    #[track_caller]
    fn ok_or_warn(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T>;

    /// 为 `Some` 返回值，`None` 时返回 error 级上下文错误
    #[track_caller]
    fn ok_or_error(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T>;
}

impl<T> ContextualOptionExt<T> for Option<T> {
    /// 为 `Some` 返回值，`None` 时返回 warn 级上下文错误
    #[inline]
    #[track_caller]
    fn ok_or_warn(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T> {
        match self {
            Some(v) => Ok(v),
            None => ContextualError::warn_without_source(reason, context).to_err(),
        }
    }

    /// 为 `Some` 返回值，`None` 时返回 error 级上下文错误
    #[inline]
    #[track_caller]
    fn ok_or_error(
        self,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> ContextualResult<T> {
        match self {
            Some(v) => Ok(v),
            None => ContextualError::error_without_source(reason, context).to_err(),
        }
    }
}
