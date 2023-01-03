use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Command, Compose, DependsCondition, DependsOnOptions,
    Environment, Healthcheck, HealthcheckTest, Services,
};
use log::warn;
use std::process::Stdio;
use std::{convert::TryInto, env, fs, path::PathBuf};
use tokio::process;

use crate::errors::Error;
use crate::{
    config::{Dev, DOCKERCOMPOSE_DEFAULT_PATH},
    errors::{CommandError, Result},
    git::get_current_branch,
};

pub const APP_SERVICE_NAME: &str = "app";
pub const DEVTOOL_SERVICE_NAME: &str = "devtool";
pub const POSTGRES_SERVICE_NAME: &str = "postgres";
pub const MONGO_SERVICE_NAME: &str = "mongodb";
const APP_BASE_IMAGE: &str = "lenra/app/";
const APP_DEFAULT_IMAGE: &str = "my";
const APP_DEFAULT_IMAGE_TAG: &str = "latest";
const DEVTOOL_IMAGE: &str = "lenra/devtools";
const DEVTOOL_DEFAULT_TAG: &str = "beta";
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_IMAGE_TAG: &str = "13";
const MONGO_IMAGE: &str = "mongo";
const MONGO_IMAGE_TAG: &str = "5.0.11-focal";
pub const OF_WATCHDOG_PORT: u16 = 8080;
pub const DEVTOOL_PORT: u16 = 4000;
pub const MONGO_PORT: u16 = 27017;
pub const POSTGRES_PORT: u16 = 5432;

/// Generates the docker-compose.yml file
pub async fn generate_docker_compose(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: Vec<Service>,
) -> Result<()> {
    let compose_content =
        generate_docker_compose_content(dockerfile, dev_conf, exposed_services).await?;
    let compose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    fs::write(compose_path, compose_content).map_err(Error::from)?;
    Ok(())
}

async fn generate_docker_compose_content(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: Vec<Service>,
) -> Result<String> {
    let mut devtool_env_vec: Vec<(String, Option<String>)> = vec![
        ("POSTGRES_USER".to_string(), Some("postgres".to_string())),
        (
            "POSTGRES_PASSWORD".to_string(),
            Some("postgres".to_string()),
        ),
        ("POSTGRES_DB".to_string(), Some("lenra_devtool".to_string())),
    ];
    let postgres_envs: [(String, Option<String>); 3] = devtool_env_vec.clone().try_into().unwrap();

    devtool_env_vec.push((
        "POSTGRES_HOST".to_string(),
        Some(POSTGRES_SERVICE_NAME.to_string()),
    ));
    devtool_env_vec.push((
        "OF_WATCHDOG_URL".to_string(),
        Some(format!("http://{}:{}", APP_SERVICE_NAME, OF_WATCHDOG_PORT)),
    ));
    devtool_env_vec.push((
        "LENRA_API_URL".to_string(),
        Some(format!("http://{}:{}", DEVTOOL_SERVICE_NAME, DEVTOOL_PORT)),
    ));
    devtool_env_vec.push((
        "MONGO_HOSTNAME".to_string(),
        Some(MONGO_SERVICE_NAME.to_string()),
    ));
    let devtool_envs: [(String, Option<String>); 7] = devtool_env_vec.try_into().unwrap();

    let mongo_envs: [(String, Option<String>); 2] = [
        (
            "MONGO_INITDB_DATABASE".to_string(),
            Some("test".to_string()),
        ),
        (
            "CONFIG".to_string(),
            Some(format!(
                r#"{{"_id" : "rs0", "members" : [{{"_id" : 0,"host" : "{}:{}"}}]}}"#,
                MONGO_SERVICE_NAME, MONGO_PORT
            )),
        ),
    ];

    let service_images = get_services_images(dev_conf).await;

    let compose = Compose {
        services: Some(Services(
            [
                (
                    APP_SERVICE_NAME.into(),
                    Some(docker_compose_types::Service {
                        image: Some(service_images.app),
                        ports: if exposed_services.contains(&Service::App) { Some(vec![format!("{}:{}", OF_WATCHDOG_PORT, OF_WATCHDOG_PORT)])} else {None},
                        build_: Some(BuildStep::Advanced(AdvancedBuildStep {
                            context: "..".into(),
                            dockerfile: Some(dockerfile.to_str().unwrap().into()),
                            ..Default::default()
                        })),
                        // TODO: Add resources management  when managed by the docker-compose-types lib
                        ..Default::default()
                    }),
                ),
                (
                    DEVTOOL_SERVICE_NAME.into(),
                    Some(docker_compose_types::Service {
                        image: Some(service_images.devtool),
                        ports: Some(vec![format!("{}:{}", DEVTOOL_PORT, DEVTOOL_PORT)]),
                        environment: Some(Environment::KvPair(devtool_envs.into())),
                        healthcheck: Some(Healthcheck {
                            test: Some(HealthcheckTest::Multiple(vec![
                                "CMD".into(),
                                "wget".into(),
                                "--spider".into(),
                                "-q".into(),
                                "http://localhost:4000/health".into(),
                            ])),
                            start_period: Some("10s".into()),
                            interval: Some("1s".into()),
                            timeout: None,
                            retries: 5,
                            disable: false,
                        }),
                        depends_on: Some(DependsOnOptions::Conditional(
                            [(
                                POSTGRES_SERVICE_NAME.into(),
                                DependsCondition {
                                    condition: "service_healthy".into(),
                                },
                            ),(
                                MONGO_SERVICE_NAME.into(),
                                DependsCondition {
                                    condition: "service_healthy".into(),
                                },
                            )]
                            .into(),
                        )),
                        ..Default::default()
                    }),
                ),
                (
                    POSTGRES_SERVICE_NAME.into(),
                    Some(
                        docker_compose_types::Service {
                            image: Some(service_images.postgres),
                            ports: if exposed_services.contains(&Service::Postgres) {Some(vec![format!("{}:{}", POSTGRES_PORT, POSTGRES_PORT)])} else {None},
                            environment: Some(Environment::KvPair(postgres_envs.into())),
                            healthcheck: Some(Healthcheck {
                                test: Some(HealthcheckTest::Multiple(vec![
                                    "CMD".into(),
                                    "pg_isready".into(),
                                    "-U".into(),
                                    "postgres".into(),
                                ])),
                                start_period: Some("5s".into()),
                                interval: Some("1s".into()),
                                timeout: None,
                                retries: 5,
                                disable: false,
                            }),
                            ..Default::default()
                        }
                   ),
                ),
                (
                    MONGO_SERVICE_NAME.into(),
                    Some( docker_compose_types::Service {
                            image: Some(service_images.mongo),
                            ports: if exposed_services.contains(&Service::Mongo) {Some(vec![format!("{}:{}", MONGO_PORT, MONGO_PORT)])} else {None},
                            environment: Some(Environment::KvPair(mongo_envs.into())),
                            healthcheck: Some(Healthcheck {
                                test: Some(HealthcheckTest::Single(r#"test $$(echo "rs.initiate($$CONFIG).ok || rs.status().ok" | mongo --quiet) -eq 1"#.to_string())),
                                start_period: Some("5s".into()),
                                interval: Some("1s".into()),
                                timeout: None,
                                retries: 5,
                                disable: false,
                            }),
                            command: Some(Command::Simple("mongod --replSet rs0".into())),
                            ..Default::default()
                        }
                    ),
                ),
            ]
            .into(),
        )),
        ..Default::default()
    };
    serde_yaml::to_string(&compose).map_err(Error::from)
}

pub fn create_compose_command() -> process::Command {
    let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    let mut cmd = process::Command::new("docker");

    cmd.arg("compose").arg("-f").arg(dockercompose_path);

    cmd
}

pub async fn compose_up() -> Result<()> {
    let mut command = create_compose_command();

    command
        .arg("up")
        .arg("-d")
        .arg("--wait")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    log::debug!("cmd: {:?}", command);
    let output = command.spawn()?.wait_with_output().await?;

    if !output.status.success() {
        warn!(
            "An error occured while running the docker-compose app:\n{}",
            CommandError { command, output }
        )
    }
    Ok(())
}

pub async fn compose_build() -> Result<()> {
    let mut command = create_compose_command();
    command.arg("build");

    // Use Buildkit to improve performance
    command.env("DOCKER_BUILDKIT", "1");

    // Display std out & err
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    log::debug!("Build: {:?}", command);
    let output = command.spawn()?.wait_with_output().await?;

    if !output.status.success() {
        warn!(
            "An error occured while building the Docker image:\n{}",
            CommandError { command, output }
        )
    }
    Ok(())
}

pub async fn execute_compose_service_command(service: &str, cmd: &[&str]) -> Result<String> {
    let mut command = create_compose_command();

    command.arg("exec").arg(service);

    cmd.iter().for_each(|&part| {
        command.arg(part);
        ()
    });

    let output = command.output().await.map_err(Error::from)?;

    if !output.status.success() {
        return Err(Error::from(CommandError { command, output }));
    }

    String::from_utf8(output.stdout)
        .map(|name| name.trim().to_string())
        .map_err(Error::from)
}

fn current_dir_name() -> Option<String> {
    if let Ok(path) = env::current_dir() {
        path.file_name()
            .map(|name| String::from(name.to_str().unwrap()))
    } else {
        None
    }
}

pub async fn get_services_images(dev_conf: &Option<Dev>) -> ServiceImages {
    let default_app_image = current_dir_name().unwrap_or(APP_DEFAULT_IMAGE.to_string());
    let default_app_tag = get_current_branch()
        .await
        .ok()
        .unwrap_or(APP_DEFAULT_IMAGE_TAG.to_string());

    if let Some(dev) = dev_conf {
        ServiceImages {
            app: format!(
                "{}{}:{}",
                APP_BASE_IMAGE,
                dev.app_name.clone().unwrap_or(default_app_image),
                dev.app_tag.clone().unwrap_or(default_app_tag)
            ),
            devtool: format!(
                "{}:{}",
                DEVTOOL_IMAGE,
                dev.devtool_tag
                    .clone()
                    .unwrap_or(DEVTOOL_DEFAULT_TAG.to_string())
            ),
            postgres: format!(
                "{}:{}",
                POSTGRES_IMAGE,
                dev.postgres_tag
                    .clone()
                    .unwrap_or(POSTGRES_IMAGE_TAG.to_string())
            ),
            mongo: format!(
                "{}:{}",
                MONGO_IMAGE,
                dev.mongo_tag.clone().unwrap_or(MONGO_IMAGE_TAG.to_string())
            ),
        }
    } else {
        ServiceImages {
            app: format!(
                "{}{}:{}",
                APP_BASE_IMAGE, default_app_image, default_app_tag
            ),
            devtool: format!("{}:{}", DEVTOOL_IMAGE, DEVTOOL_DEFAULT_TAG),
            postgres: format!("{}:{}", POSTGRES_IMAGE, POSTGRES_IMAGE_TAG),
            mongo: format!("{}:{}", MONGO_IMAGE, MONGO_IMAGE_TAG),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum Service {
    App,
    Devtool,
    Postgres,
    Mongo,
}

impl Service {
    pub fn to_str(&self) -> &str {
        match self {
            Service::App => APP_SERVICE_NAME,
            Service::Devtool => DEVTOOL_SERVICE_NAME,
            Service::Postgres => POSTGRES_SERVICE_NAME,
            Service::Mongo => MONGO_SERVICE_NAME,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServiceImages {
    pub app: String,
    pub devtool: String,
    pub postgres: String,
    pub mongo: String,
}

impl ServiceImages {
    pub fn get(&self, service: &Service) -> String {
        match service {
            Service::App => self.app.clone(),
            Service::Devtool => self.devtool.clone(),
            Service::Postgres => self.postgres.clone(),
            Service::Mongo => self.mongo.clone(),
        }
    }
}
