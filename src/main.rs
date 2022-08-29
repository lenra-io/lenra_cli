//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand};
use env_logger;

mod build;
mod cli;
mod config;
mod docker_compose;
mod new;
mod start;
mod stop;

fn main() {
    env_logger::init();
    let args = Cli::parse();
    args.command.run();
}
