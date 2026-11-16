use std::{borrow::Cow, error::Error, fmt::Debug, panic::Location};
use tracing::Level;

#[derive(Debug)]
pub struct ContextualError {
    level: Level,
    loc: &'static Location<'static>,
    reason: Reason,
    context: Cow<'static, str>,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}
impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.reason, self.context)?;
        if let Some(src) = &self.source {
            write!(f, " ({src})")?;
        }
        Ok(())
    }
}
impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(source) => Some(source.as_ref()),
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum Reason {
    // 构建 / 配置
    BuildFail,
    // IO
    IoError,
    // 数据库
    DbError,
    DbDataError,
    // Redis
    RedisError,
    // 异步运行时
    TokioError,
    // 密码哈希
    PasswordHasherError,
    // 邮件
    EmailSendFail,
    // Oss
    OssError,
    // 加密（只用于内部数据）
    EncryptFail,
    // 序列化 (只用于内部数据)
    SerializeFail,
}

impl ContextualError {
    pub fn emit(self) -> Reason {
        let Self {
            level,
            loc,
            reason,
            context,
            source,
        } = self;
        macro_rules! emit {
            ($lvl:ident) => {
                match source {
                    Some(e) => tracing::$lvl!(
                        reason = ?reason,
                        error = ?e,
                        "{context} ({}:{})",
                        loc.file(),
                        loc.line()
                    ),
                    None => tracing::$lvl!(
                        reason = ?reason,
                        "{context} ({}:{})",
                        loc.file(),
                        loc.line()
                    ),
                }
            };
        }

        match level {
            Level::ERROR => emit!(error),
            Level::WARN => emit!(warn),
            Level::INFO => emit!(info),
            Level::DEBUG => emit!(debug),
            Level::TRACE => emit!(trace),
        }

        reason
    }

    #[inline]
    #[track_caller]
    pub fn with_source(
        level: Level,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            level,
            reason,
            context: context.into(),
            source: Some(Box::new(source)),
            loc: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn without_source(
        level: Level,
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            level,
            reason,
            context: context.into(),
            source: None,
            loc: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn error(
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(Level::ERROR, reason, context, source)
    }

    #[inline]
    #[track_caller]
    pub fn error_without_source(reason: Reason, context: impl Into<Cow<'static, str>>) -> Self {
        Self::without_source(Level::ERROR, reason, context)
    }

    #[inline]
    #[track_caller]
    pub fn warn(
        reason: Reason,
        context: impl Into<Cow<'static, str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(Level::WARN, reason, context, source)
    }

    #[inline]
    #[track_caller]
    pub fn warn_without_source(reason: Reason, context: impl Into<Cow<'static, str>>) -> Self {
        Self::without_source(Level::WARN, reason, context)
    }
}
