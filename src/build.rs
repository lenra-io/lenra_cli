use log;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub use clap::Args;
use dofigen_lib::{
    from_file_path, generate_dockerfile, generate_dockerignore, Artifact, Builder, Image,
};

use crate::cli::CliCommand;
use crate::config::{load_config_file, Generator, DEFAULT_CONFIG_FILE, LENRA_CACHE_DIRECTORY};

static OF_WATCHDOG_BUILDER: &str = "of-watchdog";
static OF_WATCHDOG_IMAGE: &str = "ghcr.io/openfaas/of-watchdog";
static OF_WATCHDOG_VERSION: &str = "0.9.6";

#[derive(Args)]
pub struct Build {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl CliCommand for Build {
    fn run(&self) {
        let conf = load_config_file(&self.config);
        // TODO: check the components API version

        // create the `.lenra` cache directory
        fs::create_dir_all(LENRA_CACHE_DIRECTORY).unwrap();

        match conf.generator {
            Generator::Dofigen(dofigen) => build_dofigen(dofigen.dofigen),
            Generator::DofigenFile(dofigen_file) => {
                build_dofigen(from_file_path(&dofigen_file.dofigen))
            }
            Generator::DofigenError { dofigen: _ } => {
                panic!("Your Dofigen configuration is not correct")
            }
            Generator::Dockerfile(dockerfile) => build_docker_image(Some(dockerfile.docker)),
            Generator::Docker(docker) => {
                save_docker_content(docker.docker, docker.ignore);
                build_docker_image(None);
            }
        }
    }
}

fn build_dofigen(image: Image) {
    // Generate the Dofigen config with OpenFaaS overlay to handle the of-watchdog
    let of_overlay = dofigen_of_overlay(image);

    // generate the Dockerfile and .dockerignore files with Dofigen
    let dockerfile = generate_dockerfile(&of_overlay);
    let dockerignore = generate_dockerignore(&of_overlay);
    save_docker_content(dockerfile, Some(dockerignore));

    // build the generated Dockerfile
    build_docker_image(None);
}

fn save_docker_content(dockerfile_content: String, dockerignore_content: Option<String>) {
    let dockerfile_path: PathBuf = [LENRA_CACHE_DIRECTORY, "Dockerfile"].iter().collect();
    let dockerignore_path: PathBuf = [LENRA_CACHE_DIRECTORY, ".dockerignore"].iter().collect();

    fs::write(dockerfile_path, dockerfile_content).expect("Unable to write the Dockerfile");
    if let Some(content) = dockerignore_content {
        fs::write(dockerignore_path, content).expect("Unable to write the .dockerignore file");
    }
}

fn dofigen_of_overlay(image: Image) -> Image {
    log::debug!("Adding OpenFaaS overlay to the Dofigen descriptor");
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
        cmd: Some(vec!["fwatchdog".to_string()]),
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

fn build_docker_image(dockerfile: Option<PathBuf>) {
    log::debug!("Build the Docker image");
    let dockerfile_path: PathBuf =
        dockerfile.unwrap_or([LENRA_CACHE_DIRECTORY, "Dockerfile"].iter().collect());
    let build_tar: PathBuf = [LENRA_CACHE_DIRECTORY, "image.tar"].iter().collect();
    let cache_directory: PathBuf = [LENRA_CACHE_DIRECTORY, "dockercache"].iter().collect();
    let mut command = Command::new("docker");
    let image_tag = "lenra/app";

    // TODO: display std out & err
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    command
        .arg("buildx")
        .arg("build")
        .arg("-f")
        .arg(dockerfile_path)
        .arg(format!(
            "--cache-to=type=local,dest={}",
            cache_directory.display()
        ))
        .arg(format!(
            "--cache-from=type=local,src={}",
            cache_directory.display()
        ))
        .arg("-t")
        .arg(image_tag)
        .arg("--output")
        .arg(format!("type=tar,dest={}", build_tar.display()))
        .arg(".");

    let output = command.output().expect("Failed building the Docker image");
    if !output.status.success() {
        panic!(
            "An error occured while building the Docker image:\n{}\n{}",
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        )
    }
    log::debug!("Image built");
}
