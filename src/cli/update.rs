use async_trait::async_trait;
pub use clap::Args;
use log::{info, warn};
use tokio::task::JoinSet;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::docker;
use crate::docker_compose::{get_services_images, Service, ServiceImages};
use crate::errors::{CommandError, Error, Result};

#[derive(Args, Debug, Clone)]
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
        let mut processes = JoinSet::new();

        self.services
            .iter()
            .filter(|&service| match service {
                Service::App => false,
                _ => true,
            })
            .for_each(|service| {
                let imgs = images.clone();
                let serv = service.clone();
                processes.spawn(async move { pull_service_image(&imgs, &serv).await });
            });

        // Wait for all the pull end
        while let Some(res) = processes.join_next().await {
            res?;
        }

        Ok(())
    }
}

async fn pull_service_image(images: &ServiceImages, service: &Service) {
    let image = images.get(service);
    info!("Start pulling {}", image);
    let mut command = docker::pull(image.clone());

    let spawn_res = command.spawn().map_err(Error::from);
    if let Err(err) = &spawn_res {
        warn!("{}", err)
    }
    let res = spawn_res
        .unwrap()
        .wait_with_output()
        .await
        .map_err(Error::from);
    if let Err(err) = &res {
        warn!("{}", err)
    }

    let output = res.unwrap();
    if !output.status.success() {
        warn!("{}", CommandError { command, output });
    } else {
        info!("Image pulled {}", image);
    }
}
