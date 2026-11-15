use crate::error::{Result, UserError};
use common_core::{contextual::ext::IntoContextualExt, ext::result::ToOkExt};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QuerySelect, sea_query::Expr,
};
use type_core::UserId;
use type_user::{ActiveModel, Column, Entity, UserRecord};

pub struct UserMapper;

// 创建
impl UserMapper {}

// 修改
impl UserMapper {
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
        new_key: String,
    ) -> Result<Option<String>> {
        let old_key: Option<String> = Entity::find_by_id(user_id)
            .select_only()
            .column(Column::AvatarFileId)
            .lock_exclusive()
            .into_tuple::<Option<String>>()
            .one(db)
            .await
            .into_contextual()?
            .ok_or(UserError::UserNotFound)?;

        ActiveModel {
            id: Set(user_id),
            avatar_file_id: Set(Some(new_key)),
            ..Default::default()
        }
        .update(db)
        .await
        .into_contextual()?;

        Ok(old_key)
    }
}

// 查询
impl UserMapper {
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
}

// 删除
impl UserMapper {}
