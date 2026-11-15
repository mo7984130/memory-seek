use axum::{body::Bytes, extract::FromRequest};
use common_core::contextual::from::validator::format_validation_error_msg;

use crate::web_error::{WebError, ext::ToWebErrorExt};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let mut bytes = Bytes::from_request(req, state)
            .await
            .web_error(|| WebError::bad_request("读取请求体失败"))?;
        if bytes.is_empty() {
            bytes = Bytes::from_static(b"{}");
        }

        let value: T =
            serde_json::from_slice(&bytes).web_error(|| WebError::bad_request("请求体格式错误"))?;

        value.validate().map_err(|err| {
            let msg = format_validation_error_msg(&err);
            WebError::bad_request(msg)
        })?;

        Ok(Self(value))
    }
}
