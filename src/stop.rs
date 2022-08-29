use std::path::PathBuf;
use std::process::{Command, Stdio};

pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH};

#[derive(Args)]
pub struct Stop {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl Stop {
    /// Starts the docker-compose
    fn stop_docker_compose(&self) {
        let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
        if !dockercompose_path.exists() {
            
        }

        let mut command = Command::new("docker");

        // TODO: display std out & err
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        command
            .arg("compose")
            .arg("-f")
            .arg(dockercompose_path)
            .arg("down");

        log::debug!("cmd: {:?}", command);
        let output = command
            .output()
            .expect("Failed to stop the docker-compose app");
        if !output.status.success() {
            panic!(
                "An error occured while stoping the docker-compose app:\n{}\n{}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
    }
}

impl CliCommand for Stop {
    fn run(&self) {
        log::info!("Stoping the app");
        self.stop_docker_compose();
    }
}
