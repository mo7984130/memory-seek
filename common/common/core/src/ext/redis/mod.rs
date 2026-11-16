use std::time::Duration;

use deadpool_redis::Pool;
use redis::{AsyncCommands, FromRedisValue, ToSingleRedisArg};

use crate::contextual::ContextualError;

pub trait RedisExt {
    fn del(
        &self,
        key: impl AsRef<str> + Send + Sync,
    ) -> impl Future<Output = Result<(), ContextualError>> + Send;

    fn get_as<T: FromRedisValue + Send + Sync>(
        &self,
        key: impl AsRef<str> + Send + Sync,
    ) -> impl Future<Output = Result<Option<T>, ContextualError>> + Send;

    fn set_ex(
        &self,
        key: impl AsRef<str> + Send + Sync,
        value: impl ToSingleRedisArg + Send + Sync,
        ttl: Duration,
    ) -> impl Future<Output = Result<(), ContextualError>> + Send;
}

impl RedisExt for Pool {
    async fn del(&self, key: impl AsRef<str> + Send + Sync) -> Result<(), ContextualError> {
        let _: () = self.get().await?.del(key.as_ref()).await?;
        Ok(())
    }

    async fn get_as<T: FromRedisValue + Send + Sync>(
        &self,
        key: impl AsRef<str> + Send + Sync,
    ) -> Result<Option<T>, ContextualError> {
        let v: Option<T> = self.get().await?.get(key.as_ref()).await?;
        Ok(v)
    }

    #[inline]
    async fn set_ex(
        &self,
        key: impl AsRef<str> + Send + Sync,
        value: impl ToSingleRedisArg + Send + Sync,
        ttl: Duration,
    ) -> Result<(), ContextualError> {
        let mut conn = self.get().await?;
        let _: () = conn.set_ex(key.as_ref(), value, ttl.as_secs()).await?;
        Ok(())
    }
}
