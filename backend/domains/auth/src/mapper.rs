use crate::error::Result;
use common_core::contextual::ContextualError;
use common_core::contextual::ext::IntoContextualExt;
use common_core::ext::result::ToOkExt;
use common_core::time::{self, DateTime};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{ActiveModelTrait, DbErr, RuntimeErr};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect, sea_query::Expr,
};
use type_auth::{ActiveModel, Column, Entity, UserRecord};
use type_core::UserId;

use crate::error::AuthError;

pub struct AuthMapper;

pub struct AuthInsertParam {
    pub username: String,
    pub email: String,
    pub password: String,
    pub nickname: String,
    pub inviter: UserId,
}
pub enum AuthInsertResult {
    Suc(UserRecord),
    UsernameExisted,
    EmailExisted,
    Other(ContextualError),
}
// 创建
impl AuthMapper {
    pub async fn insert(
        db: &impl ConnectionTrait,
        param: AuthInsertParam,
    ) -> Result<AuthInsertResult> {
        let result = ActiveModel {
            id: NotSet,
            username: Set(param.username),
            email: Set(param.email),
            password: Set(param.password),
            nickname: Set(param.nickname),
            inviter: Set(param.inviter),
            created_at: Set(time::now()),
            updated_at: Set(time::now()),
            avatar_file_id: NotSet,
            refresh_token: NotSet,
            refresh_token_expire_at: NotSet,
        }
        .insert(db)
        .await;
        match result {
            Ok(model) => AuthInsertResult::Suc(UserRecord::from(model)).to_ok(),
            Err(e) => {
                if let DbErr::Query(RuntimeErr::SqlxError(sqlx_err)) = &e
                    && let Some(db_err) = sqlx_err.as_database_error()
                    && db_err.code().as_deref() == Some("23505")
                {
                    let result = match db_err.constraint() {
                        Some("auth_user_username_key" | "user_username_index") => {
                            AuthInsertResult::UsernameExisted
                        }
                        Some("auth_user_email_key" | "user_email_index") => {
                            AuthInsertResult::EmailExisted
                        }
                        _ => AuthInsertResult::Other(ContextualError::from(e)),
                    };
                    return Ok(result);
                };
                Err(ContextualError::from(e))?
            }
        }
    }

    pub async fn create_admin(
        db: &impl ConnectionTrait,
        username: String,
        email: String,
        nickname: String,
        password: String,
    ) -> Result<()> {
        ActiveModel {
            id: Set(UserId(1)),
            username: Set(username),
            email: Set(email),
            password: Set(password),
            nickname: Set(nickname),
            inviter: Set(UserId(0)),
            created_at: Set(time::now()),
            updated_at: Set(time::now()),
            avatar_file_id: NotSet,
            refresh_token: NotSet,
            refresh_token_expire_at: NotSet,
        }
        .insert(db)
        .await
        .into_contextual()?;
        Self::sync_auth_user_id_sequence(db).await?;

        Ok(())
    }

    async fn sync_auth_user_id_sequence(db: &impl ConnectionTrait) -> Result<()> {
        db.execute_unprepared(
            "SELECT setval(
                 pg_get_serial_sequence('auth_user', 'id'),
                 GREATEST((SELECT COALESCE(MAX(id), 0) FROM auth_user), 1)
             )",
        )
        .await
        .into_contextual()?;
        Ok(())
    }
}

// 修改
impl AuthMapper {
    pub async fn update_password(
        db: &impl ConnectionTrait,
        user_id: UserId,
        password: &str,
    ) -> Result<u64> {
        Entity::update_many()
            .filter(Column::Id.eq(user_id))
            .col_expr(Column::Password, Expr::value(password))
            .exec(db)
            .await
            .into_contextual()?
            .rows_affected
            .to_ok()
    }

    pub async fn update_refresh_token(
        db: &impl ConnectionTrait,
        user_id: UserId,
        refresh_token: String,
        refresh_token_expire_at: DateTime,
    ) -> Result<UserRecord> {
        UserRecord::from(
            ActiveModel {
                id: Set(user_id),
                refresh_token: Set(Some(refresh_token)),
                refresh_token_expire_at: Set(Some(refresh_token_expire_at)),
                ..Default::default()
            }
            .update(db)
            .await
            .into_contextual()?,
        )
        .to_ok()
    }

    pub async fn update_nickname(
        db: &impl ConnectionTrait,
        user_id: UserId,
        nickname: String,
    ) -> Result<u64> {
        Entity::update_many()
            .filter(Column::Id.eq(user_id))
            .col_expr(Column::Nickname, Expr::value(nickname))
            .exec(db)
            .await
            .into_contextual()?
            .rows_affected
            .to_ok()
    }

    pub async fn update_avatar(
        db: &impl ConnectionTrait,
        user_id: UserId,
        new_key: &str,
    ) -> Result<Option<String>> {
        let old_key: Option<String> = Entity::find_by_id(user_id)
            .select_only()
            .column(Column::AvatarFileId)
            .lock_exclusive()
            .into_tuple::<Option<String>>()
            .one(db)
            .await
            .into_contextual()?
            .ok_or(AuthError::UserNotFound)?;

        ActiveModel {
            id: Set(user_id),
            avatar_file_id: Set(Some(new_key.to_string())),
            ..Default::default()
        }
        .update(db)
        .await
        .into_contextual()?;

        Ok(old_key)
    }
}

#[derive(Debug, FromQueryResult)]
pub struct UserPasswordId {
    pub id: UserId,
    pub password: String,
}
// 查询
impl AuthMapper {
    pub async fn query_by_id(
        db: &impl ConnectionTrait,
        user_id: UserId,
    ) -> Result<Option<UserRecord>> {
        Entity::find()
            .filter(Column::Id.eq(user_id))
            .one(db)
            .await
            .into_contextual()?
            .map(UserRecord::from)
            .to_ok()
    }

    /// 按账号查询用户
    pub async fn query_by_account(
        db: &impl ConnectionTrait,
        account: &str,
    ) -> Result<Option<UserPasswordId>> {
        Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Password)
            .filter(
                Condition::any()
                    .add(Column::Username.eq(account))
                    .add(Column::Email.eq(account)),
            )
            .into_model::<UserPasswordId>()
            .one(db)
            .await
            .into_contextual()?
            .to_ok()
    }

    pub async fn exist_by_email(db: &impl ConnectionTrait, email: &str) -> Result<bool> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .select_only()
            .column(Column::Id)
            .into_tuple::<UserId>()
            .one(db)
            .await
            .into_contextual()?
            .is_some()
            .to_ok()
    }

    pub async fn query_refresh_token(
        db: &impl ConnectionTrait,
        user_id: UserId,
    ) -> Result<Option<String>> {
        let (refresh_token, refresh_token_expire_at) = Entity::find()
            .filter(Column::Id.eq(user_id))
            .select_only()
            .column(Column::RefreshToken)
            .column(Column::RefreshTokenExpireAt)
            .into_tuple::<(Option<String>, Option<DateTime>)>()
            .one(db)
            .await
            .into_contextual()?
            .unwrap_or((None, None));

        match (refresh_token, refresh_token_expire_at) {
            (Some(token), Some(expire_at)) if expire_at > time::now() => Ok(Some(token)),
            _ => Ok(None),
        }
    }
}

// 删除
impl AuthMapper {}
