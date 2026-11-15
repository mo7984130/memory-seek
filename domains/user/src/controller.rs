use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::routing::{get, post};
use common_core::ext::result::ToOkExt;
use common_core::types::R;
use common_web::extractors::{AuthUser, ValidatedJson};
use common_web::web_error::{Result, WebError};
use common_web::{controller_router::ControllerRouter, web_domain::WebDomain};
use tracing::warn;
use type_core::viusal::VisualTokenStr;
use type_user::{InviterCodeResponse, UpdateNicknameRequest, UserInfo};

use crate::error::UserError;
use crate::{service::UserService, state::UserState};

pub struct UserController;

impl ControllerRouter for UserController {
    type State = UserState;

    fn protected_routes() -> axum::Router<std::sync::Arc<Self::State>> {
        Router::new()
            .route("/inviter-code", post(Self::generate_inviter_code))
            .route("/me", get(Self::get_user_self_info))
            .route("/nickname", post(Self::update_nickname))
            .route("/avatar", post(Self::upload_avatar))
    }
}

pub struct UserWebDomain;
impl WebDomain for UserWebDomain {
    type State = UserState;
    type Controller = UserController;
}

impl From<UserError> for common_web::web_error::WebError {
    fn from(value: UserError) -> Self {
        use common_web::web_error::WebError;

        match value {
            UserError::Internal(error) => error.into(),
            error @ UserError::InviterCodeGenerateExhausted => {
                WebError::too_many_requests(error.to_string())
            }
            error @ UserError::UserNotFound => WebError::not_found(error.to_string()),
            UserError::ImageInvalid(e) => WebError::bad_request(e.to_string()),
        }
    }
}

// 创建
impl UserController {
    async fn generate_inviter_code(
        State(state): State<Arc<UserState>>,
        AuthUser(user_id): AuthUser,
    ) -> Result<R<InviterCodeResponse>> {
        R::ok(UserService::generate_inviter_code(state, user_id).await?).to_ok()
    }
}

// 修改
impl UserController {
    pub async fn update_nickname(
        State(state): State<Arc<UserState>>,
        AuthUser(user_id): AuthUser,
        ValidatedJson(req): ValidatedJson<UpdateNicknameRequest>,
    ) -> Result<R<()>> {
        R::ok(UserService::update_nickname(state, user_id, req).await?).to_ok()
    }

    async fn upload_avatar(
        State(state): State<Arc<UserState>>,
        AuthUser(user_id): AuthUser,
        body: Body,
    ) -> Result<R<VisualTokenStr>> {
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
            UserService::upload_avatar(state, user_id, file_data).await?,
        ))
    }
}

// 查询
impl UserController {
    pub async fn get_user_self_info(
        State(state): State<Arc<UserState>>,
        AuthUser(user_id): AuthUser,
    ) -> Result<R<UserInfo>> {
        R::ok(UserService::get_user_self_info(state, user_id).await?).to_ok()
    }
}

// 删除
impl UserController {}
