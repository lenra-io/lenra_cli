//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand};
use env_logger;
use errors::Result;

mod cli;
mod command;
mod config;
mod devtool;
mod docker;
mod docker_compose;
mod errors;
mod git;
mod keyboard_event;
mod matching;
mod template;

// #![cfg_attr(test, feature(proc_macro_hygiene))]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    args.command.run().await
}
