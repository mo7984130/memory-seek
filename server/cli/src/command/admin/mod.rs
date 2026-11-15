use crate::{command::admin::auth::AdminCreateArgs, error::CliError};
use clap::Subcommand;
use server::state::AppState;

mod auth;

/// 管理员子命令
#[derive(Subcommand)]
pub enum AdminCommand {
    /// 创建默认管理员(auth_user.id = 1), 只需在首次部署后执行一次
    Create(AdminCreateArgs),
}

impl AdminCommand {
    pub async fn exec(self, state: &AppState) -> Result<(), CliError> {
        match self {
            Self::Create(args) => args.exec(state).await,
        }
    }
}
