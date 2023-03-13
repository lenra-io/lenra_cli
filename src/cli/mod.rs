use async_trait::async_trait;
pub use clap::{Args, Parser, Subcommand};

use crate::errors::Result;

use self::{
    build::Build, check::Check, dev::Dev, logs::Logs, new::New, start::Start, stop::Stop,
    update::Update,
};

mod build;
mod check;
mod dev;
mod logs;
mod new;
mod start;
mod stop;
mod update;

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
    /// Create a new Lenra app project
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
            Command::Update(update) => update.run(),
            Command::Check(check) => check.run(),
        }
        .await
    }
}
