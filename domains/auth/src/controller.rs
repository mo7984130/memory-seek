use std::sync::Arc;

use crate::error::AuthError;
use axum::{Router, extract::State, http::HeaderMap, routing::post};
use common_core::{ext::result::ToOkExt, types::R};
use common_web::{
    controller_router::ControllerRouter, extractors::ValidatedJson, web_domain::WebDomain,
    web_error::WebError,
};
use type_core::UserId;
use type_user::{
    LoginRequest, LoginResponse, RefreshAccessTokenResponse, RegisterRequest, SendEmailRequest,
    UserInfo,
};

use crate::{service::AuthService, state::AuthState};

pub struct AuthController;

impl ControllerRouter for AuthController {
    type State = AuthState;

    fn protected_routes() -> Router<std::sync::Arc<Self::State>> {
        Router::new()
    }

    fn public_routes() -> Router<std::sync::Arc<Self::State>> {
        Router::new()
            .route("/email-code", post(Self::send_email_code))
            .route("/register", post(Self::register))
            .route("/login", post(Self::login))
            .route("/token", post(Self::refresh_access_token))
    }
}
pub struct AuthDomain;
impl WebDomain for AuthDomain {
    type State = AuthState;

    type Controller = AuthController;
}

impl From<AuthError> for common_web::web_error::WebError {
    fn from(value: AuthError) -> Self {
        use common_web::web_error::WebError;

        match value {
            AuthError::Internal(error) => error.into(),
            AuthError::Validation(message) => WebError::bad_request(message),
            error @ (AuthError::InviterCodeInvalid
            | AuthError::EmailVerifyCodeInvalid
            | AuthError::InvalidCredentials) => WebError::bad_request(error.to_string()),
            error @ (AuthError::UsernameExists | AuthError::EmailExists) => {
                WebError::conflict(error.to_string())
            }
            error @ AuthError::RefreshTokenInvalid => WebError::unauthorized(error.to_string()),
        }
    }
}

// 创建
impl AuthController {
    pub async fn register(
        State(state): State<Arc<AuthState>>,
        ValidatedJson(req): ValidatedJson<RegisterRequest>,
    ) -> Result<R<UserInfo>, WebError> {
        R::ok(AuthService::register(&state, req).await?).to_ok()
    }

    pub async fn refresh_access_token(
        State(state): State<Arc<AuthState>>,
        headers: HeaderMap,
    ) -> Result<R<RefreshAccessTokenResponse>, WebError> {
        let user_id = headers
            .get("x-user-id")
            .ok_or(WebError::bad_request("缺失x-user-id头"))?
            .to_str()
            .map_err(|_| WebError::bad_request("x-user-id头格式错误"))?
            .parse::<UserId>()
            .map_err(|_| WebError::bad_request("x-user-id头格式错误"))?;

        let refresh_token = headers
            .get("x-refresh-token")
            .ok_or(WebError::bad_request("x-refresh-token头格式错误"))?
            .to_str()
            .map_err(|_| WebError::bad_request("x-refresh-token头格式错误"))?
            .to_string();

        R::ok(AuthService::refresh_access_token(state, user_id, refresh_token).await?).to_ok()
    }

    pub async fn send_email_code(
        State(state): State<Arc<AuthState>>,
        ValidatedJson(req): ValidatedJson<SendEmailRequest>,
    ) -> Result<R<()>, WebError> {
        R::ok(AuthService::send_email_code(state, req).await?).to_ok()
    }
}

// 修改
impl AuthController {
    pub async fn login(
        State(state): State<Arc<AuthState>>,
        ValidatedJson(req): ValidatedJson<LoginRequest>,
    ) -> Result<R<LoginResponse>, WebError> {
        R::ok(AuthService::login(&state, req).await?).to_ok()
    }
}

// 查询
impl AuthController {}

// 删除
impl AuthController {}
