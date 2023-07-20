use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::devtool::stop_app_env;
use crate::docker_compose::{compose_build, compose_up, Service};
use crate::errors::Result;

#[derive(Args, Default, Debug, Clone)]
pub struct Reload {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes services ports.
    #[clap(long, value_enum, default_values = &[], default_missing_values = &["app", "postgres", "mongo"])]
    pub expose: Vec<Service>,
}

#[async_trait]
impl CliCommand for Reload {
    async fn run(&self) -> Result<()> {
        log::info!("Reload the app");
        let conf = load_config_file(&self.config)?;

        log::debug!("Generates files");
        conf.generate_files(self.expose.clone(), true).await?;

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
