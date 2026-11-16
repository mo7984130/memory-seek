use crate::contextual::{ContextualError, Reason};

impl From<deadpool_redis::PoolError> for ContextualError {
    /// 将 Redis 连接池错误转换为上下文错误
    #[track_caller]
    fn from(error: deadpool_redis::PoolError) -> Self {
        Self::error(Reason::RedisError, "Redis错误", error)
    }
}

impl From<redis::RedisError> for ContextualError {
    /// 将 `redis::RedisError` 转换为上下文错误
    #[track_caller]
    fn from(error: redis::RedisError) -> Self {
        Self::error(Reason::RedisError, "Redis错误", error)
    }
}
