use std::process::Stdio;

pub use clap::Args;

use crate::cli::CliCommand;
use crate::docker_compose::{create_compose_command, DEVTOOL_SERVICE_NAME, POSTGRES_SERVICE_NAME, MONGO_SERVICE_NAME};

#[derive(Args, Clone)]
pub struct Update;

impl CliCommand for Update {
    fn run(&self) {
        log::info!("Updating Docker images");

        let mut command = create_compose_command();

        command
            .arg("pull")
            .arg(DEVTOOL_SERVICE_NAME)
            .arg(POSTGRES_SERVICE_NAME)
            .arg(MONGO_SERVICE_NAME)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        log::debug!("cmd: {:?}", command);
        let output = command
            .output()
            .expect("Failed to update the Docker images");
        if !output.status.success() {
            panic!(
                "An error occured while updating Docker images:\n{}\n{}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
    }
}
