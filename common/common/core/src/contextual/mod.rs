mod contextual_error;
pub use contextual_error::{ContextualError, Reason};
pub type ContextualResult<T> = std::result::Result<T, ContextualError>;
pub type Result<T> = std::result::Result<T, ContextualError>;

pub mod ext;

pub mod from;
