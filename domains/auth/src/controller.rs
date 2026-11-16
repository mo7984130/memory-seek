use std::sync::Arc;

use crate::error::AuthError;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use common_core::{ext::result::ToOkExt, types::R};
use common_web::{
    controller_router::ControllerRouter,
    extractors::{AuthUser, ValidatedJson},
    web_domain::WebDomain,
    web_error::WebError,
};
use tracing::warn;
use type_auth::{
    InviterCodeResponse, LoginRequest, LoginResponse, RefreshAccessTokenResponse, RegisterRequest,
    SendEmailRequest, UpdateNicknameRequest, UserInfo,
};
use type_core::{UserId, viusal::VisualTokenStr};

use crate::{service::AuthService, state::AuthState};

pub struct AuthController;

impl ControllerRouter for AuthController {
    type State = AuthState;

    fn protected_routes() -> Router<std::sync::Arc<Self::State>> {
        Router::new()
            .route("/inviter-code", post(Self::generate_inviter_code))
            .route("/me", get(Self::get_user_self_info))
            .route("/nickname", post(Self::update_nickname))
            .route("/avatar", post(Self::upload_avatar))
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

            AuthError::InviterCodeGenerateExhausted => {
                WebError::too_many_requests(value.to_string())
            }

            AuthError::Validation(msg) => WebError::bad_request(msg),
            AuthError::ImageInvalid(err) => WebError::bad_request(err.to_string()),

            AuthError::InviterCodeInvalid
            | AuthError::EmailVerifyCodeInvalid
            | AuthError::InvalidCredentials => WebError::bad_request(value.to_string()),

            AuthError::UsernameExists | AuthError::EmailExists => {
                WebError::conflict(value.to_string())
            }

            AuthError::RefreshTokenInvalid => WebError::unauthorized(value.to_string()),

            AuthError::UserNotFound => WebError::not_found("用户不存在"),
        }
    }
}

// 创建
impl AuthController {
    async fn generate_inviter_code(
        State(state): State<Arc<AuthState>>,
        AuthUser(user_id): AuthUser,
    ) -> Result<R<InviterCodeResponse>, WebError> {
        R::ok(AuthService::generate_inviter_code(state, user_id).await?).to_ok()
    }

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

    pub async fn update_nickname(
        State(state): State<Arc<AuthState>>,
        AuthUser(user_id): AuthUser,
        ValidatedJson(req): ValidatedJson<UpdateNicknameRequest>,
    ) -> Result<R<()>, WebError> {
        R::ok(AuthService::update_nickname(state, user_id, req).await?).to_ok()
    }

    async fn upload_avatar(
        State(state): State<Arc<AuthState>>,
        AuthUser(user_id): AuthUser,
        body: Body,
    ) -> Result<R<VisualTokenStr>, WebError> {
        // 头像为小文件, 限制在图片校验上限内整体读入(复用 FileValidator 上限常量, 避免两处漂移)
        let file_data = axum::body::to_bytes(body, media_validator::IMAGE_MAX_SIZE as usize)
            .await
            .map_err(|e| {
                warn!("读取头像失败: {:?}", e);
                WebError::bad_request("读取头像文件失败")
            })?;
        if file_data.is_empty() {
            return Err(WebError::bad_request("头像文件为空"));
        }

        Ok(R::ok(
            AuthService::upload_avatar(state, user_id, file_data).await?,
        ))
    }
}

// 查询
impl AuthController {
    pub async fn get_user_self_info(
        State(state): State<Arc<AuthState>>,
        AuthUser(user_id): AuthUser,
    ) -> Result<R<UserInfo>, WebError> {
        R::ok(AuthService::get_user_self_info(state, user_id).await?).to_ok()
    }
}

// 删除
impl AuthController {}
