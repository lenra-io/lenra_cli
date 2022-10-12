use std::process::Stdio;

pub use clap::Args;
use log::warn;

use crate::cli::CliCommand;
use crate::docker_compose::{create_compose_command, Service};

#[derive(Args, Default)]
pub struct Logs {
    /// Follow log output
    #[clap(short, long, action)]
    pub follow: bool,

    /// Produce monochrome output
    #[clap(long, action)]
    pub no_color: bool,

    /// Don't print prefix in logs
    #[clap(long, action)]
    pub no_log_prefix: bool,

    /// Show logs since timestamp (e.g. 2013-01-02T13:23:37Z) or relative (e.g. 42m for 42 minutes)
    #[clap(long)]
    pub since: Option<String>,

    /// Number of lines to show from the end of the logs for each container
    #[clap(long, default_value = "all")]
    pub tail: String,

    /// Show timestamps
    #[clap(short, long, action)]
    pub timestamps: bool,

    /// Show logs before a timestamp (e.g. 2013-01-02T13:23:37Z) or relative (e.g. 42m for 42 minutes)
    #[clap(long)]
    pub until: Option<String>,

    /// The logged service list
    #[clap(value_enum, default_value = "app")]
    pub services: Vec<Service>,
}

impl CliCommand for Logs {
    fn run(&self) {
        log::info!("Show logs");

        let mut command = create_compose_command();

        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("logs")
            .arg("--tail")
            .arg(self.tail.clone());

        if self.follow {
            command.arg("--follow");
        }
        if self.no_color {
            command.arg("--no-color");
        }
        if self.no_log_prefix {
            command.arg("--no-log-prefix");
        }
        if let Some(since) = self.since.clone() {
            command.arg("--since").arg(since);
        }
        if self.timestamps {
            command.arg("--timestamps");
        }
        if let Some(until) = self.until.clone() {
            command.arg("--until").arg(until);
        }
        self.services.iter().for_each(|service| {
            command.arg(service.to_str());
        });

        log::debug!("cmd: {:?}", command);
        let output = command
            .output()
            .expect("Failed to logs the docker-compose app");

        if !output.status.success() {
            warn!(
                "An error occured while displaying the docker-compose logs:\n{}\n{}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
    }
}
