use std::error::Error;

use crate::web_error::WebError;

pub trait ToWebErrorExt<T> {
    fn web_error(self, app_error: impl FnOnce() -> WebError) -> Result<T, WebError>;
}
impl<T, E: Error> ToWebErrorExt<T> for Result<T, E> {
    fn web_error(self, web_error: impl FnOnce() -> WebError) -> Result<T, WebError> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(web_error()),
        }
    }
}
