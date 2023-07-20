//! # new
//!
//! The new subcommand creates a new Lenra app project from a template

use async_trait::async_trait;
pub use clap::Args;
use log;
use std::fs;
use std::path::PathBuf;

use crate::cli::CliCommand;
use crate::config::LENRA_CACHE_DIRECTORY;
use crate::errors::{Error, Result};
use crate::git::{get_current_commit, GIT_REPO_REGEX};
use crate::template::{
    choose_repository, clone_template, list_templates, TemplateData, TEMPLATE_DATA_FILE,
    TEMPLATE_GIT_DIR,
};
#[cfg(test)]
use mocktopus::macros::mockable;

#[derive(Args, Debug, Clone)]
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
        TemplateData {
            template,
            commit: Some(commit.clone()),
        }
        .save_to(&self.path.join(TEMPLATE_DATA_FILE))
        .await
        .map_err(Error::from)?;

        create_cache_directories(&self.path, &git_dir)?;

        Ok(())
    }
}

#[cfg_attr(test, mockable)]
fn create_cache_directories(path: &PathBuf, git_dir: &PathBuf) -> Result<()> {
    log::debug!("create cache directories");
    // create the `.lenra` cache directory
    let cache_dir = path.join(LENRA_CACHE_DIRECTORY);
    fs::create_dir_all(cache_dir.clone()).unwrap();
    // move the template `.git` directory
    fs::rename(git_dir, cache_dir.join(TEMPLATE_GIT_DIR))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, sync::Mutex};

    use mocktopus::mocking::{MockResult, Mockable};

    use super::*;
    use crate::{
        cli::{self, Command},
        git::{self, Repository},
        template,
    };

    const NODE_TEMPLATE_HTTP_URL: &str = "https://github.com/lenra-io/template-javascript.git";
    const NODE_TEMPLATE_SSH_URL: &str = "git@github.com:lenra-io/template-javascript.git";
    const BUN_TEMPLATE_HTTP_URL: &str = "https://github.com/taorepoara/lenra-template-bun-js.git";
    const COMMIT_NUMBER: &str = "398416b76fc237291280de3efb719504c43cf563";

    fn mock_all() {
        template::list_templates.mock_safe(|_| unreachable!());
        template::clone_template.mock_safe(|_, _| unreachable!());
        template::choose_repository.mock_safe(|_| unreachable!());
        git::get_current_commit.mock_safe(|_| unreachable!());
        TemplateData::save_to.mock_safe(|_, _| unreachable!());
        create_cache_directories.mock_safe(|_, _| unreachable!());
    }

    #[tokio::test]
    async fn no_matching_template() -> Result<(), Box<dyn std::error::Error>> {
        let cli = cli::test::parse_command_line(String::from("lenra new js"))?;
        let command = cli.command;
        let new = match command {
            Command::New(new) => new,
            _ => panic!("wrong command"),
        };
        let expected_topics = vec!["js".to_string()];

        assert_eq!(new.path, std::path::PathBuf::from("."));
        assert_eq!(new.topics, expected_topics);
        mock_all();
        let list_templates_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&list_templates_call_counter);
        template::list_templates.mock_safe(move |topics| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            println!("called {} times", *num);
            assert_eq!(topics, &expected_topics);
            MockResult::Return(Box::pin(async move { Ok(vec![]) }))
        });
        let result = new.run().await;
        let call_count = *list_templates_call_counter.lock().unwrap();
        println!("called {} times", call_count);
        assert_eq!(call_count, 1);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            Error::NoTemplateFound => (),
            er => panic!("wrong error type {er}"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn one_matching_template() -> Result<(), Box<dyn std::error::Error>> {
        let cli = cli::test::parse_command_line(String::from("lenra new js"))?;
        let command = cli.command;
        let new = match command {
            Command::New(new) => new,
            _ => panic!("wrong command"),
        };
        let expected_topics = vec!["js".to_string()];

        assert_eq!(new.path, std::path::PathBuf::from("."));
        assert_eq!(new.topics, expected_topics);
        mock_all();
        let list_templates_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&list_templates_call_counter);
        template::list_templates.mock_safe(move |topics| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(topics, &expected_topics);
            MockResult::Return(Box::pin(async move {
                Ok(vec![Repository {
                    name: "template-javascript".to_string(),
                    description: "Javascript template".to_string(),
                    url: NODE_TEMPLATE_HTTP_URL.into(),
                    stars: 0,
                }])
            }))
        });
        let clone_template_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&clone_template_call_counter);
        template::clone_template.mock_safe(move |url, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(url, NODE_TEMPLATE_HTTP_URL);
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let get_current_commit_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&get_current_commit_call_counter);
        git::get_current_commit.mock_safe(move |_| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(COMMIT_NUMBER.to_string()) }))
        });
        let save_to_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&save_to_call_counter);
        TemplateData::save_to.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let create_cache_directories_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_cache_directories_call_counter);
        create_cache_directories.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Ok(()))
        });
        let result = new.run().await;
        assert_eq!(*list_templates_call_counter.lock().unwrap(), 1);
        assert_eq!(*clone_template_call_counter.lock().unwrap(), 1);
        assert_eq!(*get_current_commit_call_counter.lock().unwrap(), 1);
        assert_eq!(*save_to_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_cache_directories_call_counter.lock().unwrap(), 1);
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn two_matching_templates() -> Result<(), Box<dyn std::error::Error>> {
        let cli = cli::test::parse_command_line(String::from("lenra new js"))?;
        let command = cli.command;
        let new = match command {
            Command::New(new) => new,
            _ => panic!("wrong command"),
        };
        let expected_topics = vec!["js".to_string()];

        assert_eq!(new.path, std::path::PathBuf::from("."));
        assert_eq!(new.topics, expected_topics);
        mock_all();
        let list_templates_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&list_templates_call_counter);
        template::list_templates.mock_safe(move |topics| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(topics, &expected_topics);
            MockResult::Return(Box::pin(async move {
                Ok(vec![
                    Repository {
                        name: "template-javascript".to_string(),
                        description: "Javascript template".to_string(),
                        url: NODE_TEMPLATE_HTTP_URL.into(),
                        stars: 1,
                    },
                    Repository {
                        name: "template-bun".to_string(),
                        description: "Javascript template with Bun.sh".to_string(),
                        url: BUN_TEMPLATE_HTTP_URL.into(),
                        stars: 0,
                    },
                ])
            }))
        });
        let choose_repository_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&choose_repository_call_counter);
        template::choose_repository.mock_safe(move |repos| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(repos.len(), 2);
            MockResult::Return(Box::pin(async move { Ok(repos[0].clone()) }))
        });
        let clone_template_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&clone_template_call_counter);
        template::clone_template.mock_safe(move |url, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(url, NODE_TEMPLATE_HTTP_URL);
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let get_current_commit_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&get_current_commit_call_counter);
        git::get_current_commit.mock_safe(move |_| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(COMMIT_NUMBER.to_string()) }))
        });
        let save_to_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&save_to_call_counter);
        TemplateData::save_to.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let create_cache_directories_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_cache_directories_call_counter);
        create_cache_directories.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Ok(()))
        });
        let result = new.run().await;
        assert_eq!(*list_templates_call_counter.lock().unwrap(), 1);
        assert_eq!(*clone_template_call_counter.lock().unwrap(), 1);
        assert_eq!(*get_current_commit_call_counter.lock().unwrap(), 1);
        assert_eq!(*save_to_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_cache_directories_call_counter.lock().unwrap(), 1);
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn from_http_url() -> Result<(), Box<dyn std::error::Error>> {
        let cli = cli::test::parse_command_line(format!("lenra new {}", NODE_TEMPLATE_HTTP_URL))?;
        let command = cli.command;
        let new = match command {
            Command::New(new) => new,
            _ => panic!("wrong command"),
        };
        let expected_topics = vec![NODE_TEMPLATE_HTTP_URL.to_string()];

        assert_eq!(new.path, std::path::PathBuf::from("."));
        assert_eq!(new.topics, expected_topics);
        mock_all();
        let clone_template_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&clone_template_call_counter);
        template::clone_template.mock_safe(move |url, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(url, NODE_TEMPLATE_HTTP_URL);
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let get_current_commit_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&get_current_commit_call_counter);
        git::get_current_commit.mock_safe(move |_| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(COMMIT_NUMBER.to_string()) }))
        });
        let save_to_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&save_to_call_counter);
        TemplateData::save_to.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let create_cache_directories_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_cache_directories_call_counter);
        create_cache_directories.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Ok(()))
        });
        let result = new.run().await;
        assert_eq!(*clone_template_call_counter.lock().unwrap(), 1);
        assert_eq!(*get_current_commit_call_counter.lock().unwrap(), 1);
        assert_eq!(*save_to_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_cache_directories_call_counter.lock().unwrap(), 1);
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn from_ssh_url() -> Result<(), Box<dyn std::error::Error>> {
        let cli = cli::test::parse_command_line(format!("lenra new {}", NODE_TEMPLATE_SSH_URL))?;
        let command = cli.command;
        let new = match command {
            Command::New(new) => new,
            _ => panic!("wrong command"),
        };
        let expected_topics = vec![NODE_TEMPLATE_SSH_URL.to_string()];

        assert_eq!(new.path, std::path::PathBuf::from("."));
        assert_eq!(new.topics, expected_topics);
        mock_all();
        let clone_template_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&clone_template_call_counter);
        template::clone_template.mock_safe(move |url, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            assert_eq!(url, NODE_TEMPLATE_SSH_URL);
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let get_current_commit_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&get_current_commit_call_counter);
        git::get_current_commit.mock_safe(move |_| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(COMMIT_NUMBER.to_string()) }))
        });
        let save_to_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&save_to_call_counter);
        TemplateData::save_to.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let create_cache_directories_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_cache_directories_call_counter);
        create_cache_directories.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Ok(()))
        });
        let result = new.run().await;
        assert_eq!(*clone_template_call_counter.lock().unwrap(), 1);
        assert_eq!(*get_current_commit_call_counter.lock().unwrap(), 1);
        assert_eq!(*save_to_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_cache_directories_call_counter.lock().unwrap(), 1);
        assert!(result.is_ok());
        Ok(())
    }
}
