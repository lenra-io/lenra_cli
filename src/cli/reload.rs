use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::CommandContext;

#[derive(Args, Default, Debug, Clone)]
pub struct Reload;

#[async_trait]
impl CliCommand for Reload {
    async fn run(&self, context: CommandContext) -> Result<()> {
        lenra::generate_app_env(&context.config, &context.expose, false).await?;
        lenra::build_app().await?;
        lenra::start_env().await?;
        lenra::clear_cache().await
    }
}
