pub use clap::{Args, Parser, Subcommand};

use self::{
    build::Build, dev::Dev, init::Init, logs::Logs, new::New, start::Start, stop::Stop,
    update::Update,
};

mod build;
mod dev;
mod init;
mod interactive;
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

pub trait CliCommand {
    fn run(&self);
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
    /// Generates dockerfile and docker compose file with the init command
    Init(Init),
    /// Update the tools Docker images
    Update(Update),
}

impl CliCommand for Command {
    fn run(&self) {
        match self {
            Command::New(new) => new.run(),
            Command::Build(build) => build.run(),
            Command::Start(start) => start.run(),
            Command::Logs(logs) => logs.run(),
            Command::Stop(stop) => stop.run(),
            Command::Dev(dev) => dev.run(),
            Command::Init(init) => init.run(),
            Command::Update(update) => update.run(),
        };
    }
}
