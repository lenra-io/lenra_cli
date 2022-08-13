//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliSubcommand};
use env_logger;

mod build;
mod cli;
mod config;
mod dev;
mod new;
mod start;

fn main() {
    env_logger::init();
    let args = Cli::parse();
    // TODO: manage config file
    args.command.run();
}
