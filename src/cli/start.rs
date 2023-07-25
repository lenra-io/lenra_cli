use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::{loader, CommandContext};

#[derive(Args, Default, Debug, Clone)]
pub struct Start;

#[async_trait]
impl CliCommand for Start {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        start_loader().await?;
        clear_cache_loader().await?;
        lenra::display_app_access_url();
        Ok(())
    }
}

pub async fn start_loader() -> Result<()> {
    loader(
        "Start app environment...",
        "App environment started",
        "Failed starting app",
        || async { lenra::start_env().await },
    )
    .await
}

pub async fn clear_cache_loader() -> Result<()> {
    loader(
        "Clearing cache...",
        "Cache cleared",
        "Failed clearing cache",
        || async { lenra::clear_cache().await },
    )
    .await
}
