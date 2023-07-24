//! # new
//!
//! The new subcommand creates a new Lenra app project from a template

use async_trait::async_trait;
pub use clap::Args;

use crate::cli::CliCommand;
use crate::errors::{Error, Result};
use crate::{cli, git, lenra, template};

use super::CommandContext;

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
    async fn run(&self, _context: CommandContext) -> Result<()> {
        log::debug!("topics {:?}", self.topics);

        let template =
            if self.topics.len() == 1 && git::GIT_REPO_REGEX.is_match(self.topics[0].as_str()) {
                self.topics[0].clone()
            } else {
                let repos = template::list_templates(&self.topics).await?;
                if repos.is_empty() {
                    return Err(Error::NoTemplateFound);
                } else if repos.len() == 1 {
                    repos[0].url.clone()
                } else {
                    template::choose_repository(repos).await?.url
                }
            };

        println!("Using template: {}", template);
        cli::loader(
            "Creating new project...",
            "Project created",
            "Failed creating new project",
            || async { lenra::create_new_project(template.as_str(), &self.path).await },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, sync::Mutex};

    use mocktopus::mocking::{MockResult, Mockable};

    use super::*;
    use crate::{
        cli::{self, Command},
        git::Repository,
        template,
    };

    const NODE_TEMPLATE_HTTP_URL: &str = "https://github.com/lenra-io/template-javascript.git";
    const NODE_TEMPLATE_SSH_URL: &str = "git@github.com:lenra-io/template-javascript.git";
    const BUN_TEMPLATE_HTTP_URL: &str = "https://github.com/taorepoara/lenra-template-bun-js.git";

    fn mock_all() {
        template::list_templates.mock_safe(|_| unreachable!());
        template::choose_repository.mock_safe(|_| unreachable!());
        lenra::create_new_project.mock_safe(|_, _| unreachable!());
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
        let result = new
            .run(CommandContext {
                ..Default::default()
            })
            .await;
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
        let create_new_project_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_new_project_call_counter);
        lenra::create_new_project.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let result = new
            .run(CommandContext {
                ..Default::default()
            })
            .await;
        assert_eq!(*list_templates_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_new_project_call_counter.lock().unwrap(), 1);
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
        let create_new_project_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_new_project_call_counter);
        lenra::create_new_project.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let result = new
            .run(CommandContext {
                ..Default::default()
            })
            .await;
        assert_eq!(*list_templates_call_counter.lock().unwrap(), 1);
        assert_eq!(*create_new_project_call_counter.lock().unwrap(), 1);
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
        let create_new_project_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_new_project_call_counter);
        lenra::create_new_project.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let result = new
            .run(CommandContext {
                ..Default::default()
            })
            .await;
        assert_eq!(*create_new_project_call_counter.lock().unwrap(), 1);
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
        let create_new_project_call_counter = Rc::new(Mutex::new(0));
        let counter = Rc::clone(&create_new_project_call_counter);
        lenra::create_new_project.mock_safe(move |_, _| {
            let mut num = counter.lock().unwrap();
            *num += 1;
            MockResult::Return(Box::pin(async move { Ok(()) }))
        });
        let result = new
            .run(CommandContext {
                ..Default::default()
            })
            .await;
        assert_eq!(*create_new_project_call_counter.lock().unwrap(), 1);
        assert!(result.is_ok());
        Ok(())
    }
}
