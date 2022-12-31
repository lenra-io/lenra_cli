use std::process::Output;

use rustyline::error::ReadlineError;
use thiserror::Error;
use tokio::{process::Command, task::JoinError};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not open file: {0}")]
    OpenFile(#[from] std::io::Error),
    #[error("Error while deserializing the document: {0}")]
    Deserialize(#[from] serde_yaml::Error),
    #[error("{0}")]
    Dofigen(#[from] dofigen_lib::Error),
    #[error("Could not read command: {0}")]
    ReadLine(#[from] ReadlineError),
    #[error("Could not parse command: {0}")]
    ParseCommand(#[from] clap::Error),
    #[error("Error while requesting: {0}")]
    Request(#[from] ureq::Error),
    #[error("Error while joining an async task: {0}")]
    Join(#[from] JoinError),
    #[error("The command execution failed: {0}")]
    Command(#[from] CommandError),
    #[error("Check error")]
    Check,
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug)]
pub struct CommandError {
    command: Command,
    output: Output,
}

impl std::error::Error for CommandError {}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = self.output.clone();
        write!(
            f,
            "Command exec exited with code {}:\n\tcmd: {:?}\n\tstdout: {}\n\tstderr: {}",
            output.status.code().unwrap(),
            self.command,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        )
    }
}
