use async_trait::async_trait;

use clap;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::{loader, CommandContext};

#[derive(clap::Args, Default, Debug, Clone)]
pub struct Build {
    /// Remove debug access to the app.
    #[clap(long, alias = "prod", action)]
    pub production: bool,
}

#[async_trait]
impl CliCommand for Build {
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        generate_app_env_loader(context, self.production).await?;
        build_loader(context).await
    }
}

pub async fn generate_app_env_loader(context: &mut CommandContext, production: bool) -> Result<()> {
    loader(
        "Generate app env...",
        "App env generated",
        "Failed generating app env",
        !context.verbose,
        || async { lenra::generate_app_env(context, production).await },
    )
    .await
}

pub async fn build_loader(context: &mut CommandContext) -> Result<()> {
    loader(
        "Build app...",
        "App built",
        "Failed building app",
        !context.verbose,
        || async { lenra::build_app(context).await },
    )
    .await
}
