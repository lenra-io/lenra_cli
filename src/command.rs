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
    cmd
}

pub async fn run_command(command: &mut Command, inherit_stdio: Option<bool>) -> Result<Output> {
    log::debug!("cmd: {:?}", command);
    let inherit_stdio = if let Some(inherit_stdio) = inherit_stdio {
        inherit_stdio
    } else {
        is_inherit_stdio()
    };
    let output = if inherit_stdio {
        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait_with_output()
            .await?
    } else {
        command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await?
    };

    if !output.status.success() {
        return Err(Error::Command(CommandError {
            command: format!("{:?}", command),
            output,
        }));
    }

    Ok(output)
}

pub async fn get_command_output(command: &mut Command) -> Result<String> {
    let output = run_command(command, Some(false)).await?;

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}
