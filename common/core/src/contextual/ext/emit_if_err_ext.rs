use crate::contextual::ContextualError;

pub trait EmitIfErrExt<T> {
    fn emit_if_err(self) -> Option<T>;
}

impl<T> EmitIfErrExt<T> for Result<T, ContextualError> {
    fn emit_if_err(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                e.emit();
                None
            }
        }
    }
}
