use async_trait::async_trait;
use log;

use clap;

use crate::cli::CliCommand;
use crate::config::load_config_file;
use crate::docker_compose::compose_build;
use crate::errors::Result;

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
        let conf = load_config_file(&context.config)?;
        // TODO: check the components API version

        conf.generate_files(&context.expose, !self.production)
            .await?;

        log::info!("Build the Docker image");
        compose_build().await?;
        log::info!("Image built");
        Ok(())
    }
}
