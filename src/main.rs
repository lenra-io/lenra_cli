//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{Cli, CliCommand, CommandContext};
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
mod lenra;
mod matching;
mod template;

#[tokio::main]
async fn main() -> () {
    env_logger::init();
    let args = Cli::parse();
    let context = CommandContext {
        config: args.config.clone(),
        expose: args.expose.clone(),
        verbose: args.verbose,
    };
    match args.command.run(context).await {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e.to_string());
            std::process::exit(1);
        }
    }
}
