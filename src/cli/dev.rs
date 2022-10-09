pub use clap::Args;

use crate::cli::build::Build;
use crate::cli::logs::{Logs, Service};
use crate::cli::start::Start;
use crate::cli::CliCommand;
use crate::config::DEFAULT_CONFIG_FILE;

#[derive(Args)]
pub struct Dev {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl CliCommand for Dev {
    fn run(&self) {
        log::info!("Run dev mode");

        let build = Build {
            config: self.config.clone(),
        };
        log::debug!("Run build");
        build.run();

        let start = Start {
            config: self.config.clone(),
        };
        log::debug!("Run start");
        start.run();

        let logs = Logs {
            services: vec![Service::App],
            follow: true,
            ..Default::default()
        };
        log::debug!("Run logs");
        logs.run();
    }
}
