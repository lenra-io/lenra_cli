use std::path::PathBuf;
use std::process::{Command, Stdio};

pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH, load_config_file};

#[derive(Args)]
pub struct Start {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,
}

impl Start {
    /// Starts the docker-compose
    fn start_docker_compose(&self) {
        let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
        if !dockercompose_path.exists() {
            let conf = load_config_file(&self.config);
            // TODO: check the components API version

            conf.generate_files();
        }

        let mut command = Command::new("docker");

        // TODO: display std out & err
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        command
            .arg("compose")
            .arg("-f")
            .arg(dockercompose_path)
            .arg("up")
            .arg("-d");

        log::debug!("cmd: {:?}", command);
        let output = command
            .output()
            .expect("Failed to start the docker-compose app");
        if !output.status.success() {
            panic!(
                "An error occured while running the docker-compose app:\n{}\n{}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )
        }
    }
}

impl CliCommand for Start {
    fn run(&self) {
        log::info!("Starting the app");
        self.start_docker_compose();
    }
}
