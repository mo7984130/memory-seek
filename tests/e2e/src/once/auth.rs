use common_core::{contextual::ext::ContextualOptionExt, ext::redis::RedisExt};
use server::state::AppState;
use short_uuid::ShortUuid;
use tracing::info;

use crate::{ApiClient, error::E2eError};

pub async fn run(state: &AppState, client: &ApiClient) -> Result<(), E2eError> {
    // 1. 注册一个全新账号
    info!("注册");
    let username = ShortUuid::generate().to_string();
    let email = format!("e2e_{}@example.com", uuid::Uuid::new_v4().simple());
    let password = "register_test123".to_owned();

    client
        .send_email_code(type_user::SendEmailRequest {
            email: email.clone(),
        })
        .await?;

    let email_code: Option<String> = state
        .redis
        .get_as(constant::RedisKeys::auth::email_verify_code(&email))
        .await?;
    let email_code = email_code.ok_or_error(
        common_core::contextual::Reason::RedisError,
        "获取邮箱验证码错误",
    )?;

    let info = client
        .register(type_user::RegisterRequest {
            username,
            email: email.clone(),
            password: password.clone(),
            confirm_password: password.clone(),
            nickname: "register_test".to_owned(),
            inviter_code: "DRIFTC".to_owned(),
            email_verify_code: email_code,
        })
        .await?;
    assert_eq!(info.email, email);
    info!("注册成功: {:?}", info);

    // 2. 用刚注册的账号登录
    info!("登录");
    info!(
        "登录成功: {:?}",
        client
            .login(type_user::LoginRequest {
                account: email,
                password,
            })
            .await?
    );

    info!("测试刷新access_token");
    info!("UserInfo: {:?}", client.get_self_info().await?);
    info!("移除AccessToken");
    client.remove_access_token().await?;
    info!("UserInfo: {:?}", client.get_self_info().await?);

    Ok(())
}
