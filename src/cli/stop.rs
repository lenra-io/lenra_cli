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
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        stop_loader(context).await
    }
}

pub async fn stop_loader(context: &mut CommandContext) -> Result<()> {
    loader(
        "Stop app environment...",
        "App environment stopped",
        "Failed stopping app environment",
        !context.verbose,
        || async { lenra::stop_env(context).await },
    )
    .await
}
