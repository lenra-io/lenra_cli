use std::path::PathBuf;
use std::process::{Command, Stdio};

pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH, load_config_file};
use crate::docker_compose::{APP_SERVICE_NAME, DEVTOOL_SERVICE_NAME, POSTGRES_SERVICE_NAME};

#[derive(Args)]
pub struct Start {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// The service attached.
    #[clap(value_enum, long, default_value = "app")]
    pub attach: Attach,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Attach {
    /// Attach app service
    App,
    /// Attach devtool service
    Devtool,
    /// Attach database service
    Database,
    /// Attach all services
    All,
    /// Detach
    None,
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
            .arg("up");
        match self.attach {
            Attach::App => command.arg("--attach").arg(APP_SERVICE_NAME),
            Attach::Devtool => command.arg("--attach").arg(DEVTOOL_SERVICE_NAME),
            Attach::Database => command.arg("--attach").arg(POSTGRES_SERVICE_NAME),
            Attach::All => &command,
            Attach::None => command.arg("-d"),
        };

        log::debug!("Start: {:?}", command);
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
        log::info!("App is stopped");
    }
}
