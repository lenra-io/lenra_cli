use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;

use super::{
    build::{build_loader, generate_app_env_loader},
    start::{clear_cache_loader, start_loader},
    CommandContext,
};

#[derive(Args, Default, Debug, Clone)]
pub struct Reload;

#[async_trait]
impl CliCommand for Reload {
    async fn run(&self, context: CommandContext) -> Result<()> {
        generate_app_env_loader(context, false).await?;
        build_loader().await?;
        start_loader().await?;
        clear_cache_loader().await
    }
}
