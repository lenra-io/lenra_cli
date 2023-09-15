use async_trait::async_trait;
use clap;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::lenra;

use super::CommandContext;

#[derive(clap::Args, Debug, Clone)]
pub struct Upgrade;

#[async_trait]
impl CliCommand for Upgrade {
    async fn run(&self, _context: &mut CommandContext) -> Result<()> {
        lenra::upgrade_app().await
    }
}
