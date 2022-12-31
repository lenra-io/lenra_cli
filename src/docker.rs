use tokio::process::Command;

pub fn pull(image: String) -> Command {
    *Command::new("docker").arg("pull").arg(image)
}
