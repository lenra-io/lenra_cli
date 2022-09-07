//! # new
//!
//! The new subcommand creates a new Lenra app project from a given template and into a given path

pub use clap::Args;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use std::fs;
use std::process::Command;

use crate::cli::CliCommand;

#[derive(Args)]
pub struct New {
    /// The project template from which your project will be created.
    /// For example, defining `rust` or `template-rust` will use the next one: https://github.com/lenra-io/template-rust
    /// You can find all our templates at this url: https://github.com/orgs/lenra-io/repositories?q=&type=template&language=&sort=stargazers
    /// You also can set the template project full url to use custom ones.
    pub template: String,

    /// The project path
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

lazy_static! {
    static ref TEMPLATE_SHORT_REGEX: Regex =
        Regex::new(r"^(template-)?([0-9a-zA-Z]+([_-][0-9a-zA-Z]+)*)$").unwrap();
}

impl CliCommand for New {
    fn run(&self) {
        if self.path.exists() {
            panic!("The path '{}' already exists", self.path.display())
        }

        let template = if TEMPLATE_SHORT_REGEX.is_match(self.template.as_str()) {
            format!(
                "https://github.com/lenra-io/template-{}",
                TEMPLATE_SHORT_REGEX.replace(self.template.as_str(), "$2")
            )
        } else {
            self.template.clone()
        };

        log::debug!(
            "clone the template {} into {}",
            template,
            self.path.display()
        );
        match Command::new("git")
            .arg("clone")
            .arg("--single-branch")
            .arg("--depth")
            .arg("1")
            .arg(template)
            .arg(self.path.as_os_str())
            .spawn()
        {
            Ok(child) => {
                child
                    .wait_with_output()
                    .expect("Failed to clone the template");
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
