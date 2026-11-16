use axum::{Extension, extract::FromRequestParts, http::request::Parts};
use tracing::error;
use type_core::UserId;

use crate::web_error::WebError;

pub struct AuthUser(pub UserId);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(user_id) = Extension::<UserId>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                error!("缺失user_id extension");
                WebError::InternalServerError
            })?;

        Ok(Self(user_id))
    }
}
