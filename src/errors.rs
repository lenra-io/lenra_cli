use rustyline::error::ReadlineError;
use thiserror::Error;
use tokio::task::JoinError;

use crate::docker_compose;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not open file: {0}")]
    OpenFile(#[from] std::io::Error),
    #[error("Error while deserializing the JSON document: {0}")]
    DeserializeJson(#[from] serde_json::Error),
    #[error("Error while deserializing the YAML document: {0}")]
    DeserializeYaml(#[from] serde_yaml::Error),
    #[error("Could not read command: {0}")]
    ReadLine(#[from] ReadlineError),
    #[error("Could not parse command: {0}")]
    ParseCommand(#[from] clap::Error),
    #[error("Error while requesting: {0}")]
    Request(#[from] ureq::Error),
    #[error("Error while joining an async task: {0}")]
    JoinError(#[from] JoinError),
    #[error("Error while running a Docker Compose command: {0}")]
    ComposeError(#[from] docker_compose::Error),
    #[error("Check error")]
    CheckError,
    #[error("{0}")]
    Custom(String),
}
