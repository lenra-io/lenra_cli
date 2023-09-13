use std::future::Future;

use async_trait::async_trait;
pub use clap::{Args, Parser, Subcommand};
use loading::Loading;

use crate::{
    config::{load_config_file, Application, DEFAULT_CONFIG_FILE},
    docker_compose::Service,
    errors::Result,
};

use self::{
    build::Build, check::Check, dev::Dev, logs::Logs, new::New, reload::Reload, start::Start,
    stop::Stop, update::Update, upgrade::Upgrade,
};

mod build;
mod check;
mod dev;
mod logs;
mod new;
mod reload;
mod start;
mod stop;
pub mod terminal;
mod update;
mod upgrade;

/// The Lenra command line interface
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The app configuration file.
    #[clap(global=true, parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes services ports.
    #[clap(global=true, long, value_enum, default_values = &[], default_missing_values = &["app", "postgres", "mongo"])]
    pub expose: Vec<Service>,

    /// Run the commands as verbose.
    #[clap(global = true, short, long, action)]
    pub verbose: bool,
}

#[async_trait]
pub trait CliCommand {
    async fn run(&self, context: &mut CommandContext) -> Result<()>;
    fn need_config(&self) -> bool {
        true
    }
}

/// The subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Create a new Lenra app project from a template
    New(New),
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
}

#[async_trait]
impl CliCommand for Command {
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        log::debug!("Run command {:?}", self);
        if self.need_config() {
            context.load_config()?;
        }
        match self {
            Command::New(new) => new.run(context),
            Command::Build(build) => build.run(context),
            Command::Start(start) => start.run(context),
            Command::Logs(logs) => logs.run(context),
            Command::Stop(stop) => stop.run(context),
            Command::Dev(dev) => dev.run(context),
            Command::Upgrade(upgrade) => upgrade.run(context),
            Command::Update(update) => update.run(context),
            Command::Check(check) => check.run(context),
            Command::Reload(reload) => reload.run(context),
        }
        .await
    }

    fn need_config(&self) -> bool {
        match self {
            Command::New(_) => false,
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommandContext {
    /// The app configuration file.
    pub config_path: std::path::PathBuf,

    /// The app configuration.
    pub config: Option<Application>,

    /// Exposes all services ports.
    pub expose: Vec<Service>,

    /// Run command as verbose.
    pub verbose: bool,
}

impl CommandContext {
    pub fn load_config(mut self) -> Result<()> {
        self.config = Some(load_config_file(&self.config_path)?);
        Ok(())
    }
}

pub async fn loader<F, Fut, R>(text: &str, success: &str, fail: &str, task: F) -> Result<R>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<R>>,
{
    let loading = Loading::default();
    loading.text(text);
    let res = task().await;
    if res.is_ok() {
        loading.success(success);
    } else {
        loading.fail(fail);
    }
    loading.end();
    res
}

#[cfg(test)]
mod test {
    use clap::{CommandFactory, FromArgMatches};

    use super::Cli;

    pub fn parse_command_line(line: String) -> Result<Cli, clap::Error> {
        let args = &mut line.split_whitespace().collect::<Vec<&str>>();
        let command = <Cli as CommandFactory>::command();
        let mut matches = command
            .clone()
            .try_get_matches_from(args.clone())
            .map_err(format_error)?;
        <Cli as FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error)
    }

    fn format_error(err: clap::Error) -> clap::Error {
        let mut command = <Cli as CommandFactory>::command();
        err.format(&mut command)
    }
}
