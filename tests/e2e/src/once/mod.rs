use server::state::AppState;

use crate::{ApiClient, error::E2eError};

mod auth;
mod user;

pub async fn run(state: &AppState, client: &ApiClient) -> Result<(), E2eError> {
    auth::run(state, client).await?;
    user::run(state, client).await?;
    Ok(())
}
