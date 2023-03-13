use async_trait::async_trait;
use log;

use clap;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::docker_compose::{compose_build, Service};
use crate::errors::Result;

#[derive(clap::Args, Default)]
pub struct Build {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes services ports.
    #[clap(long, value_enum, default_values = &[], default_missing_values = &["app", "postgres", "mongo"])]
    pub expose: Vec<Service>,

    #[clap(long, alias = "prod", action)]
    pub production: bool,
}

#[async_trait]
impl CliCommand for Build {
    async fn run(&self) -> Result<()> {
        let conf = load_config_file(&self.config)?;
        // TODO: check the components API version

        conf.generate_files(self.expose.clone(), !self.production).await?;

        log::info!("Build the Docker image");
        compose_build().await?;
        log::info!("Image built");
        Ok(())
    }
}
