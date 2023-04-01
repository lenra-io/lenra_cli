use std::fs;
use std::path::Path;

use async_trait::async_trait;
use clap;

use crate::cli::CliCommand;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::Result;
use crate::git;
use crate::template::{clone_template, get_template_data, TEMPLATE_GIT_DIR, TEMPLATE_TEMP_DIR};

#[derive(clap::Args)]
pub struct Upgrade {}

#[async_trait]
impl CliCommand for Upgrade {
    async fn run(&self) -> Result<()> {
        // get template data
        let template_data = get_template_data().await?;
        let git_dir = Path::new(LENRA_CACHE_DIRECTORY).join(TEMPLATE_GIT_DIR);

        if git_dir.is_dir() {
            // update the template repo
            git::fetch(Some(git_dir)).await?;
        } else {
            let template_tmp = Path::new(LENRA_CACHE_DIRECTORY).join(TEMPLATE_TEMP_DIR);
            // clone template project
            clone_template(template_data.template, template_tmp.clone()).await?;
            fs::rename(template_tmp.join(".git"), git_dir)?;
            fs::remove_dir_all(template_tmp)?;
        }

        if let Some(_commit) = template_data.commit {
            
            // TODO: if commit exists and is in template branch, apply a patch
        } else {
            // TODO: checkout the template in the current dir
        }

        Ok(())
    }
}
