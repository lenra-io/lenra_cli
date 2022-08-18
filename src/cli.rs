pub use clap::{Args, Parser, Subcommand};

use crate::{build::Build, new::New, start::Start};

/// The Lenra CLI arguments to manage your local app development.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
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
}

impl CliCommand for Command {
    fn run(&self) {
        match self {
            Command::New(new) => new.run(),
            Command::Build(build) => build.run(),
            Command::Start(start) => start.run(),
        };
    }
}
