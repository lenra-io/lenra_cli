pub use clap::Args;
use log::debug;

use crate::cli::build::Build;
use crate::cli::logs::Logs;
use crate::cli::start::Start;
use crate::cli::stop::Stop;
use crate::cli::CliCommand;
use crate::config::DEFAULT_CONFIG_FILE;
use crate::docker_compose::Service;

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
            ..Default::default()
        };
        log::debug!("Run build");
        build.run();

        let start = Start {
            config: self.config.clone(),
            ..Default::default()
        };
        log::debug!("Run start");
        start.run();

        ctrlc::set_handler(move || {
            debug!("Stop asked");
        })
        .expect("Error setting Ctrl-C handler");

        let logs = Logs {
            services: vec![Service::App],
            follow: true,
            ..Default::default()
        };
        log::debug!("Run logs");
        logs.run();

        let stop = Stop;
        log::debug!("Run stop");
        stop.run();

        log::debug!("End of dev mode");
    }
}
