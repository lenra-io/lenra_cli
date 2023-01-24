use tokio::process::Command;

use crate::errors::{CommandError, Error, Result};

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
