//! # new
//!
//! The new subcommand creates a new Lenra app project from a template

use async_trait::async_trait;
pub use clap::Args;
use log;
use std::fs;

use crate::cli::CliCommand;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::{get_current_commit, GIT_REPO_REGEX};
use crate::template::{
    choose_repository, clone_template, list_templates, TEMPLATE_DATA_FILE, TEMPLATE_GIT_DIR,
};

#[derive(Args)]
pub struct New {
    /// The project template topics from which your project will be created.
    /// For example, defining `rust` look for the next API endpoint: https://api.github.com/search/repositories?q=topic:lenra+topic:template+topic:rust&sort=stargazers
    /// You can find all the templates at this url: https://github.com/search?q=topic%3Alenra+topic%3Atemplate&sort=stargazers&type=repositories
    /// You also can set the template project full url to use custom ones.
    pub topics: Vec<String>,

    /// The new project path
    #[clap(short, long, parse(from_os_str), default_value = ".")]
    path: std::path::PathBuf,
}

#[async_trait]
impl CliCommand for New {
    async fn run(&self) -> Result<()> {
        log::debug!("topics {:?}", self.topics);

        let template = if self.topics.len() == 1 && GIT_REPO_REGEX.is_match(self.topics[0].as_str())
        {
            self.topics[0].clone()
        } else {
            let repos = list_templates(&self.topics).await?;
            if repos.is_empty() {
                return Err(Error::NoTemplateFound);
            } else if repos.len() == 1 {
                repos[0].url.clone()
            } else {
                choose_repository(repos).await?.url
            }
        };

        clone_template(template.clone(), self.path.clone()).await?;

        // create `.template` file to save template repo url and commit
        let git_dir = self.path.join(".git");
        let commit = get_current_commit(Some(git_dir.clone())).await?;
        fs::write(
            self.path.join(TEMPLATE_DATA_FILE),
            format!("{}\n{}", template, commit),
        )
        .map_err(Error::from)?;

        log::debug!("move git directory");
        // create the `.lenra` cache directory
        let cache_dir = self.path.join(LENRA_CACHE_DIRECTORY);
        fs::create_dir_all(cache_dir.clone()).unwrap();
        fs::rename(git_dir, cache_dir.join(TEMPLATE_GIT_DIR))?;

        Ok(())
    }
}
