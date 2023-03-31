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
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::get_current_commit;

#[derive(Args)]
pub struct New {
    /// The project template from which your project will be created.
    /// For example, defining `rust` or `template-rust` will use the next one: https://github.com/lenra-io/template-rust
    /// You can find all our templates at this url: https://github.com/orgs/lenra-io/repositories?q=&type=template&language=&sort=stargazers
    /// You also can set the template project full url to use custom ones.
    pub template: String,

    /// The project path
    #[clap(parse(from_os_str), default_value = ".")]
    path: std::path::PathBuf,
}

lazy_static! {
    static ref TEMPLATE_SHORT_REGEX: Regex =
        Regex::new(r"^(template-)?([0-9a-zA-Z]+([_-][0-9a-zA-Z]+)*)$").unwrap();
}

#[async_trait]
impl CliCommand for New {
    async fn run(&self) -> Result<()> {
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
            .kill_on_drop(true)
            .arg("clone")
            .arg("--single-branch")
            .arg("--depth")
            .arg("1")
            .arg(template.clone())
            .arg(self.path.as_os_str())
            .spawn()?
            .wait_with_output()
            .await
            .map_err(Error::from)?;

        // create `.template` file to save template repo url and commit
        let commit = get_current_commit().await?;
        fs::write(
            self.path.join(".template"),
            format!("{}\n{}", template, commit),
        )
        .map_err(Error::from)?;

        log::debug!("move git directory");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();
        fs::rename(
            self.path.join(".git"),
            self.path.join(".lenra").join("template"),
        )
        .unwrap();

        Ok(())
    }
}
