use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
pub use clap::Args;
use tokio::select;

use crate::docker_compose::Service;
use crate::errors::Result;
use crate::{
    cli::{
        dev::interactive::listen_interactive_command,
        logs::Logs,
        terminal::{run_command, TerminalCommand},
        CliCommand, CommandContext,
    },
    lenra,
};

use interactive::{InteractiveCommand, KeyboardShorcut};

mod interactive;

#[derive(Args, Debug, Clone)]
pub struct Dev {
    /// Attach the dev mode without rebuilding the app and restarting it.
    #[clap(long, action)]
    pub attach: bool,
}

#[async_trait]
impl CliCommand for Dev {
    async fn run(&self, context: CommandContext) -> Result<()> {
        log::info!("Run dev mode");

        if !self.attach {
            lenra::generate_app_env(&context.config, &context.expose, false).await?;
            lenra::build_app().await?;
            lenra::start_env().await?;
            lenra::clear_cache().await?;
        }

        let previous_log = Logs {
            services: vec![Service::App],
            follow: true,
            ..Default::default()
        };
        let mut last_logs: Option<DateTime<Utc>> = None;

        let mut cmd_context = context;

        lenra::display_app_access_url();
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
            let end_date;
            (end_date, interactive_cmd) =
                run_logs(&previous_log, last_logs, cmd_context.clone()).await?;
            last_logs = Some(end_date);
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
