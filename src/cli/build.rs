use log;

use clap;

use crate::cli::CliCommand;
use crate::config::{load_config_file, DEFAULT_CONFIG_FILE};
use crate::docker_compose::compose_build;

#[derive(clap::Args, Default)]
pub struct Build {
    /// The app configuration file.
    #[clap(parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    #[clap(long, action)]
    pub expose: bool,
}

impl Build {
    /// Builds a Dockerfile. If None, get's it at the default path: ./.lenra/Dockerfile
    fn build_docker_compose(&self) {
        log::info!("Build the Docker image");

        compose_build();

        log::info!("Image built");
    }
}

impl CliCommand for Build {
    fn run(&self) {
        let conf = load_config_file(&self.config);
        // TODO: check the components API version

        conf.generate_files(self.expose);

        // self.build_docker_image(conf);
        self.build_docker_compose();
    }
}
