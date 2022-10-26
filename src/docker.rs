use std::process;

pub fn pull(image: String) -> process::Command {
    let mut cmd = process::Command::new("docker");

    cmd.arg("pull").arg(image);

    cmd
}
