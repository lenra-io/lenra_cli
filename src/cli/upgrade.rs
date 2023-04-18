use std::fs;
use std::path::Path;
use std::process::Stdio;

use async_trait::async_trait;
use clap;
use rustyline::Editor;

use crate::cli::CliCommand;
use crate::command::get_command_output;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::{self, create_git_command, get_current_commit};
use crate::template::{
    clone_template, get_template_data, save_template_data, TemplateData, TEMPLATE_GIT_DIR,
    TEMPLATE_TEMP_DIR,
};

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
            git::pull(Some(git_dir.clone())).await?;
        } else {
            let template_tmp = Path::new(LENRA_CACHE_DIRECTORY).join(TEMPLATE_TEMP_DIR);
            // clone template project
            clone_template(template_data.template.clone(), template_tmp.clone()).await?;
            fs::rename(template_tmp.join(".git"), git_dir.clone())?;
            fs::remove_dir_all(template_tmp)?;
        }

        let current_commit = get_current_commit(Some(git_dir.clone())).await?;
        if let Some(commit) = template_data.commit {
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
                .arg(current_commit.clone());
            let mut patch = get_command_output(cmd).await?;
            patch.push('\n');
            fs::write(patch_file.clone(), patch)?;

            // apply a patch
            log::debug!("apply patch on project");
            let mut cmd = create_git_command();
            cmd.arg("apply")
                .arg(patch_file.clone())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            let patch_file_str = patch_file.to_string_lossy();
            while !cmd.spawn()?.wait_with_output().await?.status.success() {
                println!("An error occured applying the patch {patch_file_str}");
                let mut rl = Editor::<()>::new()?;
                rl.readline("Fix it and press enter to retry")?;
            }
            fs::remove_file(patch_file)?;
        } else {
            // ask for user confirmation
            if !confirm_checkout()? {
                println!("Upgrade canceled");
                return Ok(());
            }

            // checkout the template in the current dir
            log::debug!("checkout the template");
            let mut cmd = create_git_command();
            cmd.arg("--git-dir")
                .arg(git_dir.as_os_str())
                .arg("checkout")
                .arg("HEAD")
                .arg("--")
                .arg(".")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            cmd.spawn()?.wait_with_output().await.map_err(Error::from)?;
        }
        // save template data
        save_template_data(TemplateData {
            template: template_data.template,
            commit: Some(current_commit),
        })
        .await
    }
}

fn confirm_checkout() -> Result<bool> {
    let mut rl = Editor::<()>::new()?;
    println!("There is no template last commit in this project, the template files will checked out to your app.\nMake sure your project is saved (for example with git).");
    loop {
        let res = rl
            .readline("Checkout the template ? [y/N] ")?
            .trim()
            .to_lowercase();
        if res == "y" || res == "yes" {
            return Ok(true);
        } else if res.is_empty() || res == "n" || res == "no" {
            return Ok(false);
        }
    }
}
