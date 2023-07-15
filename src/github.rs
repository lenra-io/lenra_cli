use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    git::{PlatformRepository, Repository},
};

lazy_static! {
    pub static ref GITHUB_TOPIC_REGEX: Regex =
        Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
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

pub async fn search_repositories(query: &str) -> Result<Vec<Repository>> {
    let request = format!(
        "https://api.github.com/search/repositories?q={}&sort=stars&order=desc",
        query
    );
    log::debug!("search repositories on GitHub: {}", request);
    let reponse: GitHubSearchRepoResponse = ureq::get(request.as_str())
        .call()
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)?;

    reponse
        .items
        .into_iter()
        .map(|repo| Ok(repo.to_repository()))
        .collect()
}
