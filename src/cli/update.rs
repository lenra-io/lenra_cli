use async_trait::async_trait;
pub use clap::Args;
use futures::future::join_all;
use log::{warn, info};

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::docker;
use crate::docker_compose::{get_services_images, Service};
use crate::errors::{CommandError, Result};

#[derive(Args, Clone)]
pub struct Update {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// The service list to pull
    #[clap(value_enum, default_values = &["devtool", "postgres", "mongo"])]
    pub services: Vec<Service>,
}

#[async_trait]
impl CliCommand for Update {
    async fn run(&self) -> Result<()> {
        log::info!("Updating Docker images");

        let conf = load_config_file(&self.config).ok();
        let dev_conf = if let Some(dev_opt) = conf.iter().map(|c| &c.dev).next() {
            dev_opt
        } else {
            &None
        };
        let images = get_services_images(dev_conf).await;

        // Pull images in parallele
        let processes = self.services.iter().map(|service| async move {
            let image = service.get_image(&images);
            info!("Start pulling {}", image);
            let command = docker::pull(image);

            let res = command.output().await;

            match res {
                Ok(output) => {
                    if !output.status.success() {
                        warn!("{}", CommandError { command, output });
                    }
                    else {
                        info!("Image pulled {}", image);
                    }
                }
                Err(err) => warn!("{}", err),
            }
        });

        // Wait for all the pull end
        join_all(processes).await;
        Ok(())
    }
}
