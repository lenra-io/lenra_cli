use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::load_config_file;
use crate::devtool::stop_app_env;
use crate::docker_compose::{compose_build, compose_up};
use crate::errors::Result;

use super::CommandContext;

#[derive(Args, Default, Debug, Clone)]
pub struct Reload;

#[async_trait]
impl CliCommand for Reload {
    async fn run(&self, context: CommandContext) -> Result<()> {
        log::info!("Reload the app");
        let conf = load_config_file(&context.config)?;

        log::debug!("Generates files");
        conf.generate_files(&context.expose, true).await?;

        log::debug!("Docker compose build");
        compose_build().await?;

        log::debug!("Starts the containers");
        compose_up().await?;

        log::debug!("Stop the devtool app env to reset cache");
        println!("Clearing cache");
        let result = stop_app_env().await;
        if let Err(error) = result {
            log::error!("{:?}", error);
        }
        Ok(())
    }
}
