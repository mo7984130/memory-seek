pub mod ext;
pub mod into_response;
pub type Result<T> = std::result::Result<T, WebError>;

use std::{borrow::Cow, fmt::Display};

use axum::response::IntoResponse;
use common_core::{contextual::ContextualError, types::R};

#[derive(Debug)]
pub enum WebError {
    BadRequest(Cow<'static, str>),
    Conflict(Cow<'static, str>),
    TooManyRequests(Cow<'static, str>),
    NotFound(Cow<'static, str>),
    Unauthorized(Cow<'static, str>),
    InternalServerError,
}

impl Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(msg)
            | Self::Conflict(msg)
            | Self::TooManyRequests(msg)
            | Self::NotFound(msg)
            | Self::Unauthorized(msg) => f.write_str(msg),
            Self::InternalServerError => f.write_str("服务器内部错误"),
        }
    }
}
impl WebError {
    pub fn bad_request(context: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(context.into())
    }

    pub fn conflict(context: impl Into<Cow<'static, str>>) -> Self {
        Self::Conflict(context.into())
    }

    pub fn too_many_requests(context: impl Into<Cow<'static, str>>) -> Self {
        Self::TooManyRequests(context.into())
    }

    pub fn not_found(context: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(context.into())
    }

    pub fn unauthorized(context: impl Into<Cow<'static, str>>) -> Self {
        Self::Unauthorized(context.into())
    }
}

impl From<ContextualError> for WebError {
    #[track_caller]
    fn from(value: ContextualError) -> Self {
        value.emit();
        Self::InternalServerError
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        R::err(self.status_code().as_u16(), self.to_string().as_str()).into_response()
    }
}
