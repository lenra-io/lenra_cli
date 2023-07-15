use std::path::PathBuf;

use crate::{
    command::{create_command, get_command_output},
    errors::{Error, Result},
};
use lazy_static::lazy_static;
use regex::Regex;
use tokio::process;

#[cfg(test)]
use mocktopus::macros::mockable;

lazy_static! {
    pub static ref GIT_REPO_REGEX: Regex =
        Regex::new(r"^((git|ssh|http(s)?)|(git@[\w\.]+))(:(//)?)([\w\.@\:/\-~]+)(\.git)(/)?$").unwrap();
}

pub fn create_git_command() -> process::Command {
    create_command("git")
}

#[cfg_attr(test, mockable)]
pub async fn get_current_branch(git_dir: Option<PathBuf>) -> Result<String> {
    let mut cmd = create_git_command();
    if let Some(dir) = git_dir {
        cmd.arg("--git-dir").arg(dir.as_os_str());
    }
    cmd.arg("rev-parse").arg("--abbrev-ref").arg("HEAD");
    get_command_output(cmd).await
}

#[cfg_attr(test, mockable)]
pub async fn get_current_commit(git_dir: Option<PathBuf>) -> Result<String> {
    let mut cmd = create_git_command();
    if let Some(dir) = git_dir {
        cmd.arg("--git-dir").arg(dir.as_os_str());
    }
    cmd.arg("rev-parse").arg("HEAD");
    get_command_output(cmd).await
}

pub async fn pull(git_dir: Option<PathBuf>) -> Result<()> {
    log::debug!("git pull {:?}", git_dir);
    let mut cmd = create_git_command();

    if let Some(dir) = git_dir {
        cmd.arg("--git-dir").arg(dir.as_os_str());
    }

    cmd.arg("pull");

    cmd.spawn()?.wait_with_output().await.map_err(Error::from)?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Repository {
    pub name: String,
    pub description: String,
    pub url: String,
    pub stars: u32,
}

pub trait PlatformRepository {
    fn to_repository(&self) -> Repository;
}
