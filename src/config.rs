use std::fs;

use dofigen_lib::Image;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

pub static DEFAULT_CONFIG_FILE: &str = "lenra.yml";
pub static LENRA_CACHE_DIRECTORY: &str = ".lenra";

pub fn load_config_file(path: &std::path::PathBuf) -> Application {
    let file = fs::File::open(path).unwrap();
    match path.extension() {
        Some(os_str) => match os_str.to_str() {
            Some("yml" | "yaml") => serde_yaml::from_reader(file).unwrap(),
            Some("json") => serde_json::from_reader(file).unwrap(),
            Some(ext) => panic!("Not managed config file extension {}", ext),
            None => panic!("The config file has no extension"),
        },
        None => panic!("The config file has no extension"),
    }
}

/** The main component of the config file */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Application {
    #[serde(rename = "componentsApi")]
    pub components_api: String,
    pub generator: Generator,
}

/** Represents the Dockerfile main stage */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Generator {
    Dofigen(Dofigen),
    DofigenFile(DofigenFile),
    DofigenError { dofigen: Value },
}

/** The application generator configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Dofigen {
    pub dofigen: Image,
}

/** The application generator configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct DofigenFile {
    pub dofigen: std::path::PathBuf,
}
