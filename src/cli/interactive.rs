use std::{result, fs};

pub use clap::{Args, Parser, Subcommand};
use clap::{CommandFactory, FromArgMatches};
use dirs::config_dir;
use rustyline::{error::ReadlineError, Editor};

use crate::docker_compose::Error;

use super::{logs::Logs, stop::Stop, CliCommand};

const LENRA_COMMAND: &str = "lenra";

pub fn run_interactive_command() -> Result<(), ReadlineError> {
    let history_path = config_dir()
        .expect("Can't get the user config directory")
        .join("lenra")
        .join("dev.history");
    let mut rl = Editor::<()>::new()?;
    if rl.load_history(&history_path).is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("lenra$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
                let result = parse_command_line(line);

                match result {
                    Ok(interactive) => interactive.command.run(),
                    Err(e) => e.print()?,
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    println!("Save history to {:?}", history_path);
    fs::create_dir_all(history_path.parent().unwrap());
    rl.save_history(&history_path)
}

fn parse_command_line(line: String) -> Result<Interactive, clap::Error> {
    let args = &mut line.split_whitespace().collect::<Vec<&str>>();

    let first_arg = if args.len() > 0 { Some(args[0]) } else { None };
    if let Some(arg) = first_arg {
        if LENRA_COMMAND != arg {
            args.push(LENRA_COMMAND);
            args.rotate_right(1);
        }
    }
    let mut command = <Interactive as CommandFactory>::command();
    let mut matches = command.clone().get_matches_from(args.clone());
    <Interactive as FromArgMatches>::from_arg_matches_mut(&mut matches)
        .map_err(|err| err.format(&mut command))
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
            InteractiveCommand::Logs(logs) => println!("logs"), //logs.run(),
            InteractiveCommand::Stop(stop) => println!("stop"), //stop.run(),
        };
    }
}
