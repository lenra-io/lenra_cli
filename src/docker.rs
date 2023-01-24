use tokio::process::Command;

pub fn pull(image: String) -> Command {
    let mut cmd = Command::new("docker");
    cmd.kill_on_drop(true).arg("pull").arg(image);
    cmd
}
