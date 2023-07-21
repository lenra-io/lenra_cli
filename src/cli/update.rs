use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::docker_compose;
use crate::docker_compose::Service;
use crate::errors::Result;

use super::CommandContext;

#[derive(Args, Debug, Clone)]
pub struct Update {
    /// The service list to pull
    #[clap(value_enum, default_values = &["devtool", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

#[async_trait]
impl CliCommand for Update {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        docker_compose::compose_pull(
            self.services
                .iter()
                .map(|service| service.to_str())
                .collect(),
        )
        .await
    }
}
