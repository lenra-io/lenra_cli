use std::{collections::HashMap, fmt::Debug, fs, path::PathBuf};

use dofigen_lib::{
    self, from_file_path, generate_dockerfile, generate_dockerignore, Artifact, Builder,
    Healthcheck,
};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::{
    docker_compose::{generate_docker_compose, Service},
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
    let file = fs::File::open(path).map_err(|err| Error::OpenFile(err, path.clone()))?;
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Dev {
    pub app: Option<Image>,
    pub devtool: Option<Image>,
    pub postgres: Option<Image>,
    pub mongo: Option<Image>,
    pub dofigen: Option<DebugDofigen>,
}

/** A Docker image */
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Image {
    pub image: Option<String>,
    pub tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DebugDofigen {
    pub cmd: Option<Vec<String>>,
    pub ports: Option<Vec<u16>>,
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
    pub dofigen: dofigen_lib::Image,
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
    pub async fn generate_files(&self, exposed_services: &Vec<Service>, debug: bool) -> Result<()> {
        self.generate_docker_files(debug)?;
        self.generate_docker_compose_file(exposed_services, debug)
            .await?;
        Ok(())
    }

    pub fn generate_docker_files(&self, debug: bool) -> Result<()> {
        log::info!("Docker files generation");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();

        match &self.generator {
            // If args '--prod' is passed then not debug
            Generator::Dofigen(dofigen) => self.build_dofigen(dofigen.dofigen.clone(), debug),
            Generator::DofigenFile(dofigen_file) => self.build_dofigen(
                from_file_path(&dofigen_file.dofigen).map_err(Error::from)?,
                debug,
            ),
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

    pub async fn generate_docker_compose_file(
        &self,
        exposed_services: &Vec<Service>,
        debug: bool,
    ) -> Result<()> {
        log::info!("Docker Compose file generation");
        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).map_err(Error::from)?;

        let dockerfile: PathBuf = if let Generator::Dockerfile(file_conf) = &self.generator {
            file_conf.docker.clone()
        } else {
            DOCKERFILE_DEFAULT_PATH.iter().collect()
        };

        generate_docker_compose(dockerfile, &self.dev, exposed_services, debug)
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Builds a Docker image from a Dofigen structure
    fn build_dofigen(&self, image: dofigen_lib::Image, debug: bool) -> Result<()> {
        // Generate the Dofigen config with OpenFaaS overlay to handle the of-watchdog
        let overlay = self.dofigen_of_overlay(image)?;

        // when debug add cmd and ports to the Dofigen descriptor
        let overlay = if debug {
            self.dofigen_debug_overlay(overlay)?
        } else {
            overlay
        };

        // generate the Dockerfile and .dockerignore files with Dofigen
        let dockerfile = generate_dockerfile(&overlay);
        let dockerignore = generate_dockerignore(&overlay);
        self.save_docker_content(dockerfile, Some(dockerignore))
    }

    fn dofigen_debug_overlay(&self, image: dofigen_lib::Image) -> Result<dofigen_lib::Image> {
        log::info!("Adding debug overlay to the Dofigen descriptor");
        let mut debug_overlay = image;
        if let Some(dev) = &self.dev {
            if let Some(dofigen) = &dev.dofigen {
                if let Some(cmd) = &dofigen.cmd {
                    let mut envs = debug_overlay.envs.unwrap();
                    envs.insert("fprocess".to_string(), cmd.join(" "));
                    debug_overlay.envs = Some(envs);
                }
                if let Some(ports) = &dofigen.ports {
                    debug_overlay.ports = Some(
                        debug_overlay
                            .ports
                            .unwrap()
                            .into_iter()
                            .chain(ports.into_iter().map(|&value| value))
                            .collect(),
                    )
                }
            }
        }
        Ok(debug_overlay)
    }

    /// Add an overlay to the given Dofigen structure to manage OpenFaaS
    fn dofigen_of_overlay(&self, image: dofigen_lib::Image) -> Result<dofigen_lib::Image> {
        log::info!("Adding OpenFaaS overlay to the Dofigen descriptor");
        let mut builders = if let Some(vec) = image.builders {
            vec
        } else {
            Vec::new()
        };
        // add of-watchdog builder
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
        // get of-watchdog artifact
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

        let mut healthcheck = None;
        // http mode (not if empty)
        if let Some(ports) = image.ports {
            if ports.len() > 1 {
                return Err(Error::Custom(
                    "More than one port has been defined in the Dofigen descriptor".into(),
                ));
            }
            if ports.len() == 1 {
                envs.insert("mode".to_string(), "http".to_string());
                envs.insert(
                    "upstream_url".to_string(),
                    format!("http://127.0.0.1:{}", ports[0]),
                );
                envs.insert("suppress_lock".to_string(), "true".to_string());
                if !envs.contains_key("exec_timeout") {
                    envs.insert("exec_timeout".to_string(), "3600".to_string());
                }
                if !envs.contains_key("read_timeout") {
                    envs.insert("read_timeout".to_string(), "3600".to_string());
                }
                if !envs.contains_key("write_timeout") {
                    envs.insert("write_timeout".to_string(), "3600".to_string());
                }
                // handle healthcheck
                healthcheck = Some(Healthcheck {
                    cmd: "curl --fail http://localhost:8080/_/health".into(),
                    start: Some("3s".into()),
                    interval: Some("3s".into()),
                    timeout: Some("1s".into()),
                    retries: Some(10),
                });
            }
        };

        // prevent custom entrypoint
        if image.entrypoint.is_some() {
            return Err(Error::Custom(
                "The Dofigen descriptor can't have entrypoint defined. Use cmd instead".into(),
            ));
        }

        // envs.insert("exec_timeout".to_string(), "0".to_string());

        if let Some(cmd) = image.cmd {
            envs.insert("fprocess".to_string(), cmd.join(" "));
        } else {
            return Err(Error::Custom(
                "The Dofigen cmd property is not defined".into(),
            ));
        }

        Ok(dofigen_lib::Image {
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
            healthcheck: healthcheck,
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
        let image = dofigen_lib::Image {
            image: "my-dockerimage".into(),
            cmd: Some(vec!["/app/my-app".into()]),
            ..Default::default()
        };
        let overlayed_image = dofigen_lib::Image {
            builders: Some(vec![Builder {
                name: Some("of-watchdog".into()),
                image: format!("ghcr.io/openfaas/of-watchdog:{}", OF_WATCHDOG_VERSION),
                ..Default::default()
            }]),
            image: String::from("my-dockerimage"),
            envs: Some(
                [
                    // ("exec_timeout".to_string(), "0".to_string()),
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
        let image = dofigen_lib::Image {
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

impl Image {
    pub fn to_image(&self, default_image: &str, default_tag: &str) -> String {
        format!(
            "{}:{}",
            self.image.clone().unwrap_or(default_image.to_string()),
            self.tag.clone().unwrap_or(default_tag.to_string())
        )
    }
}
