pub use clap::{Args, Parser, Subcommand};

use crate::{build::Build, dev::Dev, new::New, start::Start};

/// The Lenra CLI arguments to manage your local app development.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

pub trait CliSubcommand {
    fn run(&self);
}

/// The subcommands
#[derive(Subcommand)]
pub enum Command {
    /// Create a new Lenra app project
    New(New),
    /// Start your app in development mode
    Dev(Dev),
    /// Build your app in release mode
    Build(Build),
    /// Start your app previously built with the build command
    Start(Start),
}

impl CliSubcommand for Command {
    fn run(&self) {
        match self {
            Command::New(new) => new.run(),
            Command::Dev(dev) => dev.run(),
            Command::Build(build) => build.run(),
            Command::Start(start) => start.run(),
        };
    }
}
