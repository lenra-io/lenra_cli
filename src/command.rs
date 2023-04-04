use tokio::process::Command;

use crate::errors::{CommandError, Error, Result};

pub fn create_command(cmd: &str) -> Command {
    let mut cmd = Command::new(cmd);
    cmd.kill_on_drop(true);
    cmd
}

pub async fn get_command_output(command: Command) -> Result<String> {
    let mut command = Command::from(command);
    let output = command.output().await?;

    if !output.status.success() {
        return Err(Error::Command(CommandError { command, output }));
    }

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}
