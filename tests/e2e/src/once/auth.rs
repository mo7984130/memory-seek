use common_core::{
    contextual::ext::ContextualOptionExt,
    ext::{println_ext::DebugExt, redis::RedisExt},
};
use server::state::AppState;
use short_uuid::ShortUuid;
use tracing::info;
use type_user::{LoginRequest, RegisterRequest};

use crate::{ApiClient, LOGIN_CRED, error::E2eError};

pub async fn run(state: &AppState, client: &ApiClient) -> Result<(), E2eError> {
    // 基础登录
    info!("登录");
    client
        .login(LoginRequest {
            account: LOGIN_CRED.0.to_owned(),
            password: LOGIN_CRED.1.to_owned(),
        })
        .await?
        .debugln();

    client.get_self_info().await?.debugln();
    client.remove_access_token().await?;
    client.get_self_info().await?.debugln();

    // 注册
    info!("注册");
    let username = ShortUuid::generate().to_string();
    let register_email = format!("register_test_{}@email.com", uuid::Uuid::new_v4().simple());
    let password = "register_test123".to_owned();
    client
        .send_email_code(type_user::SendEmailRequest {
            email: register_email.clone(),
        })
        .await?;
    let email_code: Option<String> = state
        .redis
        .get_as(constant::RedisKeys::auth::email_verify_code(
            &register_email,
        ))
        .await?;
    let email_code = email_code.ok_or_error(
        common_core::contextual::Reason::RedisError,
        "获取邮箱验证码错误",
    )?;

    let info = client
        .register(RegisterRequest {
            username,
            email: register_email.clone(),
            password: password.clone(),
            confirm_password: password.clone(),
            nickname: "register_test".to_owned(),
            inviter_code: client.generate_inviter_code().await?.inviter_code,
            email_verify_code: email_code,
        })
        .await?;
    assert_eq!(info.email, register_email);
    info.debugln();
    client
        .login_only(LoginRequest {
            account: register_email,
            password,
        })
        .await?
        .debugln();

    Ok(())
}
