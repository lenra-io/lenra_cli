use async_trait::async_trait;
pub use clap::Args;

use crate::cli::build::Build;
use crate::cli::interactive::{run_interactive_command, InteractiveContext};
use crate::cli::start::Start;
use crate::cli::stop::Stop;
use crate::cli::CliCommand;
use crate::config::DEFAULT_CONFIG_FILE;
use crate::errors::Result;

#[derive(Args)]
pub struct Dev {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    #[clap(long, action)]
    pub expose: bool,
}

#[async_trait]
impl CliCommand for Dev {
    async fn run(&self) -> Result<()> {
        log::info!("Run dev mode");

        let build = Build {
            config: self.config.clone(),
            expose: self.expose,
            ..Default::default()
        };
        log::debug!("Run build");
        build.run().await?;

        let start = Start {
            config: self.config.clone(),
            expose: self.expose,
            ..Default::default()
        };
        log::debug!("Run start");
        start.run().await?;

        run_interactive_command(&InteractiveContext {
            config: self.config.clone(),
            expose: self.expose,
        })
        .await?;

        let stop = Stop;
        log::debug!("Run stop");
        stop.run().await?;

        log::debug!("End of dev mode");
        Ok(())
    }
}
