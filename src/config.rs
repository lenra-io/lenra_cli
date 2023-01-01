use std::{collections::HashMap, fs, path::PathBuf};

use dofigen_lib::{
    from_file_path, generate_dockerfile, generate_dockerignore, Artifact, Builder, Image,
};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::{
    docker_compose::generate_docker_compose,
    errors::{Error, Result},
};

pub const DEFAULT_CONFIG_FILE: &str = "lenra.yml";
pub const LENRA_CACHE_DIRECTORY: &str = ".lenra";

pub const DOCKERFILE_DEFAULT_PATH: [&str; 2] = [LENRA_CACHE_DIRECTORY, "Dockerfile"];
pub const DOCKERIGNORE_DEFAULT_PATH: [&str; 2] = [LENRA_CACHE_DIRECTORY, "Dockerfile.dockerignore"];
pub const DOCKERCOMPOSE_DEFAULT_PATH: [&str; 2] = [LENRA_CACHE_DIRECTORY, "docker-compose.yml"];

pub const OF_WATCHDOG_BUILDER: &str = "of-watchdog";
pub const OF_WATCHDOG_IMAGE: &str = "ghcr.io/openfaas/of-watchdog";
pub const OF_WATCHDOG_VERSION: &str = "0.9.10";

pub fn load_config_file(path: &std::path::PathBuf) -> Result<Application> {
    let file = fs::File::open(path).map_err(Error::from)?;
    match path.extension() {
        Some(os_str) => match os_str.to_str() {
            Some("yml" | "yaml" | "json") => {
                Ok(serde_yaml::from_reader(file).map_err(Error::from)?)
            }
            Some(ext) => Err(Error::Custom(format!(
                "Not managed config file extension {}",
                ext
            ))),
            None => Err(Error::Custom(
                "The config file has no extension".to_string(),
            )),
        },
        None => Err(Error::Custom(
            "The config file has no extension".to_string(),
        )),
    }
}

/** The main component of the config file */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    #[serde(rename = "componentsApi")]
    pub components_api: String,
    pub generator: Generator,
    pub dev: Option<Dev>,
}

/** The dev specific configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Dev {
    pub app_name: Option<String>,
    pub app_tag: Option<String>,
    pub devtool_tag: Option<String>,
    pub postgres_tag: Option<String>,
    pub mongo_tag: Option<String>,
}

/** The application generator configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Generator {
    Dofigen(Dofigen),
    DofigenFile(DofigenFile),
    DofigenError { dofigen: Value },
    Dockerfile(Dockerfile),
    Docker(Docker),
    Unknow,
}

/** The Dofigen configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Dofigen {
    pub dofigen: Image,
}

/** The Dofigen configuration file */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct DofigenFile {
    pub dofigen: std::path::PathBuf,
}

/** The Docker configuration */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Docker {
    pub docker: String,
    pub ignore: Option<String>,
}

/** The Docker configuration file */
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Dockerfile {
    pub docker: std::path::PathBuf,
}

impl Application {
    /// Generates all the files needed to build and run the application
    pub async fn generate_files(&self, expose: bool) -> Result<()> {
        self.generate_docker_files()?;
        self.generate_docker_compose_file(expose).await?;
        Ok(())
    }

    pub fn generate_docker_files(&self) -> Result<()> {
        log::info!("Docker files generation");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();

        match &self.generator {
            Generator::Dofigen(dofigen) => self.build_dofigen(dofigen.dofigen.clone()),
            Generator::DofigenFile(dofigen_file) => {
                self.build_dofigen(from_file_path(&dofigen_file.dofigen).map_err(Error::from)?)
            }
            Generator::DofigenError { dofigen: _ } => Err(Error::Custom(
                "Your Dofigen configuration is not correct".into(),
            )),
            Generator::Dockerfile(_dockerfile) => Ok(()),
            Generator::Docker(docker) => {
                self.save_docker_content(docker.docker.clone(), docker.ignore.clone())
            }
            Generator::Unknow => Err(Error::Custom("Not managed generator".into())),
        }
    }

    pub async fn generate_docker_compose_file(&self, expose: bool) -> Result<()> {
        log::info!("Docker Compose file generation");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).map_err(Error::from)?;

        let dockerfile: PathBuf = if let Generator::Dockerfile(file_conf) = &self.generator {
            file_conf.docker.clone()
        } else {
            DOCKERFILE_DEFAULT_PATH.iter().collect()
        };

        generate_docker_compose(dockerfile, &self.dev, expose)
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Builds a Docker image from a Dofigen structure
    fn build_dofigen(&self, image: Image) -> Result<()> {
        // Generate the Dofigen config with OpenFaaS overlay to handle the of-watchdog
        let of_overlay = self.dofigen_of_overlay(image)?;

        // generate the Dockerfile and .dockerignore files with Dofigen
        let dockerfile = generate_dockerfile(&of_overlay);
        let dockerignore = generate_dockerignore(&of_overlay);
        self.save_docker_content(dockerfile, Some(dockerignore))
    }

    /// Add an overlay to the given Dofigen structure to manage OpenFaaS
    fn dofigen_of_overlay(&self, image: Image) -> Result<Image> {
        log::info!("Adding OpenFaaS overlay to the Dofigen descriptor");
        let mut builders = if let Some(vec) = image.builders {
            vec
        } else {
            Vec::new()
        };
        builders.push(Builder {
            name: Some(String::from(OF_WATCHDOG_BUILDER)),
            image: format!("{}:{}", OF_WATCHDOG_IMAGE, OF_WATCHDOG_VERSION),
            ..Default::default()
        });

        let mut artifacts = if let Some(arts) = image.artifacts {
            arts
        } else {
            Vec::new()
        };
        artifacts.push(Artifact {
            builder: OF_WATCHDOG_BUILDER.to_string(),
            source: "/fwatchdog".to_string(),
            destination: "/fwatchdog".to_string(),
        });

        let mut envs = if let Some(envs) = image.envs {
            envs
        } else {
            HashMap::new()
        };

        if let Some(ports) = image.ports {
            if ports.len() == 1 {
                envs.insert("mode".to_string(), "http".to_string());
                envs.insert(
                    "upstream_url".to_string(),
                    format!("http://127.0.0.1:{}", ports[0]),
                );
            } else if ports.len() > 1 {
                return Err(Error::Custom(
                    "More than one port has been defined in the Dofigen descriptor".into(),
                ));
            }
        };

        if image.entrypoint.is_some() {
            return Err(Error::Custom(
                "The Dofigen descriptor can't have entrypoint defined. Use cmd instead".into(),
            ));
        }

        envs.insert("exec_timeout".to_string(), "0".to_string());

        if let Some(cmd) = image.cmd {
            envs.insert("fprocess".to_string(), cmd.join(" "));
        } else {
            return Err(Error::Custom(
                "The Dofigen cmd property is not defined".into(),
            ));
        }

        Ok(Image {
            image: image.image,
            builders: Some(builders),
            artifacts: Some(artifacts),
            ports: Some(vec![8080]),
            envs: Some(envs),
            entrypoint: None,
            cmd: Some(vec!["/fwatchdog".to_string()]),
            user: image.user,
            workdir: image.workdir,
            adds: image.adds,
            root: image.root,
            script: image.script,
            caches: image.caches,
            healthcheck: image.healthcheck,
            ignores: image.ignores,
        })
    }

    /// Saves the Dockerfile and dockerignore (if present) files from their contents
    fn save_docker_content(
        &self,
        dockerfile_content: String,
        dockerignore_content: Option<String>,
    ) -> Result<()> {
        let dockerfile_path: PathBuf = DOCKERFILE_DEFAULT_PATH.iter().collect();
        let dockerignore_path: PathBuf = DOCKERIGNORE_DEFAULT_PATH.iter().collect();

        fs::write(dockerfile_path, dockerfile_content)?;
        if let Some(content) = dockerignore_content {
            fs::write(dockerignore_path, content)?;
        }
        Ok(())
    }
}

impl Default for Generator {
    fn default() -> Self {
        Generator::Unknow
    }
}

#[cfg(test)]
mod dofigen_of_overlay_tests {
    use super::*;

    #[test]
    fn simple_image() {
        let image = Image {
            image: "my-dockerimage".into(),
            cmd: Some(vec!["/app/my-app".into()]),
            ..Default::default()
        };
        let overlayed_image = Image {
            builders: Some(vec![Builder {
                name: Some("of-watchdog".into()),
                image: format!("ghcr.io/openfaas/of-watchdog:{}", OF_WATCHDOG_VERSION),
                ..Default::default()
            }]),
            image: String::from("my-dockerimage"),
            envs: Some(
                [
                    ("exec_timeout".to_string(), "0".to_string()),
                    ("fprocess".to_string(), "/app/my-app".to_string()),
                ]
                .into(),
            ),
            artifacts: Some(vec![Artifact {
                builder: "of-watchdog".into(),
                source: "/fwatchdog".into(),
                destination: "/fwatchdog".into(),
            }]),
            ports: Some(vec![8080]),
            cmd: Some(vec!["/fwatchdog".into()]),
            ..Default::default()
        };
        let config = Application {
            components_api: "".to_string(),
            generator: Generator::Dofigen(Dofigen {
                dofigen: image.clone(),
            }),
            ..Default::default()
        };

        assert_eq!(config.dofigen_of_overlay(image).unwrap(), overlayed_image);
    }

    #[test]
    #[should_panic]
    fn no_cmd() {
        let image = Image {
            image: "my-dockerimage".into(),
            ..Default::default()
        };
        let config = Application {
            components_api: "".to_string(),
            generator: Generator::Dofigen(Dofigen {
                dofigen: image.clone(),
            }),
            ..Default::default()
        };
        config.dofigen_of_overlay(image).unwrap();
    }
}
