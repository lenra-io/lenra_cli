use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::docker_compose::Service;
use crate::errors::Result;
use crate::lenra;

use super::{loader, CommandContext};

#[derive(Args, Debug, Clone)]
pub struct Update {
    /// The service list to pull
    #[clap(value_enum, default_values = &["devtool", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

#[async_trait]
impl CliCommand for Update {
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        update_loader(context, &self.services).await
    }
}

pub async fn update_loader(context: &mut CommandContext, services: &Vec<Service>) -> Result<()> {
    loader(
        "Update environment images...",
        "Environment images updated",
        "Failed updating environment images",
        || async { lenra::update_env_images(context, services).await },
    )
    .await
}
