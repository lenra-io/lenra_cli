use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::CommandContext;

#[derive(Args, Debug, Clone)]
pub struct Stop;

#[async_trait]
impl CliCommand for Stop {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        lenra::stop_env().await
    }
}
