#[macro_export]
macro_rules! db_transaction {
    ($db:expr, |$txn:ident| $body:block) => {
        async {
            let transaction = $crate::sea_orm::TransactionTrait::begin($db)
                .await
                .map_err(|error| {
                    $crate::common_core::contextual::ContextualError::error(
                        ::common_core::contextual::Reason::DbError,
                        "开启数据库事务失败",
                        error,
                    )
                })?;
            let result: std::result::Result<_, _> = async {
                let $txn = &transaction;
                $body
            }
            .await;

            match result {
                Ok(value) => {
                    transaction.commit().await.map_err(|error| {
                        $crate::common_core::contextual::ContextualError::error(
                            $crate::common_core::contextual::Reason::DbError,
                            "提交数据库事务失败",
                            error,
                        )
                    })?;
                    Ok(value)
                }
                Err(error) => {
                    transaction.rollback().await.map_err(|error| {
                        $crate::common_core::contextual::ContextualError::error(
                            $crate::common_core::contextual::Reason::DbError,
                            "回滚数据库事务失败",
                            error,
                        )
                    })?;
                    Err(error)
                }
            }
        }
    };
}
