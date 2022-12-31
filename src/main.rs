//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand};
use env_logger;
use errors::Result;

mod cli;
mod config;
mod devtool;
mod docker;
mod docker_compose;
mod errors;
mod git;
mod matching;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    args.command.run().await
}
