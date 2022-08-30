use log;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap;

use crate::cli::CliCommand;
use crate::config::{
    load_config_file, DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH
};

#[derive(clap::Args)]
pub struct Build {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl Build {
    /// Builds a Dockerfile. If None, get's it at the default path: ./.lenra/Dockerfile
    fn build_docker_compose(&self) {
        log::info!("Build the Docker image");
        let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
        let mut command = Command::new("docker");

        // TODO: display std out & err
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        command
            .arg("compose")
            .arg("-f")
            .arg(dockercompose_path)
            .arg("build");
        // command
        //     .arg("buildx")
        //     .arg("bake")
        //     .arg("-f")
        //     .arg(dockercompose_path)
        //     .arg("--load");

        log::debug!("Build: {:?}", command);
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

        conf.generate_files();

        // self.build_docker_image(conf);
        self.build_docker_compose();
    }
}
