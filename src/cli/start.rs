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
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        start_loader(context).await?;
        clear_cache_loader(context).await?;
        lenra::display_app_access_url();
        Ok(())
    }
}

pub async fn start_loader(context: &mut CommandContext) -> Result<()> {
    loader(
        "Start app environment...",
        "App environment started",
        "Failed starting app",
        !context.verbose,
        || async { lenra::start_env(context).await },
    )
    .await
}

pub async fn clear_cache_loader(context: &mut CommandContext) -> Result<()> {
    loader(
        "Clearing cache...",
        "Cache cleared",
        "Failed clearing cache",
        !context.verbose,
        || async { lenra::clear_cache(context).await },
    )
    .await
}
