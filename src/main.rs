//! # lenra_cli
//!
//! The Lenra's command line interface

use clap::Parser;
use cli::{terminal::start_terminal, Cli, CliCommand, CommandContext};
use env_logger;

mod app_checker;
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
    if args.verbose {
        command::set_inherit_stdio(true);
    }
    let res = match args.command {
        Some(command) => command.run(context).await,
        None => start_terminal(context).await,
    };
    match res {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e.to_string());
            std::process::exit(1);
        }
    }
}
