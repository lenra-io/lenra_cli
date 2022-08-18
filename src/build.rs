use log;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap;
use dofigen_lib::{
    from_file_path, generate_dockerfile, generate_dockerignore, Artifact, Builder, Image,
};

use crate::cli::CliCommand;
use crate::config::{load_config_file, Generator, DEFAULT_CONFIG_FILE, LENRA_CACHE_DIRECTORY};

static OF_WATCHDOG_BUILDER: &str = "of-watchdog";
static OF_WATCHDOG_IMAGE: &str = "ghcr.io/openfaas/of-watchdog";
static OF_WATCHDOG_VERSION: &str = "0.9.6";

#[derive(clap::Args)]
pub struct Build {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// The app configuration file.
    #[clap(value_enum, long, default_value = "local")]
    pub cache: Cache,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Cache {
    Local,
    Inline,
    Image,
    No,
}

impl Build {
    /// Builds a Docker image from a Dofigen structure
    fn build_dofigen(&self, image: Image) {
        // Generate the Dofigen config with OpenFaaS overlay to handle the of-watchdog
        let of_overlay = self.dofigen_of_overlay(image);

        // generate the Dockerfile and .dockerignore files with Dofigen
        let dockerfile = generate_dockerfile(&of_overlay);
        let dockerignore = generate_dockerignore(&of_overlay);
        self.save_docker_content(dockerfile, Some(dockerignore));

        // build the generated Dockerfile
        self.build_docker_image(None);
    }

    /// Add an overlay to the given Dofigen structure to manage OpenFaaS
    fn dofigen_of_overlay(&self, image: Image) -> Image {
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
                panic!("More than one port has been defined in the Dofigen descriptor");
            }
        };

        if image.entrypoint.is_some() {
            panic!("The Dofigen descriptor can't have entrypoint defined. Use cmd instead");
        }

        if let Some(cmd) = image.cmd {
            envs.insert("fprocess".to_string(), cmd.join(" "));
        } else {
            panic!("The Dofigen cmd property is not defined");
        }

        Image {
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
        }
    }

    /// Saves the Dockerfile and dockerignore (if present) files from their contents
    fn save_docker_content(
        &self,
        dockerfile_content: String,
        dockerignore_content: Option<String>,
    ) {
        let dockerfile_path: PathBuf = [LENRA_CACHE_DIRECTORY, "Dockerfile"].iter().collect();
        let dockerignore_path: PathBuf = [LENRA_CACHE_DIRECTORY, ".dockerignore"].iter().collect();

        fs::write(dockerfile_path, dockerfile_content).expect("Unable to write the Dockerfile");
        if let Some(content) = dockerignore_content {
            fs::write(dockerignore_path, content).expect("Unable to write the .dockerignore file");
        }
    }

    /// Builds a Dockerfile. If None, get's it at the default path: ./.lenra/Dockerfile
    fn build_docker_image(&self, dockerfile: Option<PathBuf>) {
        log::info!("Build the Docker image");
        let dockerfile_path: PathBuf =
            dockerfile.unwrap_or([LENRA_CACHE_DIRECTORY, "Dockerfile"].iter().collect());
        let cache_directory: PathBuf = [LENRA_CACHE_DIRECTORY, "dockercache"].iter().collect();
        let mut command = Command::new("docker");
        let image_name = "lenra/app";

        // TODO: display std out & err
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        command
            .arg("buildx")
            .arg("build")
            .arg("-f")
            .arg(dockerfile_path);

        match self.cache {
            Cache::Inline => command
                .arg("--cache-to=type=inline")
                .arg(format!("--cache-from={}", image_name)),
            Cache::Local => command
                .arg(format!(
                    "--cache-to=type=local,dest={}",
                    cache_directory.display()
                ))
                .arg(format!(
                    "--cache-from=type=local,src={}",
                    cache_directory.display()
                )),
            Cache::Image => command
                .arg(format!("--cache-to={}:cache", image_name))
                .arg(format!("--cache-from={}:cache", image_name)),
            Cache::No => &command,
        };
        command.arg("-t").arg(image_name).arg("--load").arg(".");

        log::debug!("Build image: {:?}", command);
        let output = command.output().expect("Failed building the Docker image");
        if !output.status.success() {
            panic!(
                "An error occured while building the Docker image:\n{}\n{}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
        log::info!("Image built");
    }
}

impl CliCommand for Build {
    fn run(&self) {
        let conf = load_config_file(&self.config);
        // TODO: check the components API version

        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();

        match conf.generator {
            Generator::Dofigen(dofigen) => self.build_dofigen(dofigen.dofigen),
            Generator::DofigenFile(dofigen_file) => self.build_dofigen(
                from_file_path(&dofigen_file.dofigen).expect("Failed loading the Dofigen file"),
            ),
            Generator::DofigenError { dofigen: _ } => {
                panic!("Your Dofigen configuration is not correct")
            }
            Generator::Dockerfile(dockerfile) => self.build_docker_image(Some(dockerfile.docker)),
            Generator::Docker(docker) => {
                self.save_docker_content(docker.docker, docker.ignore);
                self.build_docker_image(None);
            }
        }
    }
}

#[cfg(test)]
mod dofigen_of_overlay_tests {
    use super::*;

    #[test]
    fn simple_image() {
        let build = Build {
            config: DEFAULT_CONFIG_FILE.into(),
            cache: Cache::Inline,
        };
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
            envs: Some([("fprocess".to_string(), "/app/my-app".to_string())].into()),
            artifacts: Some(vec![Artifact {
                builder: "of-watchdog".into(),
                source: "/fwatchdog".into(),
                destination: "/fwatchdog".into(),
            }]),
            ports: Some(vec![8080]),
            cmd: Some(vec!["/fwatchdog".into()]),
            ..Default::default()
        };

        assert_eq!(build.dofigen_of_overlay(image), overlayed_image);
    }

    #[test]
    #[should_panic]
    fn no_cmd() {
        let build = Build {
            config: DEFAULT_CONFIG_FILE.into(),
            cache: Cache::Inline,
        };
        let image = Image {
            image: "my-dockerimage".into(),
            ..Default::default()
        };
        build.dofigen_of_overlay(image);
    }
}
