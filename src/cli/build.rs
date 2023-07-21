use async_trait::async_trait;

use clap;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::CommandContext;

#[derive(clap::Args, Default, Debug, Clone)]
pub struct Build {
    /// Remove debug access to the app.
    #[clap(long, alias = "prod", action)]
    pub production: bool,
}

#[async_trait]
impl CliCommand for Build {
    async fn run(&self, context: CommandContext) -> Result<()> {
        lenra::generate_app_env(&context.config, &context.expose, self.production).await?;
        lenra::build_app().await
    }
}
