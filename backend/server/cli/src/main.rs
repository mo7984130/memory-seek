use std::error::Error;

use clap::Parser;

mod command;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let init_bundle = server::init(None).await?;
    command::Command::parse().run(&init_bundle.state).await?;
    Ok(())
}
