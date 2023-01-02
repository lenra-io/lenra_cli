//! # new
//!
//! The new subcommand creates a new Lenra app project from a given template and into a given path

use async_trait::async_trait;
pub use clap::Args;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use std::fs;
use tokio::process::Command;

use crate::cli::CliCommand;
use crate::errors::{Error, Result};

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

#[async_trait]
impl CliCommand for New {
    async fn run(&self) -> Result<()> {
        if self.path.exists() {
            return Err(Error::Custom(format!(
                "The path '{}' already exists",
                self.path.display()
            )));
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
        Command::new("git")
            .arg("clone")
            .arg("--single-branch")
            .arg("--depth")
            .arg("1")
            .arg(template)
            .arg(self.path.as_os_str())
            .spawn()?
            .wait_with_output()
            .await
            .map_err(Error::from)?;

        log::debug!("remove git directory");
        fs::remove_dir_all(self.path.join(".git")).unwrap();

        log::debug!("init git project");
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("init")
            .spawn()?
            .wait_with_output()
            .await
            .map_err(Error::from)?;
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("add")
            .arg(".")
            .spawn()?
            .wait_with_output()
            .await
            .map_err(Error::from)?;
        Command::new("git")
            .current_dir(self.path.as_os_str())
            .arg("commit")
            .arg("-m")
            .arg("Init project")
            .spawn()?
            .wait_with_output()
            .await
            .map_err(Error::from)?;
        Ok(())
    }
}
