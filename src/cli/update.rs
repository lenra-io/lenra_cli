use std::process::Stdio;

pub use clap::Args;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::docker;
use crate::docker_compose::{get_services_images, Service};

#[derive(Args, Clone)]
pub struct Update {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// The service list to pull
    #[clap(value_enum, default_values = &["devtool", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

impl CliCommand for Update {
    fn run(&self) {
        log::info!("Updating Docker images");

        let conf = load_config_file(&self.config).ok();
        let dev_conf = if let Some(dev_opt) = conf.iter().map(|c| &c.dev).next() {
            dev_opt
        } else {
            &None
        };
        let images = get_services_images(dev_conf);

        let processes = self.services.iter().map(|service| {
            docker::pull(service.get_image(&images))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
        });

        processes.for_each(|p| {
            let output = p
                .expect("Failed to update the Docker image")
                .wait_with_output()
                .expect("Failed to get command output");

            if !output.status.success() {
                panic!(
                    "An error occured while updating Docker image:\n{}\n{}",
                    String::from_utf8(output.stdout).unwrap(),
                    String::from_utf8(output.stderr).unwrap()
                )
            }
        });
    }
}
