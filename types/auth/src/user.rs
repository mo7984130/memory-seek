#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    // 用户ID
    pub id: UserId,

    /// 用户名
    pub username: String,

    /// 昵称
    pub nickname: String,

    /// 邮箱
    pub email: String,

    /// 头像令牌
    pub avatar_token: Option<VisualTokenStr>,

    /// 创建时间
    pub created_at: DateTime,
}

#[cfg(feature = "orm")]
mod entity {
    use common_core::{contextual::ContextualError, ext::result::ToOkExt, time::DateTime};
    use sea_orm::entity::prelude::*;
    use type_core::{UserId, viusal::VisualTokenStr};

    use super::*;

    common_db::scheme_register!();

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "auth_user")]
    pub struct Model {
        /// 主键ID
        #[sea_orm(primary_key)]
        pub id: UserId,

        /// 用户名
        #[sea_orm(column_type = "String(StringLen::N(63))", unique)]
        pub username: String,

        /// 邮箱地址
        #[sea_orm(column_type = "String(StringLen::N(255))", unique)]
        pub email: String,

        /// 加密后的密码
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub password: String,

        /// 用户昵称
        #[sea_orm(column_type = "String(StringLen::N(63))")]
        pub nickname: String,

        /// 头像 文件ID
        #[sea_orm(column_type = "String(StringLen::N(1023))", nullable)]
        pub avatar_file_id: Option<String>,

        /// 邀请人ID
        pub inviter: UserId,

        /// 刷新令牌
        #[sea_orm(column_type = "String(StringLen::N(32))", nullable)]
        pub refresh_token: Option<String>,
        /// 刷新令牌过期时间
        pub refresh_token_expire_at: Option<DateTime>,

        /// 更新时间
        #[sea_orm(default_expr = "sea_orm::sea_query::Expr::current_timestamp()")]
        pub updated_at: DateTime,
        /// 创建时间
        #[sea_orm(default_expr = "sea_orm::sea_query::Expr::current_timestamp()")]
        pub created_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Clone, Debug)]
    pub struct UserRecord {
        pub id: UserId,
        pub username: String,
        pub email: String,
        pub password: String,
        pub nickname: String,
        pub avatar_file_id: Option<String>,
        pub inviter: UserId,
        pub refresh_token: Option<String>,
        pub refresh_token_expire_at: Option<DateTime>,
        pub updated_at: DateTime,
        pub created_at: DateTime,
    }

    impl From<Model> for UserRecord {
        fn from(model: Model) -> Self {
            Self {
                id: model.id,
                username: model.username,
                email: model.email,
                password: model.password,
                nickname: model.nickname,
                avatar_file_id: model.avatar_file_id,
                inviter: model.inviter,
                refresh_token: model.refresh_token,
                refresh_token_expire_at: model.refresh_token_expire_at,
                updated_at: model.updated_at,
                created_at: model.created_at,
            }
        }
    }

    impl UserInfo {
        /// 为头像文件 ID 生成加密访问令牌
        pub fn from_record(
            user: UserRecord,
            generate_avatar_token: impl FnOnce(String) -> Result<VisualTokenStr, ContextualError>,
        ) -> Result<Self, ContextualError> {
            UserInfo {
                id: user.id,
                username: user.username,
                nickname: user.nickname,
                email: user.email,
                avatar_token: match user.avatar_file_id {
                    Some(file_id) => Some(generate_avatar_token(file_id)?),
                    None => None,
                },
                created_at: user.created_at,
            }
            .to_ok()
        }

        pub fn without_avatar(user: UserRecord) -> Self {
            Self {
                id: user.id,
                username: user.username,
                nickname: user.nickname,
                email: user.email,
                avatar_token: None,
                created_at: user.created_at,
            }
        }
    }
}
use common_core::time::DateTime;
#[cfg(feature = "orm")]
pub use entity::*;
use serde::{Deserialize, Serialize};
use type_core::{UserId, viusal::VisualTokenStr};
