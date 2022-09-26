use clap;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};

#[derive(clap::Args)]
pub struct Init {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl CliCommand for Init {
    fn run(&self) {
        let conf = load_config_file(&self.config);
        // TODO: check the components API version

        conf.generate_files();
    }
}
