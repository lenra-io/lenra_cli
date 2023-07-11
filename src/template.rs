use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    command::get_command_output,
    config::LENRA_CACHE_DIRECTORY,
    errors::{Error, Result},
    git::create_git_command,
};
use lazy_static::lazy_static;
use log;
use regex::Regex;
use rustyline::Editor;
use serde::{Deserialize, Serialize};

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

trait PlatformRepository {
    fn to_repository(&self) -> Repository;
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubSearchRepoResponse {
    pub total_count: u32,
    pub items: Vec<GitHubRepository>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubRepository {
    pub full_name: String,
    pub description: String,
    pub clone_url: String,
    pub stargazers_count: u32,
}

impl PlatformRepository for GitHubRepository {
    fn to_repository(&self) -> Repository {
        Repository {
            name: self.full_name.clone(),
            description: self.description.clone(),
            url: self.clone_url.clone(),
            stars: self.stargazers_count,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Repository {
    pub name: String,
    pub description: String,
    pub url: String,
    pub stars: u32,
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

pub async fn list_templates(topics: Vec<String>) -> Result<Vec<Repository>> {
    let mut request: String = String::from(
        "https://api.github.com/search/repositories?sort=stargazers&q=topic:lenra+topic:template",
    );
    for topic in topics {
        // TODO: check topic format
        request.push_str(format!("+topic:{}", topic).as_str());
    }
    let reponse: GitHubSearchRepoResponse = ureq::get(request.as_str())
        .call()
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)?;

    // TODO: return a list of repos (generic in order to add GitLab, Bitbucket, etc.)
    reponse
        .items
        .into_iter()
        .map(|repo| Ok(repo.to_repository()))
        .collect()
}

pub async fn choose_repository(repos: Vec<Repository>) -> Result<Repository> {
    if repos.len() == 1 {
        return Ok(repos[0].clone());
    }
    let mut rl = Editor::<()>::new()?;
    let mut index = 0;
    let mut max_index = 0;
    for repo in &repos {
        println!(
            "{}: {} ({} stars) - {}",
            index + 1,
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

pub async fn clone_template(template: String, target_dir: PathBuf) -> Result<()> {
    log::debug!(
        "clone the template {} into {}",
        template,
        target_dir.display()
    );
    create_git_command()
        .arg("clone")
        .arg(template)
        .arg(target_dir.as_os_str())
        .spawn()?
        .wait_with_output()
        .await
        .map_err(Error::from)?;

    Ok(())
}

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
        let template = get_command_output(cmd).await?;
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

pub async fn save_template_data(template_data: TemplateData) -> Result<()> {
    let template_data_file = Path::new(TEMPLATE_DATA_FILE);
    fs::write(
        template_data_file,
        format!(
            "{}\n{}",
            template_data.template,
            template_data.commit.unwrap()
        ),
    )
    .map_err(Error::from)
}
