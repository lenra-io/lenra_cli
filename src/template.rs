use std::{fs, path::PathBuf};

use crate::errors::{Error, Result};
use lazy_static::lazy_static;
use log;
use regex::Regex;
use tokio::process::Command;

pub const TEMPLATE_DATA_FILE: &str = ".template";
pub const TEMPLATE_GIT_DIR: &str = "template.git";
pub const TEMPLATE_TEMP_DIR: &str = "template.tmp";

lazy_static! {
    pub static ref TEMPLATE_SHORT_REGEX: Regex =
        Regex::new(r"^(template-)?([0-9a-zA-Z]+([_-][0-9a-zA-Z]+)*)$").unwrap();
}

pub struct TemplateData {
    pub template: String,
    pub commit: Option<String>,
}

pub fn normalize_template(template: String) -> String {
    if TEMPLATE_SHORT_REGEX.is_match(template.as_str()) {
        format!(
            "https://github.com/lenra-io/template-{}",
            TEMPLATE_SHORT_REGEX.replace(template.as_str(), "$2")
        )
    } else {
        template.clone()
    }
}

pub async fn clone_template(template: String, target_dir: PathBuf) -> Result<()> {
    log::debug!(
        "clone the template {} into {}",
        template,
        target_dir.display()
    );
    Command::new("git")
        .kill_on_drop(true)
        .arg("clone")
        .arg("--single-branch")
        .arg("--depth")
        .arg("1")
        .arg(template)
        .arg(target_dir.as_os_str())
        .spawn()?
        .wait_with_output()
        .await
        .map_err(Error::from)?;

    Ok(())
}

pub async fn get_template_data() -> Result<TemplateData> {
    // TODO: if template data file exists
    let template_data = fs::read_to_string(TEMPLATE_DATA_FILE).map_err(Error::from)?;
    let template_data: Vec<&str> = template_data.split("\n").collect();

    Ok(TemplateData {
        template: template_data[0].into(),
        commit: Some(template_data[1].into()),
    })
    
    // TODO: else ask user the template
}
