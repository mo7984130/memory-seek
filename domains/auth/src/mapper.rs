use common_core::contextual::{ContextualError, Result};
use common_core::ext::result::ToOkExt;
use common_core::time::{self, DateTime};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{ActiveModelTrait, DbErr, RuntimeErr};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect, sea_query::Expr,
};
use type_core::UserId;
use type_user::{ActiveModel, Column, Entity, UserRecord};

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
                Err(ContextualError::from(e))
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
        .await?;
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
        .await?;
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
            .await?
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
            .await?,
        )
        .to_ok()
    }
}

#[derive(Debug, FromQueryResult)]
pub struct UserPasswordId {
    pub id: UserId,
    pub password: String,
}
// 查询
impl AuthMapper {
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
            .await?
            .to_ok()
    }

    pub async fn exist_by_email(db: &impl ConnectionTrait, email: &str) -> Result<bool> {
        let ret = Entity::find()
            .filter(Column::Email.eq(email))
            .select_only()
            .column(Column::Id)
            .into_tuple::<UserId>()
            .one(db)
            .await?;
        Ok(ret.is_some())
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
            .await?
            .unwrap_or((None, None));

        match (refresh_token, refresh_token_expire_at) {
            (Some(token), Some(expire_at)) if expire_at > time::now() => Ok(Some(token)),
            _ => Ok(None),
        }
    }
}

// 删除
impl AuthMapper {}
