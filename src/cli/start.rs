use std::path::PathBuf;

use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH};
use crate::devtool::stop_app_env;
use crate::docker_compose::{self, compose_up};
use crate::errors::Result;

#[derive(Args, Default)]
pub struct Start {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    #[clap(long, action)]
    pub expose: bool,
}

#[async_trait]
impl CliCommand for Start {
    async fn run(&self) -> Result<()> {
        log::info!("Starting the app");
        let conf = load_config_file(&self.config)?;

        let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
        if !dockercompose_path.exists() {
            // TODO: check the components API version

            conf.generate_files(self.expose).await?;
        }

        // Start the containers
        compose_up().await?;
        // Stop the devtool app env to reset cache
        let result = stop_app_env().await;
        if let Err(error) = result {
            log::info!("{:?}", error);
        }
        // Open the app
        println!(
            "\nApplication available at http://localhost:{}\n",
            docker_compose::DEVTOOL_PORT
        );
        Ok(())
    }
}
