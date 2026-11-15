use crate::contextual::{ContextualError, Reason};

impl From<std::io::Error> for ContextualError {
    /// 将 `std::io::Error` 转换为上下文错误
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        Self::error(Reason::IoError, "I/O 操作失败", error)
    }
}
