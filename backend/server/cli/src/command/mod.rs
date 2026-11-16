mod admin;

use clap::Parser;
use server::state::AppState;

use crate::{command::admin::AdminCommand, error::CliError};

#[derive(Parser)]
pub enum Command {
    Admin {
        #[command(subcommand)]
        action: AdminCommand,
    },
}

impl Command {
    pub async fn run(self, state: &AppState) -> Result<(), CliError> {
        match self {
            Self::Admin { action } => action.exec(state).await?,
        }
        Ok(())
    }
}
