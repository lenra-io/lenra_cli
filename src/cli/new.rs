//! # new
//!
//! The new subcommand creates a new Lenra app project from a given template and into a given path

use async_trait::async_trait;
pub use clap::Args;
use log;
use std::fs;

use crate::cli::CliCommand;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::get_current_commit;
use crate::template::{clone_template, normalize_template, TEMPLATE_DATA_FILE, TEMPLATE_GIT_DIR};

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

#[async_trait]
impl CliCommand for New {
    async fn run(&self) -> Result<()> {
        let template = normalize_template(self.template.clone());

        clone_template(template.clone(), self.path.clone()).await?;

        // create `.template` file to save template repo url and commit
        let commit = get_current_commit().await?;
        fs::write(
            self.path.join(TEMPLATE_DATA_FILE),
            format!("{}\n{}", template, commit),
        )
        .map_err(Error::from)?;

        log::debug!("move git directory");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();
        fs::rename(
            self.path.join(".git"),
            self.path.join(LENRA_CACHE_DIRECTORY).join(TEMPLATE_GIT_DIR),
        )
        .unwrap();

        Ok(())
    }
}
