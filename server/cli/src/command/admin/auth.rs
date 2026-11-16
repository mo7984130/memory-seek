use auth::service::AuthService;
use clap::Args;
use common_core::contextual::ext::IntoContextualExt;
use server::state::AppState;
use type_auth::CreateAdminArgs;

use crate::error::CliError;

/// `admin create` 参数
#[derive(Args)]
pub struct AdminCreateArgs {
    /// 登录用户名(4-20 位字母、数字、下划线或短横线)
    #[arg(long)]
    pub username: String,

    /// 登录邮箱
    #[arg(long)]
    pub email: String,

    /// 昵称
    #[arg(long, default_value = "管理员")]
    pub nickname: String,
}
impl AdminCreateArgs {
    pub async fn exec(self, state: &AppState) -> Result<(), CliError> {
        let password = rpassword::prompt_password("请输入密码: ").into_contextual()?;
        let confirm_password = rpassword::prompt_password("请重复密码: ").into_contextual()?;

        let args = CreateAdminArgs {
            username: self.username,
            email: self.email,
            password,
            confirm_password,
            nickname: self.nickname,
        };

        AuthService::create_admin(&state.db, args).await?;
        Ok(())
    }
}
