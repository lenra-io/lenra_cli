pub use clap::Args;
use log;
use std::fs;
use std::process::Command;

use crate::cli::CliSubcommand;

#[derive(Args)]
pub struct New {
    /// The project template
    pub template: String,

    /// The target project path
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,

    /// The project name
    #[clap(long)]
    pub name: Option<String>,
}

impl CliSubcommand for New {
    fn run(&self) {
        if self.path.exists() {
            panic!("The path '{}' already exists", self.path.display())
        }
        let template = format!("https://github.com/lenra-io/template-{}", self.template);
        let download_url = format!("{}/archive/refs/heads/main.zip", template);
        let name = if let Some(n) = &self.name {
            // TODO: check name pattern
            n.clone()
        } else {
            String::from(self.path.file_name().unwrap().to_str().unwrap())
        };
        println!(
            "new project {} from tempalte {}[{}]",
            name, template, download_url
        );

        log::debug!("clone the template");
        match Command::new("git")
            .arg("clone")
            .arg(template)
            .arg(self.path.as_os_str())
            .spawn()
        {
            Ok(child) => {
                child.wait_with_output().expect("Failed to clone the template");
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => panic!("`git` was not found!"),
                _ => panic!("Some unkown error occurred"),
            },
        }

        log::debug!("remove git directory");
        fs::remove_dir_all(self.path.join(".git")).unwrap();

        log::debug!("init git project");
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("init")
            .output()
            .expect("Failed initiating git project");
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("add")
            .arg(".")
            .output()
            .expect("Failed during git add");
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("commit")
            .arg("-m")
            .arg("Init project")
            .output()
            .expect("Failed during git add");
    }
}
