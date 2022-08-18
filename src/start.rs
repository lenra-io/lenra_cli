pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::DEFAULT_CONFIG_FILE;

#[derive(Args)]
pub struct Start {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl CliCommand for Start {
    fn run(&self) {
        todo!()
    }
}
