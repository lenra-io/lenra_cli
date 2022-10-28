use std::process::Stdio;

pub use clap::Args;

use crate::cli::CliCommand;
use crate::docker_compose::create_compose_command;
use crate::errors::Result;

#[derive(Args, Clone)]
pub struct Stop;

impl CliCommand for Stop {
    fn run(&self) -> Result<()> {
        log::info!("Stoping the app");

        let mut command = create_compose_command();

        command
            .arg("down")
            .arg("--volumes")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

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
        Ok(())
    }
}
