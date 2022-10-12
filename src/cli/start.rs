use std::path::PathBuf;

pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE, DOCKERCOMPOSE_DEFAULT_PATH};
use crate::docker_compose::{compose_up, execute_compose_service_command, DEVTOOL_SERVICE_NAME};

#[derive(Args, Default)]
pub struct Start {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes all services ports.
    #[clap(long, action)]
    pub expose: bool,
}

impl CliCommand for Start {
    fn run(&self) {
        log::info!("Starting the app");

        let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
        if !dockercompose_path.exists() {
            let conf = load_config_file(&self.config);
            // TODO: check the components API version

            conf.generate_files(self.expose);
        }

        // Start the containers
        compose_up();
        // Stop the devtool app env to reset cache
        let result = execute_compose_service_command(
            DEVTOOL_SERVICE_NAME,
            &[
                "bin/dev_tools",
                "rpc",
                "ApplicationRunner.Environment.DynamicSupervisor.stop_env(1)",
            ],
        );
        if let Err(error) = result {
            log::info!("{}", error);
        }
        // Open the app
        open::that("http://localhost:4000").unwrap();
    }
}
