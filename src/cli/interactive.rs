use std::fs;

use chrono::{DateTime, SecondsFormat, Utc};
pub use clap::{Args, Parser, Subcommand};
use clap::{CommandFactory, FromArgMatches};
use dirs::config_dir;
use log::{debug, warn};
use rustyline::{error::ReadlineError, Editor};

use crate::{
    cli::{build::Build, start::Start},
    docker_compose::Service,
};

use super::{logs::Logs, CliCommand};

const LENRA_COMMAND: &str = "lenra";
const READLINE_PROMPT: &str = "[lenra]$ ";

pub fn run_interactive_command(context: &InteractiveContext) -> Result<(), ReadlineError> {
    let history_path = config_dir()
        .expect("Can't get the user config directory")
        .join("lenra")
        .join("dev.history");
    let mut rl = Editor::<()>::new()?;

    debug!("Load history from {:?}", history_path);
    if rl.load_history(&history_path).is_err() {
        debug!("No previous history.");
    }

    let mut previous_log = Logs {
        services: vec![Service::App],
        follow: true,
        ..Default::default()
    };
    let mut last_logs = run_logs(&previous_log, None);

    loop {
        let readline = rl.readline(READLINE_PROMPT);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let result = parse_command_line(line);

                match result {
                    Ok(interactive) => {
                        let logs_since = match interactive.command {
                            InteractiveCommand::Continue => Some(last_logs),
                            InteractiveCommand::Logs(logs) => {
                                previous_log = logs.clone();
                                None
                            }
                            InteractiveCommand::Stop => break,
                            cmd => {
                                cmd.run(context);
                                Some(last_logs)
                            }
                        };
                        last_logs = run_logs(&previous_log, logs_since);
                    }
                    Err(e) => {
                        e.print().expect("Could not print error");
                    }
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

fn run_logs(logs: &Logs, last_end: Option<DateTime<Utc>>) -> DateTime<Utc> {
    let mut clone = logs.clone();
    if let Some(last_logs) = last_end {
        // Only displays new logs
        clone.since = Some(last_logs.to_rfc3339_opts(SecondsFormat::Secs, true));
        // Follows the logs
        clone.follow = true;
    }
    clone.run();
    Utc::now()
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
    let mut matches = command
        .clone()
        .try_get_matches_from(args.clone())
        .map_err(format_error)?;
    <Interactive as FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error)
}

fn format_error(err: clap::Error) -> clap::Error {
    let mut command = <Interactive as CommandFactory>::command();
    err.format(&mut command)
}

pub struct InteractiveContext {
    /// The app configuration file.
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    pub expose: bool,
}

/// The Lenra interactive command line interface
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Interactive {
    #[clap(subcommand)]
    pub command: InteractiveCommand,
}

/// The interactive commands
#[derive(Subcommand, Clone)]
pub enum InteractiveCommand {
    /// Continue the previous logs command since the last displayed logs
    Continue,
    /// View output from the containers
    Logs(Logs),
    /// Reload the app by rebuilding and restarting it
    Reload,
    /// Stop your app previously started with the start command
    Stop,
}

impl InteractiveCommand {
    fn run(&self, context: &InteractiveContext) {
        match self {
            InteractiveCommand::Continue => warn!("The continue command should not be run"),
            InteractiveCommand::Logs(_logs) => println!("logs is not implemented yet"),
            InteractiveCommand::Stop => warn!("The stop command should not be run"),
            InteractiveCommand::Reload => {
                let build = Build {
                    config: context.config.clone(),
                    expose: context.expose,
                    ..Default::default()
                };
                log::debug!("Run build");
                build.run();

                let start = Start {
                    config: context.config.clone(),
                    expose: context.expose,
                    ..Default::default()
                };
                log::debug!("Run start");
                start.run();
            }
        };
    }
}
