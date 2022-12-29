//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand};
use env_logger;

mod cli;
mod config;
mod docker;
mod docker_compose;
mod errors;
mod git;
mod matching;

fn main() -> Result<(), errors::Error> {
    env_logger::init();
    let args = Cli::parse();
    args.command.run()
}
