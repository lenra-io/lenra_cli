use std::process::{Output, Stdio};

use tokio::process::Command;

use crate::errors::{CommandError, Error, Result};

static mut INHERIT_STDIO: bool = false;

pub fn is_inherit_stdio() -> bool {
    unsafe { INHERIT_STDIO }
}

pub fn set_inherit_stdio(val: bool) {
    unsafe {
        INHERIT_STDIO = val;
    }
}

pub fn create_command(cmd: &str) -> Command {
    let mut cmd = Command::new(cmd);
    cmd.kill_on_drop(true);
    if is_inherit_stdio() {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    cmd
}

pub async fn run_command(command: Command) -> Result<Output> {
    let mut command = Command::from(command);

    log::debug!("cmd: {:?}", command);
    let output = command.output().await?;

    if !output.status.success() {
        return Err(Error::Command(CommandError { command, output }));
    }

    Ok(output)
}

pub async fn get_command_output(command: Command) -> Result<String> {
    let output = run_command(command).await?;

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}
