use std::path::PathBuf;

use crate::{
    command::{create_command, get_command_output},
    errors::{Error, Result},
};
use tokio::process;

#[cfg(test)]
use mocktopus::macros::mockable;

pub fn create_git_command() -> process::Command {
    create_command("git")
}

#[cfg_attr(test, mockable)]
pub async fn get_current_branch() -> Result<String> {
    let mut command = create_git_command();
    command.arg("rev-parse").arg("--abbrev-ref").arg("HEAD");
    get_command_output(command).await
}

#[cfg_attr(test, mockable)]
pub async fn get_current_commit() -> Result<String> {
    let mut command = create_git_command();
    command.arg("rev-parse").arg("HEAD");
    get_command_output(command).await
}

pub async fn fetch(git_dir: Option<PathBuf>) -> Result<()> {
    log::debug!("git fetch {:?}", git_dir);
    let mut cmd = create_git_command();
    cmd.arg("fetch");

    if let Some(dir) = git_dir {
        cmd.arg("--git-dir").arg(dir.as_os_str());
    }

    cmd.spawn()?.wait_with_output().await.map_err(Error::from)?;

    Ok(())
}
