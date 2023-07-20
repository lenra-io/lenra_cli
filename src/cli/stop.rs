use std::process::Stdio;

use async_trait::async_trait;
pub use clap::Args;
use log::warn;

use crate::cli::CliCommand;
use crate::docker_compose::create_compose_command;
use crate::errors::{CommandError, Error, Result};

#[derive(Args, Debug, Clone)]
pub struct Stop;

#[async_trait]
impl CliCommand for Stop {
    async fn run(&self) -> Result<()> {
        log::info!("Stoping the app");

        let mut command = create_compose_command();

        command
            .arg("down")
            .arg("--volumes")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        log::debug!("cmd: {:?}", command);
        let output = command.spawn()?.wait_with_output().await?;
        if !output.status.success() {
            warn!("An error occured while stoping the docker-compose app");
            return Err(Error::Command(CommandError { command, output }));
        }
        Ok(())
    }
}
