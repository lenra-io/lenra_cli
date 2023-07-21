use std::path::PathBuf;

use crate::{
    config::{load_config_file, DOCKERCOMPOSE_DEFAULT_PATH},
    devtool::stop_app_env,
    docker_compose::{self, compose_build, compose_up, Service},
    errors::{Error, Result},
};

pub async fn generate_app_env(
    config: &PathBuf,
    expose: &Vec<Service>,
    production: bool,
) -> Result<()> {
    log::info!("Generating the app environment");
    let conf = load_config_file(config)?;
    // TODO: check the components API version

    conf.generate_files(expose, !production).await
}

pub async fn build_app() -> Result<()> {
    log::info!("Build the Docker image");
    compose_build().await?;
    log::info!("Image built");
    Ok(())
}

pub async fn start_env() -> Result<()> {
    let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    if !dockercompose_path.exists() {
        return Err(Error::NeverBuiltApp);
    }

    log::info!("Start the containers");
    compose_up().await
}

pub async fn clear_cache() -> Result<()> {
    log::info!("Clearing cache");
    stop_app_env().await
}

pub fn display_app_access_url() {
    println!(
        "\nApplication available at http://localhost:{}\n",
        docker_compose::DEVTOOL_WEB_PORT
    );
}
