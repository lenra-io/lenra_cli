use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    command::{get_command_output, run_command},
    config::LENRA_CACHE_DIRECTORY,
    errors::{Error, Result},
    git::{create_git_command, Repository},
    github::{search_repositories, GITHUB_TOPIC_REGEX},
};
use lazy_static::lazy_static;
use log;
use regex::Regex;
use rustyline::Editor;

#[cfg(test)]
use mocktopus::macros::mockable;

pub const TEMPLATE_DATA_FILE: &str = ".template";
pub const TEMPLATE_GIT_DIR: &str = "template.git";
pub const TEMPLATE_TEMP_DIR: &str = "template.tmp";
pub const RL_CHOOSE_TEMPLATE_MSG: &str = "Which template do you want to use ? ";

lazy_static! {
    static ref TEMPLATE_ALIASES: HashMap<&'static str, &'static str> = vec![
        ("js", "javascript"),
        ("ts", "typescript"),
        ("rs", "rust"),
        ("py", "python"),
    ]
    .into_iter()
    .collect();
}

lazy_static! {
    pub static ref TEMPLATE_SHORT_REGEX: Regex =
        Regex::new(r"^(template-)?([0-9a-zA-Z]+([_-][0-9a-zA-Z]+)*)$").unwrap();
}

pub struct TemplateData {
    pub template: String,
    pub commit: Option<String>,
}

#[cfg_attr(test, mockable)]
impl TemplateData {
    pub async fn save(&self) -> Result<()> {
        let path = Path::new(TEMPLATE_DATA_FILE);
        self.save_to(&path).await
    }

    pub async fn save_to(&self, path: &Path) -> Result<()> {
        let commit = self.commit.clone().unwrap();
        log::debug!("save template data {}:{}", self.template, commit);
        fs::write(path, format!("{}\n{}", self.template, commit)).map_err(Error::from)
    }
}

pub fn normalize_template(template: String) -> String {
    if TEMPLATE_SHORT_REGEX.is_match(template.as_str()) {
        // Replace aliases
        let &name = TEMPLATE_ALIASES
            .get(template.as_str())
            .unwrap_or(&template.as_str());

        format!(
            "https://github.com/lenra-io/template-{}",
            TEMPLATE_SHORT_REGEX.replace(name, "$2")
        )
    } else {
        template.clone()
    }
}

#[cfg_attr(test, mockable)]
pub async fn list_templates(topics: &Vec<String>) -> Result<Vec<Repository>> {
    let mut query: String = String::from("topic:lenra+topic:template");
    for topic in topics {
        // check topic format
        if !GITHUB_TOPIC_REGEX.is_match(topic.as_str()) {
            return Err(Error::InvalidGitHubTopic(topic.clone()));
        }
        query.push_str(format!("+topic:{}", topic).as_str());
    }
    search_repositories(query.as_str()).await
}

#[cfg_attr(test, mockable)]
pub async fn choose_repository(repos: Vec<Repository>) -> Result<Repository> {
    let mut rl = Editor::<()>::new()?;
    let mut index = 0;
    let mut max_index = 0;
    for repo in &repos {
        println!(
            "{:5} {} ({} stars) - {}",
            format!("[{}]:", index + 1),
            repo.name,
            repo.stars,
            repo.description
        );
        index += 1;
        max_index = index;
    }
    let mut choice = rl.readline(RL_CHOOSE_TEMPLATE_MSG)?;
    while choice.parse::<usize>().is_err()
        || choice.parse::<usize>().unwrap() < 1
        || choice.parse::<usize>().unwrap() > max_index
    {
        choice = rl.readline(RL_CHOOSE_TEMPLATE_MSG)?;
    }
    Ok(repos[choice.parse::<usize>().unwrap() - 1].clone())
}

#[cfg_attr(test, mockable)]
pub async fn clone_template(template: &str, target_dir: &PathBuf) -> Result<()> {
    log::debug!(
        "clone the template {} into {}",
        template,
        target_dir.display()
    );
    let mut cmd = create_git_command();
    cmd.arg("clone").arg(template).arg(target_dir.as_os_str());
    run_command(&mut cmd, None).await?;

    Ok(())
}

#[cfg_attr(test, mockable)]
pub async fn get_template_data() -> Result<TemplateData> {
    let template_data_file = Path::new(TEMPLATE_DATA_FILE);
    let git_dir = Path::new(LENRA_CACHE_DIRECTORY).join(TEMPLATE_GIT_DIR);
    if template_data_file.exists() {
        let template_data = fs::read_to_string(template_data_file).map_err(Error::from)?;
        let template_data: Vec<&str> = template_data.split("\n").collect();
        Ok(TemplateData {
            template: template_data[0].into(),
            commit: Some(template_data[1].into()),
        })
    } else if git_dir.exists() {
        let mut cmd = create_git_command();
        cmd.arg("--git-dir")
            .arg(git_dir.as_os_str())
            .arg("config")
            .arg("--get")
            .arg("remote.origin.url");
        let template = get_command_output(&mut cmd).await?;
        Ok(TemplateData {
            template,
            commit: None,
        })
    } else {
        let mut rl = Editor::<()>::new()?;
        println!("The '.template' file does not exist.");
        let template = normalize_template(rl.readline("What is the template to use ? ")?);
        Ok(TemplateData {
            template,
            commit: None,
        })
    }
}
