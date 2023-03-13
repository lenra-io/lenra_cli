use std::fs;

use crate::cli::dev::interactive::listen_interactive_command;
use chrono::{DateTime, SecondsFormat, Utc};
pub use clap::{Args, Parser, Subcommand};
use clap::{CommandFactory, FromArgMatches};
use dirs::config_dir;
use log::{debug, warn};
use rustyline::{error::ReadlineError, Editor};
use tokio::select;

use crate::{
    config::load_config_file,
    devtool::stop_app_env,
    docker_compose::{compose_build, compose_up, Service},
    errors::{Error, Result},
};

use crate::cli::{check::Check, logs::Logs, CliCommand};

use super::interactive::{InteractiveCommand, KeyboardShorcut};

const LENRA_COMMAND: &str = "lenra";
const READLINE_PROMPT: &str = "[lenra]$ ";
// const ESCAPE_EVENT: KeyEvent = KeyEvent(KeyCode::Esc, Modifiers::NONE);

pub async fn run_dev_terminal(initial_context: &DevTermContext, terminal: bool) -> Result<()> {
    let history_path = config_dir()
        .ok_or(Error::Custom("Can't get the user config directory".into()))?
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
    let mut last_logs = Utc::now();
    let mut interactive_cmd = if !terminal {
        InteractiveCommand::Help.run();
        Some(DevTermCommand::Logs(previous_log.clone()))
    } else {
        None
    };
    let mut context = initial_context.clone();

    loop {
        let command = if let Some(cmd) = interactive_cmd {
            interactive_cmd = match cmd {
                DevTermCommand::Reload => Some(DevTermCommand::Continue),
                _ => None,
            };
            cmd
        } else {
            let readline = rl.readline(READLINE_PROMPT);
            match readline {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    rl.add_history_entry(line.as_str());

                    let parse_result = parse_command_line(line.clone()).map_err(Error::from);
                    match parse_result {
                        Ok(dev_cli) => dev_cli.command,
                        Err(Error::ParseCommand(clap_error)) => {
                            clap_error.print().ok();
                            continue;
                        }
                        Err(err) => {
                            debug!("Parse command error: {}", err);
                            warn!("not a valid command {}", line);
                            continue;
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
        };

        debug!("Run command {:#?}", command);
        match command {
            DevTermCommand::Continue => {
                (last_logs, interactive_cmd) = run_logs(&previous_log, Some(last_logs)).await?;
            }
            DevTermCommand::Logs(logs) => match logs {
                Logs { follow: true, .. } => {
                    previous_log = logs.clone();
                    (last_logs, interactive_cmd) = run_logs(&previous_log, None).await?;
                }
                _ => {
                    logs.run().await?;
                }
            },
            DevTermCommand::Stop | DevTermCommand::Exit => break,
            cmd => {
                let ctx = context.clone();
                match cmd.run(&ctx).await {
                    Ok(ctx_opt) => {
                        if let Some(ctx) = ctx_opt {
                            context = ctx.clone();
                        }
                    }
                    Err(error) => eprintln!("{}", error),
                }
            }
        }
    }
    debug!("Save history to {:?}", history_path);
    fs::create_dir_all(history_path.parent().unwrap())?;
    rl.save_history(&history_path).map_err(Error::from)
}

async fn run_logs(
    logs: &Logs,
    last_end: Option<DateTime<Utc>>,
) -> Result<(DateTime<Utc>, Option<DevTermCommand>)> {
    let mut clone = logs.clone();
    if let Some(last_logs) = last_end {
        // Only displays new logs
        clone.since = Some(last_logs.to_rfc3339_opts(SecondsFormat::Secs, true));
    }

    let command = select! {
        res = listen_interactive_command() => {res?}
        res = clone.run() => {res?; None}
        // res = tokio::signal::ctrl_c() => {res?; None}
    };
    Ok((Utc::now(), command))
}

fn parse_command_line(line: String) -> Result<DevCli, clap::Error> {
    let args = &mut line.split_whitespace().collect::<Vec<&str>>();

    let first_arg = if args.len() > 0 { Some(args[0]) } else { None };
    if let Some(arg) = first_arg {
        if LENRA_COMMAND != arg {
            args.push(LENRA_COMMAND);
            args.rotate_right(1);
        }
    }
    debug!("Try to parse dev terminal command: {:?}", args);

    let command = <DevCli as CommandFactory>::command();
    let mut matches = command
        .clone()
        .try_get_matches_from(args.clone())
        .map_err(format_error)?;
    <DevCli as FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error)
}

fn format_error(err: clap::Error) -> clap::Error {
    let mut command = <DevCli as CommandFactory>::command();
    err.format(&mut command)
}

#[derive(Clone, Debug)]
pub struct DevTermContext {
    /// The app configuration file.
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    pub expose: Vec<Service>,
}

/// The Lenra interactive command line interface
#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct DevCli {
    #[clap(subcommand)]
    pub command: DevTermCommand,
}

/// The interactive commands
#[derive(Subcommand, Clone, Debug)]
pub enum DevTermCommand {
    /// Continue the previous logs command since the last displayed logs
    Continue,
    /// View output from the containers
    Logs(Logs),
    /// Reload the app by rebuilding and restarting it
    Reload,
    /// Stop your app and the local Lenra environment
    Stop,
    /// 'stop' alias
    Exit,
    /// Checks the running app
    Check(Check),
    /// Exposes the app ports
    Expose(Expose),
}

#[derive(Args, Clone, Debug)]
pub struct Expose {
    /// The service list to expose
    #[clap(value_enum, default_values = &["app", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

impl DevTermCommand {
    async fn run(&self, context: &DevTermContext) -> Result<Option<DevTermContext>> {
        let conf = load_config_file(&context.config).unwrap();
        match self {
            DevTermCommand::Continue => warn!("The continue command should not be run"),
            DevTermCommand::Logs(_logs) => warn!("The logs command should not be run"),
            DevTermCommand::Stop | DevTermCommand::Exit => {
                warn!("The stop command should not be run")
            }
            DevTermCommand::Reload => {
                log::debug!("Generates files");
                conf.generate_files(context.expose.clone(), true).await?;

                log::debug!("Docker compose build");
                compose_build().await?;

                log::debug!("Starts the containers");
                compose_up().await?;

                log::debug!("Stop the devtool app env to reset cache");
                println!("Clearing cache");
                let result = stop_app_env().await;
                if let Err(error) = result {
                    log::error!("{:?}", error);
                }
            }
            DevTermCommand::Check(check) => {
                if context.expose.contains(&Service::App) {
                    check.run().await?
                } else {
                    println!("The check commands can't run if the ports are not exposed. Run the expose command first");
                }
            }
            DevTermCommand::Expose(expose) => {
                conf.generate_files(expose.services.clone(), true).await?;

                compose_up().await?;

                let mut ctx = context.clone();
                ctx.expose = expose.services.clone();
                return Ok(Some(ctx));
            }
        };
        Ok(None)
    }
}
