use tokio::process::Command;

pub fn pull(image: String) -> Command {
    let mut cmd = Command::new("docker");
    cmd.arg("pull").arg(image);
    cmd
}
