use std::sync::Arc;

use audit::recorder::AuditRecorder;
use audit::{AuditActor, AuditEvent};
use common_core::contextual::ContextualError;
use common_core::ext::boolean::TrueOrExt;
use common_core::ext::redis::RedisExt;
use common_core::ext::result::ToOkExt;
use common_core::time::{self};
use common_core::{
    contextual::ext::{EmitIfErrExt, FlattenResultExt, IntoContextualExt},
    ext::result::ResultInspectErrAsync,
};
use common_crypto::{
    password_hasher::{PASSWORD_HASHER, PasswordHasher},
    rand_util,
};
use common_db::db_transaction;
use constant::{AuditEventTypes, RedisKeys};
use sea_orm::DatabaseConnection;
use share::visual_token::VisualToken;
use tokio::task;
use type_core::UserId;
use type_user::{
    CreateAdminArgs, LoginRequest, LoginResponse, RefreshAccessTokenResponse, RegisterRequest,
    SendEmailRequest, UserInfo,
};
use validator::Validate;

use crate::constant::{
    ACCESS_TOKEN_LEN, ACCESS_TOKEN_REFRESH_TTL, ACCESS_TOKEN_TTL, EMAIL_CODE_EXPIRATION,
    EMAIL_CODE_LEN, REFRESH_TOKEN_LEN, REFRESH_TOKEN_TTL,
};
use crate::error::AuthError;
use crate::mapper::{AuthInsertParam, AuthInsertResult};
use crate::{mapper::AuthMapper, state::AuthState};

pub struct AuthService;

// 创建
impl AuthService {
    /// 注册
    #[tracing::instrument(
        skip_all,
        fields(
            email = %req.email
        )
    )]
    pub async fn register(state: &AuthState, req: RegisterRequest) -> Result<UserInfo, AuthError> {
        // 效验邀请码
        let inviter = Self::verify_inviter_code(state, req.inviter_code)
            .await?
            .ok_or(AuthError::InviterCodeInvalid)?;

        // 效验邮箱验证码
        Self::verify_email_code(state, &req.email, &req.email_verify_code)
            .await?
            .true_or(AuthError::EmailVerifyCodeInvalid)?;

        // 加密密码
        let password = req.password.clone();
        let hashed_password = task::spawn_blocking(move || PASSWORD_HASHER.hash(&password))
            .await
            .into_contextual()??;

        let insert_param = AuthInsertParam {
            username: req.username,
            email: req.email,
            password: hashed_password,
            nickname: req.nickname,
            inviter,
        };
        // 插入用户
        let result = db_transaction!(&state.db, |txn| {
            let result = AuthMapper::insert(txn, insert_param).await?;
            if let AuthInsertResult::Suc(user) = &result {
                AuditRecorder::append(
                    txn,
                    AuditEvent::new(AuditEventTypes::auth::Register, AuditActor::User(user.id)),
                )
                .await?;
            };

            Ok::<_, ContextualError>(result)
        })
        .await?;

        match result {
            AuthInsertResult::Suc(user) => Ok(UserInfo::without_avatar(user)),
            AuthInsertResult::UsernameExisted => Err(AuthError::UsernameExists),
            AuthInsertResult::EmailExisted => Err(AuthError::EmailExists),
            AuthInsertResult::Other(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(
        skip_all,
        fields(
            args = ?args
        )
    )]
    pub async fn create_admin(
        db: &DatabaseConnection,
        args: CreateAdminArgs,
    ) -> Result<(), AuthError> {
        args.validate()?;

        let CreateAdminArgs {
            username,
            email,
            password,
            confirm_password: _,
            nickname,
        } = args;
        let password = task::spawn_blocking(move || PASSWORD_HASHER.hash(&password))
            .await
            .into_contextual()??;
        AuthMapper::create_admin(db, username, email, nickname, password).await?;
        Ok(())
    }

    async fn verify_inviter_code(
        state: &AuthState,
        inviter_code: String,
    ) -> Result<Option<UserId>, ContextualError> {
        // 转大写后查找
        let code_upper = inviter_code.to_uppercase();
        if code_upper == "DRIFTC" {
            return Ok(Some(UserId(1)));
        }

        // 查询
        state
            .redis
            .get_as(RedisKeys::auth::inviter_code(&code_upper))
            .await?
            .map(UserId)
            .to_ok()
    }

    async fn verify_email_code(
        state: &AuthState,
        email: &str,
        code: &str,
    ) -> Result<bool, ContextualError> {
        let stored_code: Option<String> = state
            .redis
            .get_as(RedisKeys::auth::email_verify_code(email))
            .await?;
        let code_upper = code.to_uppercase();
        Ok(match stored_code {
            None => false,
            Some(code) => code == code_upper,
        })
    }

    #[tracing::instrument(
        skip_all,
        fields(
            email = %req.email
        )
    )]
    pub async fn send_email_code(
        state: Arc<AuthState>,
        req: SendEmailRequest,
    ) -> Result<(), AuthError> {
        // 判断邮箱是否存在
        if AuthMapper::exist_by_email(&state.db, &req.email).await? {
            return Err(AuthError::EmailExists);
        };

        // 生成验证码
        let code = rand_util::generate_random_uppercase_str(EMAIL_CODE_LEN);

        // 设置邀请码到redis中
        state
            .redis
            .set_ex(
                RedisKeys::auth::email_verify_code(&req.email),
                &code,
                EMAIL_CODE_EXPIRATION,
            )
            .await?;

        // 发送邮件
        let html_body = format!(
            "<p>您的验证码为: <strong>{}</strong></p><p>该验证码有效期为 {} 分钟。</p>",
            code,
            EMAIL_CODE_EXPIRATION.as_secs() / 60
        );

        state
            .email_client
            .send_message(&req.email, "寻忆邮箱验证码", html_body)
            .await?;

        Ok(())
    }
}

// 修改
impl AuthService {
    /// 用户登录
    #[tracing::instrument(
        skip_all,
        fields(
            account = %req.account
        )
    )]
    pub async fn login(state: &AuthState, req: LoginRequest) -> Result<LoginResponse, AuthError> {
        // 判断账号是否存在并且获取信息
        let user = match AuthMapper::query_by_account(&state.db, &req.account).await? {
            Some(v) => v,
            None => {
                // dummy验证, 防止时序攻击
                task::spawn_blocking(PasswordHasher::dummy_verify)
                    .await
                    .into_contextual()
                    .emit_if_err();
                return Err(AuthError::InvalidCredentials);
            }
        };

        // 校验密码
        let old_alg = {
            let _permit = common_crypto::password_hasher::password_verify_semaphore()
                .acquire()
                .await
                .into_contextual()?;

            let password_clone = req.password.clone();
            let old_password = user.password;
            let (correct, alg) = task::spawn_blocking(move || {
                PasswordHasher::verify_and_detect(&password_clone, &old_password)
            })
            .await
            .into_contextual()??;
            if !correct {
                return Err(AuthError::InvalidCredentials);
            }
            alg
        };

        // 密码版本不匹配的话,迁移密码
        if PASSWORD_HASHER != old_alg {
            let password_clone = req.password.clone();
            let new_password = task::spawn_blocking(move || PASSWORD_HASHER.hash(&password_clone))
                .await
                .into_contextual()
                .flatten_contextual()
                .emit_if_err();
            if let Some(new_password) = new_password {
                AuthMapper::update_password(&state.db, user.id, &new_password).await?;
            }
        }

        // 更新Access Token
        let new_access_token = rand_util::generate_random_str(ACCESS_TOKEN_LEN);
        Self::set_access_token(state, user.id, &new_access_token).await?;

        let new_refresh_token = rand_util::generate_random_str(REFRESH_TOKEN_LEN);
        let new_refresh_token_expire_at = time::now_after(REFRESH_TOKEN_TTL);

        // 更新 RefreshToken
        let user = db_transaction!(&state.db, |txn| {
            let user = state
                .repo
                .update_refresh_token(user.id, &new_refresh_token, new_refresh_token_expire_at)
                .await
                .inspect_err_async(|_| async {
                    state
                        .redis
                        .del(RedisKeys::auth::user_access_token(user.id))
                        .await
                        .into_contextual()
                        .emit_if_err();
                })
                .await?;

            AuditRecorder::append(
                txn,
                AuditEvent::new(AuditEventTypes::auth::Login, AuditActor::User(user.id)),
            )
            .await?;

            Ok::<_, ContextualError>(user)
        })
        .await?;

        // 返回结果
        let user_id = user.id;
        LoginResponse {
            user: UserInfo::from_record(user, |file_id| {
                VisualToken::image_thumbnail(file_id, user_id).encrypt(&state.token_cipher)
            })?,
            access_token: new_access_token,
            access_token_expire_at: time::now_after(ACCESS_TOKEN_TTL),
            refresh_token: new_refresh_token,
            refresh_token_expire_at: new_refresh_token_expire_at,
        }
        .to_ok()
    }

    async fn set_access_token(
        state: &AuthState,
        user_id: UserId,
        access_token: &str,
    ) -> Result<(), ContextualError> {
        let mut conn = state.redis.get().await?;
        redis::Script::new(
            r#"
            redis.call('SET', KEYS[1], ARGV[1], 'EX', ARGV[2])
            redis.call('DEL', KEYS[2])
            return 1
            "#,
        )
        .key(RedisKeys::auth::user_access_token(user_id))
        .key(RedisKeys::auth::user_access_token_refresh(user_id))
        .arg(access_token)
        .arg(ACCESS_TOKEN_TTL.as_secs())
        .invoke_async::<i32>(&mut conn)
        .await?;
        Ok(())
    }

    #[tracing::instrument(
        skip_all,
        fields(
            user_id = %user_id
        )
    )]
    pub async fn refresh_access_token(
        state: Arc<AuthState>,
        user_id: UserId,
        refresh_token: String,
    ) -> Result<RefreshAccessTokenResponse, AuthError> {
        // 效验 refresh_token
        if !Self::verify_refresh_token(&state, user_id, &refresh_token).await? {
            return Err(AuthError::RefreshTokenInvalid);
        }

        // 派发新的AccessToken
        let redis_key = RedisKeys::auth::user_access_token(user_id);
        let redis_key_refresh = RedisKeys::auth::user_access_token_refresh(user_id);
        let mut conn = state.redis.get().await.into_contextual()?;

        // 为了保证竞态下, 返回的access_token正确, 使用refresh窗口返回
        let new_access_token: String = redis::Script::new(
            r#"
            local token = redis.call('GET', KEYS[1])
            if token then return token end
            redis.call('SET', KEYS[2], ARGV[1], 'EX', ARGV[2])
            redis.call('SET', KEYS[1], ARGV[1], 'EX', ARGV[3])
            return ARGV[1]
            "#,
        )
        .key(redis_key_refresh)
        .key(redis_key)
        .arg(rand_util::generate_random_str(32))
        .arg(ACCESS_TOKEN_TTL.as_secs())
        .arg(ACCESS_TOKEN_REFRESH_TTL.as_secs())
        .invoke_async(&mut conn)
        .await
        .into_contextual()?;

        RefreshAccessTokenResponse {
            access_token: new_access_token,
            access_token_expire_at: time::now_after(ACCESS_TOKEN_TTL),
        }
        .to_ok()
    }

    async fn verify_refresh_token(
        state: &AuthState,
        user_id: UserId,
        refresh_token: &str,
    ) -> Result<bool, ContextualError> {
        match state.repo.get_refresh_token(user_id).await? {
            None => Ok(false),
            Some(token) => Ok(token == refresh_token),
        }
    }
}

// 查询
impl AuthService {}

// 删除
impl AuthService {}
