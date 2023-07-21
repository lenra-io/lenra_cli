use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
pub use clap::Args;
use tokio::select;

use crate::cli::{
    build::Build,
    dev::interactive::listen_interactive_command,
    logs::Logs,
    start::Start,
    terminal::{run_command, TerminalCommand},
    CliCommand, CommandContext,
};
use crate::config::DEFAULT_CONFIG_FILE;
use crate::docker_compose::Service;
use crate::errors::Result;

use interactive::{InteractiveCommand, KeyboardShorcut};

mod interactive;

#[derive(Args, Debug, Clone)]
pub struct Dev {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes services ports.
    #[clap(long, value_enum, default_values = &[], default_missing_values = &["app", "postgres", "mongo"])]
    pub expose: Vec<Service>,
}

#[async_trait]
impl CliCommand for Dev {
    async fn run(&self, context: CommandContext) -> Result<()> {
        log::info!("Run dev mode");

        let build = Build {
            ..Default::default()
        };
        log::debug!("Run build");
        build.run(context.clone()).await?;

        let previous_log = Logs {
            services: vec![Service::App],
            follow: true,
            ..Default::default()
        };
        let mut last_logs = Utc::now();

        log::debug!("Run start");
        Start.run(context.clone()).await?;

        let mut cmd_context = context;

        InteractiveCommand::Help.to_value();
        let mut interactive_cmd = None;
        loop {
            if let Some(command) = interactive_cmd {
                let (ctx_opt, keep_running) = run_command(&command, cmd_context.clone()).await;
                if !keep_running {
                    break;
                }
                if let Some(ctx) = ctx_opt {
                    cmd_context = ctx.clone();
                }
            }
            (last_logs, interactive_cmd) =
                run_logs(&previous_log, Some(last_logs), cmd_context.clone()).await?;
        }

        log::debug!("End of dev mode");
        Ok(())
    }
}

async fn run_logs(
    logs: &Logs,
    last_end: Option<DateTime<Utc>>,
    context: CommandContext,
) -> Result<(DateTime<Utc>, Option<TerminalCommand>)> {
    let mut clone = logs.clone();
    if let Some(last_logs) = last_end {
        // Only displays new logs
        clone.since = Some(last_logs.to_rfc3339_opts(SecondsFormat::Secs, true));
    }

    let command = select! {
        res = listen_interactive_command() => {res?}
        res = clone.run(context) => {res?; None}
        // res = tokio::signal::ctrl_c() => {res?; None}
    };
    Ok((Utc::now(), command))
}
