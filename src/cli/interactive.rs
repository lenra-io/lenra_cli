use std::fs;

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

use super::{check::Check, logs::Logs, CliCommand};

const LENRA_COMMAND: &str = "lenra";
const READLINE_PROMPT: &str = "[lenra]$ ";

pub async fn run_interactive_command(initial_context: &InteractiveContext) -> Result<()> {
    // ctrlc::set_handler(move || {
    //     debug!("Stop asked");
    // })
    // .expect("Error setting Ctrl-C handler");

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
    let mut last_logs = run_logs(&previous_log, None).await?;
    let mut context = initial_context.clone();

    loop {
        let readline = rl.readline(READLINE_PROMPT);
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str());

                let command = parse_command_line(line.clone()).map_err(Error::from);
                match command {
                    Ok(interactive) => {
                        debug!("Run command {:#?}", interactive.command);
                        match interactive.command {
                            InteractiveCommand::Continue => {
                                last_logs = run_logs(&previous_log, Some(last_logs)).await?;
                            }
                            InteractiveCommand::Logs(logs) => {
                                previous_log = logs.clone();
                                last_logs = run_logs(&previous_log, None).await?;
                            }
                            InteractiveCommand::Stop | InteractiveCommand::Exit => break,
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
                    Err(Error::ParseCommand(clap_error)) => {
                        clap_error.print().ok();
                    }
                    Err(err) => {
                        debug!("Parse command error: {}", err);
                        warn!("not a valid command {}", line);
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
    rl.save_history(&history_path).map_err(Error::from)
}

async fn run_logs(logs: &Logs, last_end: Option<DateTime<Utc>>) -> Result<DateTime<Utc>> {
    let mut clone = logs.clone();
    if let Some(last_logs) = last_end {
        // Only displays new logs
        clone.since = Some(last_logs.to_rfc3339_opts(SecondsFormat::Secs, true));
        // Follows the logs
        clone.follow = true;
    }
    select! {
        res = clone.run() => {res?}
        _ = tokio::signal::ctrl_c() => {}
    }
    Ok(Utc::now())
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
    debug!("Try to parse interactive command: {:?}", args);

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

#[derive(Clone, Debug)]
pub struct InteractiveContext {
    /// The app configuration file.
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    pub expose: Vec<Service>,
}

/// The Lenra interactive command line interface
#[derive(Parser, Debug)]
#[clap(rename_all = "kebab-case")]
pub struct Interactive {
    #[clap(subcommand)]
    pub command: InteractiveCommand,
}

/// The interactive commands
#[derive(Subcommand, Clone, Debug)]
pub enum InteractiveCommand {
    /// Continue the previous logs command since the last displayed logs
    Continue,
    /// View output from the containers
    Logs(Logs),
    /// Reload the app by rebuilding and restarting it
    Reload,
    /// Stop your app previously started with the start command
    Stop,
    /// stop alias. Stop your app previously started with the start command
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

impl InteractiveCommand {
    async fn run(&self, context: &InteractiveContext) -> Result<Option<InteractiveContext>> {
        let conf = load_config_file(&context.config).unwrap();
        match self {
            InteractiveCommand::Continue => warn!("The continue command should not be run"),
            InteractiveCommand::Logs(_logs) => warn!("The logs command should not be run"),
            InteractiveCommand::Stop | InteractiveCommand::Exit => {
                warn!("The stop command should not be run")
            }
            InteractiveCommand::Reload => {
                log::debug!("Generates files");
                conf.generate_files(context.expose.clone()).await?;

                log::debug!("Docker compose build");
                compose_build().await?;

                log::debug!("Starts the containers");
                compose_up().await?;

                log::debug!("Stop the devtool app env to reset cache");
                let result = stop_app_env().await;
                if let Err(error) = result {
                    log::info!("{:?}", error);
                }
            }
            InteractiveCommand::Check(check) => {
                if context.expose.contains(&Service::App) {
                    check.run().await?
                } else {
                    println!("The check commands can't run if the ports are not exposed. Run the expose command first");
                }
            }
            InteractiveCommand::Expose(expose) => {
                conf.generate_files(expose.services.clone()).await?;

                compose_up().await?;

                let mut ctx = context.clone();
                ctx.expose = expose.services.clone();
                return Ok(Some(ctx));
            }
        };
        Ok(None)
    }
}
