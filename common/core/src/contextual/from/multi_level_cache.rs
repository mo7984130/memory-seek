use crate::contextual::{ContextualError, Reason};

impl From<multi_level_cache::CacheError> for ContextualError {
    /// 将多级缓存错误转换为 warn 级上下文错误
    #[track_caller]
    fn from(error: multi_level_cache::CacheError) -> Self {
        Self::warn(Reason::RedisError, "缓存错误", error)
    }
}
