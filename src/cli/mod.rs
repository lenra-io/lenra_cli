use async_trait::async_trait;
pub use clap::{Args, Parser, Subcommand};

use crate::errors::Result;

use self::{
    build::Build, check::Check, dev::Dev, logs::Logs, new::New, start::Start, stop::Stop,
    update::Update, upgrade::Upgrade,
};

mod build;
mod check;
mod dev;
mod logs;
mod new;
mod start;
mod stop;
mod update;
mod upgrade;

/// The Lenra command line interface
#[derive(Parser)]
#[clap(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[async_trait]
pub trait CliCommand {
    async fn run(&self) -> Result<()>;
}

/// The subcommands
#[derive(Subcommand)]
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
}

#[async_trait]
impl CliCommand for Command {
    async fn run(&self) -> Result<()> {
        match self {
            Command::New(new) => new.run(),
            Command::Build(build) => build.run(),
            Command::Start(start) => start.run(),
            Command::Logs(logs) => logs.run(),
            Command::Stop(stop) => stop.run(),
            Command::Dev(dev) => dev.run(),
            Command::Upgrade(upgrade) => upgrade.run(),
            Command::Update(update) => update.run(),
            Command::Check(check) => check.run(),
        }
        .await
    }
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
