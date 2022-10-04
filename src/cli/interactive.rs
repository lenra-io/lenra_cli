use std::io;

pub use clap::{Args, Parser, Subcommand};

use super::{logs::Logs, stop::Stop, CliCommand};

const LENRA_COMMAND: &str = "lenra";

pub fn run_interactive_command() {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input.");
    let args = &mut input.split_whitespace().collect::<Vec<&str>>();

    let first_arg = if args.len() > 0 { Some(args[0]) } else { None };
    if let Some(arg) = first_arg {
        if LENRA_COMMAND != arg {
            args.push(LENRA_COMMAND);
            args.rotate_right(1);
        }
    }

    let interactive = Interactive::parse_from(args.clone());
    interactive.command.run();
}

/// The Lenra interactive command line interface
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Interactive {
    #[clap(subcommand)]
    pub command: InteractiveCommand,
}

/// The interactive commands
#[derive(Subcommand)]
pub enum InteractiveCommand {
    /// View output from the containers
    Logs(Logs),
    /// Stop your app previously started with the start command
    Stop(Stop),
}

impl CliCommand for InteractiveCommand {
    fn run(&self) {
        match self {
            InteractiveCommand::Logs(logs) => logs.run(),
            InteractiveCommand::Stop(stop) => stop.run(),
        };
    }
}
