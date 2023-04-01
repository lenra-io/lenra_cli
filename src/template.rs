use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    errors::{Error, Result},
    git::create_git_command,
};
use lazy_static::lazy_static;
use log;
use regex::Regex;
use rustyline::Editor;

pub const TEMPLATE_DATA_FILE: &str = ".template";
pub const TEMPLATE_GIT_DIR: &str = "template.git";
pub const TEMPLATE_TEMP_DIR: &str = "template.tmp";

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

pub async fn clone_template(template: String, target_dir: PathBuf) -> Result<()> {
    log::debug!(
        "clone the template {} into {}",
        template,
        target_dir.display()
    );
    create_git_command()
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
    let template_data_file = Path::new(TEMPLATE_DATA_FILE);
    if template_data_file.exists() {
        let template_data = fs::read_to_string(template_data_file).map_err(Error::from)?;
        let template_data: Vec<&str> = template_data.split("\n").collect();
        Ok(TemplateData {
            template: template_data[0].into(),
            commit: Some(template_data[1].into()),
        })
    } else {
        let mut rl = Editor::<()>::new()?;
        println!("The '.template' file does not exist.");
        let template = normalize_template(rl.readline("What is the template to use ? ")?);
        Ok(TemplateData {
            template: template,
            commit: None,
        })
    }
}
