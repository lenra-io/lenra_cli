use tokio::process::Command;

pub async fn get_current_branch() -> Option<String> {
    let result = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .await;
    match result {
        Ok(out) => {
            if out.status.success() {
                return String::from_utf8(out.stdout)
                    .map(|name| name.trim().to_string())
                    .ok();
            }
            None
        }
        Err(_) => None,
    }
}
