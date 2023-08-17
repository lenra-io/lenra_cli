use std::{process::Output, string::FromUtf8Error};

use rustyline::error::ReadlineError;
use thiserror::Error;
use tokio::{process::Command, task::JoinError};

use crate::docker_compose::Service;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not open file {1}: {0}")]
    OpenFile(std::io::Error, std::path::PathBuf),
    #[error("StdIO error {0}")]
    Stdio(#[from] std::io::Error),
    #[error("Error while deserializing the document: {0}")]
    Deserialize(#[from] serde_yaml::Error),
    #[error("Error while deserializing the document: {0}")]
    DeserializeJson(#[from] serde_json::Error),
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
    #[error("{0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("The {0} service is not exposed")]
    ServiceNotExposed(Service),
    #[error("Some services are not started")]
    NotStartedServices,
    #[error("The app must be built before running it")]
    NeverBuiltApp,
    #[error("The new project directory is not empty")]
    ProjectPathNotEmpty,
    #[error("Check error")]
    Check,
    #[error("The next GitHub topic is not correct: {0}")]
    InvalidGitHubTopic(String),
    #[error("No template found")]
    NoTemplateFound,
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug)]
pub struct CommandError {
    pub command: Command,
    pub output: Output,
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
