use std::fs;

use crate::lenra;
pub use clap::{Args, Parser, Subcommand};
use clap::{CommandFactory, FromArgMatches};
use colored::{Color, Colorize};
use dirs::config_dir;
use log::{debug, warn};
use rustyline::{error::ReadlineError, Editor};

use crate::{
    docker_compose::Service,
    errors::{Error, Result},
};

use crate::cli::{check::Check, logs::Logs, CliCommand};

use super::{
    build::Build, dev::Dev, reload::Reload, start::Start, stop::Stop, update::Update,
    upgrade::Upgrade, CommandContext,
};

const LENRA_COMMAND: &str = "lenra";
const READLINE_PROMPT: &str = "[lenra]$ ";
// const ESCAPE_EVENT: KeyEvent = KeyEvent(KeyCode::Esc, Modifiers::NONE);

pub async fn start_terminal(context: CommandContext) -> Result<()> {
    let history_path = config_dir()
        .ok_or(Error::Custom("Can't get the user config directory".into()))?
        .join("lenra")
        .join("dev.history");
    let mut rl = Editor::<()>::new()?;

    debug!("Load history from {:?}", history_path);
    if rl.load_history(&history_path).is_err() {
        debug!("No previous history.");
    }

    let mut context = context.clone();

    loop {
        let readline = rl.readline(READLINE_PROMPT);
        let command = match readline {
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
        };

        debug!("Run command {:#?}", command);
        let (ctx_opt, keep_running) = run_command(&command, context.clone()).await;
        if !keep_running {
            break;
        }
        if let Some(ctx) = ctx_opt {
            context = ctx.clone();
        }
    }
    debug!("Save history to {:?}", history_path);
    fs::create_dir_all(history_path.parent().unwrap())?;
    rl.save_history(&history_path).map_err(Error::from)
}

pub async fn run_command(
    command: &TerminalCommand,
    context: CommandContext,
) -> (Option<CommandContext>, bool) {
    debug!("Run command {:#?}", command);
    let context = command.run(context).await.unwrap_or_else(|error| {
        eprintln!("{}", error.to_string().color(Color::Red));
        None
    });
    let keep_running = match command {
        TerminalCommand::Exit | TerminalCommand::Stop(_) => false,
        _ => true,
    };
    (context, keep_running)
}

fn parse_command_line(line: String) -> Result<TerminalCli, clap::Error> {
    let args = &mut line.split_whitespace().collect::<Vec<&str>>();

    let first_arg = if args.len() > 0 { Some(args[0]) } else { None };
    if let Some(arg) = first_arg {
        if LENRA_COMMAND != arg {
            args.push(LENRA_COMMAND);
            args.rotate_right(1);
        }
    }
    debug!("Try to parse dev terminal command: {:?}", args);

    let command = <TerminalCli as CommandFactory>::command();
    let mut matches = command
        .clone()
        .try_get_matches_from(args.clone())
        .map_err(format_error)?;
    <TerminalCli as FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error)
}

fn format_error(err: clap::Error) -> clap::Error {
    let mut command = <TerminalCli as CommandFactory>::command();
    err.format(&mut command)
}

/// The Lenra interactive command line interface
#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct TerminalCli {
    #[clap(subcommand)]
    pub command: TerminalCommand,
}

/// The interactive commands
#[derive(Subcommand, Clone, Debug)]
pub enum TerminalCommand {
    /// Build your app in release mode
    Build(Build),
    /// Start your app previously built with the build command
    Start(Start),
    /// View output from the containers
    Logs(Logs),
    /// Stop your app previously started with the start command
    Stop(Stop),
    /// Start the app in an interactive mode
    Dev(Dev),
    /// Upgrade the app with the last template updates
    Upgrade(Upgrade),
    /// Update the tools Docker images
    Update(Update),
    /// Checks the running app
    Check(Check),
    /// Reload the app by rebuilding and restarting it
    Reload(Reload),
    /// Exits the terminal
    Exit,
    /// Exposes the app ports
    Expose(Expose),
}

#[derive(Args, Clone, Debug)]
pub struct Expose {
    /// The service list to expose
    #[clap(value_enum, default_values = &["app", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

impl TerminalCommand {
    pub async fn run(&self, context: CommandContext) -> Result<Option<CommandContext>> {
        log::debug!("Run terminal command {:?}", self);
        match self {
            TerminalCommand::Exit => {}
            TerminalCommand::Expose(expose) => {
                lenra::generate_app_env(&context.config_path, &expose.services, false).await?;
                lenra::start_env().await?;

                let mut ctx = context.clone();
                ctx.expose = expose.services.clone();
                return Ok(Some(ctx));
            }
            TerminalCommand::Build(build) => build.run(context).await?,
            TerminalCommand::Start(start) => start.run(context).await?,
            TerminalCommand::Logs(logs) => logs.run(context).await?,
            TerminalCommand::Stop(stop) => stop.run(context).await?,
            TerminalCommand::Dev(dev) => dev.run(context).await?,
            TerminalCommand::Upgrade(upgrade) => upgrade.run(context).await?,
            TerminalCommand::Update(update) => update.run(context).await?,
            TerminalCommand::Check(check) => check.run(context).await?,
            TerminalCommand::Reload(reload) => reload.run(context).await?,
        };
        Ok(None)
    }
}
