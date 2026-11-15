use crate::contextual::ContextualError;

pub trait FlattenResultExt<T> {
    fn flatten_contextual(self) -> Result<T, ContextualError>;
}

impl<T> FlattenResultExt<T> for Result<Result<T, ContextualError>, ContextualError> {
    fn flatten_contextual(self) -> Result<T, ContextualError> {
        self.and_then(|inner| inner)
    }
}
