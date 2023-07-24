//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand};
use env_logger;

mod cli;
mod command;
mod config;
mod devtool;
mod docker;
mod docker_compose;
mod errors;
mod git;
mod github;
mod keyboard_event;
mod matching;
mod template;

#[tokio::main]
async fn main() -> () {
    env_logger::init();
    let args = Cli::parse();
    match args.command.run().await {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e.to_string());
            std::process::exit(1);
        }
    }
}
