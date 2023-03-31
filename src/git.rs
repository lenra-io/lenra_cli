use crate::errors::{CommandError, Error, Result};
use tokio::process::Command;

#[cfg(test)]
use mocktopus::macros::mockable;

#[cfg_attr(test, mockable)]
pub async fn get_current_branch() -> Result<String> {
    let mut command = Command::new("git");
    let output = command
        .kill_on_drop(true)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .await?;

    if !output.status.success() {
        return Err(Error::Command(CommandError { command, output }));
    }

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}

#[cfg_attr(test, mockable)]
pub async fn get_current_commit() -> Result<String> {
    let mut command = Command::new("git");
    let output = command
        .kill_on_drop(true)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .await?;

    if !output.status.success() {
        return Err(Error::Command(CommandError { command, output }));
    }

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}
