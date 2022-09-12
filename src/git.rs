use std::process::Command;

pub fn get_current_branch() -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8(out.stdout)
                .map(|name| name.trim().to_string())
                .ok();
        }
    }

    None
}
