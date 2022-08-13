pub use clap::Args;

use crate::cli::CliSubcommand;

#[derive(Args)]
pub struct Dev {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = "lenra.config.yml")]
    pub config: std::path::PathBuf,
}

impl CliSubcommand for Dev {
    fn run(&self) {
        todo!()
    }
}