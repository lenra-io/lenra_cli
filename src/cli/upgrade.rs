use std::fs;
use std::path::Path;
use std::process::Stdio;

use async_trait::async_trait;
use clap;

use crate::cli::CliCommand;
use crate::command::get_command_output;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::{self, create_git_command, get_current_commit};
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
            git::fetch(Some(git_dir.clone())).await?;
        } else {
            let template_tmp = Path::new(LENRA_CACHE_DIRECTORY).join(TEMPLATE_TEMP_DIR);
            // clone template project
            clone_template(template_data.template, template_tmp.clone()).await?;
            fs::rename(template_tmp.join(".git"), git_dir.clone())?;
            fs::remove_dir_all(template_tmp)?;
        }

        if let Some(commit) = template_data.commit {
            let current_commit = get_current_commit(Some(git_dir.clone())).await?;
            if commit == current_commit {
                println!("This application is already up to date");
                return Ok(());
            }

            // get diff between previous commit and current commit
            let patch_file = Path::new(LENRA_CACHE_DIRECTORY)
                .join(format!("patch.{}-{}.diff", commit, current_commit));
            log::debug!(
                "create patch between {} and {}: {:?}",
                commit,
                current_commit,
                patch_file
            );
            let mut cmd = create_git_command();
            cmd.arg("--git-dir")
                .arg(git_dir.as_os_str())
                .arg("diff")
                .arg(commit)
                .arg(current_commit);
            let patch = get_command_output(cmd).await?;
            fs::write(patch_file.clone(), patch)?;

            // apply a patch
            log::debug!("apply patch on project");
            let mut cmd = create_git_command();
            cmd.arg("apply")
                .arg(patch_file)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            cmd.spawn()?.wait_with_output().await.map_err(Error::from)?;
        } else {
            // TODO: checkout the template in the current dir
        }

        Ok(())
    }
}
