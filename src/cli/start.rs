use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::CommandContext;

#[derive(Args, Default, Debug, Clone)]
pub struct Start;

#[async_trait]
impl CliCommand for Start {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        lenra::start_env().await?;
        lenra::clear_cache().await?;
        lenra::display_app_access_url();
        Ok(())
    }
}
