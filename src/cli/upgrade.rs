use async_trait::async_trait;
use clap;

use crate::cli::CliCommand;
use crate::errors::Result;
use crate::template::get_template_data;

#[derive(clap::Args)]
pub struct Upgrade {}

#[async_trait]
impl CliCommand for Upgrade {
    async fn run(&self) -> Result<()> {
        // get template data
        let _template_data = get_template_data().await?;

        // TODO: clone template project if not exists, else pull

        // TODO: if commit exists and is in template branch, apply a patch
        // TODO: checkout the template in the current dir
        Ok(())
    }
}
