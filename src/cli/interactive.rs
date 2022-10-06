use std::fs;

pub use clap::{Args, Parser, Subcommand};
use clap::{CommandFactory, FromArgMatches};
use dirs::config_dir;
use log::debug;
use rustyline::{error::ReadlineError, Editor};

use super::{logs::Logs, stop::Stop, CliCommand};

const LENRA_COMMAND: &str = "lenra";
const READLINE_PROMPT: &str = "[lenra]$ ";

pub fn run_interactive_command() -> Result<(), ReadlineError> {
    let history_path = config_dir()
        .expect("Can't get the user config directory")
        .join("lenra")
        .join("dev.history");
    let mut rl = Editor::<()>::new()?;

    debug!("Load history from {:?}", history_path);
    if rl.load_history(&history_path).is_err() {
        debug!("No previous history.");
    }

    loop {
        let readline = rl.readline(READLINE_PROMPT);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let result = parse_command_line(line);

                match result {
                    Ok(interactive) => interactive.command.run(),
                    Err(e) => {
                        e.print().expect("Could not print error");
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                debug!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                debug!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    debug!("Save history to {:?}", history_path);
    fs::create_dir_all(history_path.parent().unwrap())?;
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
    let command = <Interactive as CommandFactory>::command();
    let mut matches = command.clone().try_get_matches_from(args.clone())
        .map_err(format_error)?;
    <Interactive as FromArgMatches>::from_arg_matches_mut(&mut matches)
        .map_err(format_error)
}

fn format_error(err: clap::Error) -> clap::Error {
    let mut command = <Interactive as CommandFactory>::command();
    err.format(&mut command)
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
            InteractiveCommand::Logs(_logs) => println!("logs"), //logs.run(),
            InteractiveCommand::Stop(_stop) => println!("stop"), //stop.run(),
        };
    }
}
