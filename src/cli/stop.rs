use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::{loader, CommandContext};

#[derive(Args, Debug, Clone)]
pub struct Stop;

#[async_trait]
impl CliCommand for Stop {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        stop_loader().await
    }
}

pub async fn stop_loader() -> Result<()> {
    loader(
        "Stop app environment...",
        "App environment stopped",
        "Failed stopping app environment",
        || async { lenra::stop_env().await },
    )
    .await
}
