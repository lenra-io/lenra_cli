use async_trait::async_trait;
use clap;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::errors::Result;

#[derive(clap::Args)]
pub struct Init {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

#[async_trait]
impl CliCommand for Init {
    async fn run(&self) -> Result<()> {
        let conf = load_config_file(&self.config).unwrap();
        // TODO: check the components API version
        conf.generate_docker_files()
    }
}
