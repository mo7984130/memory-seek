use crate::contextual::{ContextualError, ContextualResult};

pub trait IntoContextualExt<T, E> {
    fn into_contextual(self) -> ContextualResult<T>;
}
impl<T, E: Into<ContextualError>> IntoContextualExt<T, E> for Result<T, E> {
    #[inline]
    fn into_contextual(self) -> ContextualResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }
}
