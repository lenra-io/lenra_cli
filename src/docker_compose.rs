use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Command, Compose, DependsCondition, DependsOnOptions, Deploy,
    EnvTypes, Environment, Healthcheck, HealthcheckTest, Limits, Resources, Services,
};
use lazy_static::lazy_static;
use log::{debug, warn};
use std::process::Stdio;
use std::{convert::TryInto, env, fs, path::PathBuf};
use tokio::process;

use crate::config::Image;
use crate::docker::normalize_tag;
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
pub const NON_ROOT_USER: &str = "12000";
const MEMORY_RESERVATION: &str = "128M";
const MEMORY_LIMIT: &str = "256M";

lazy_static! {
    static ref COMPOSE_COMMAND: std::process::Command = get_compose_command();
}

/// Generates the docker-compose.yml file
pub async fn generate_docker_compose(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: Vec<Service>,
    debug: bool,
) -> Result<()> {
    let compose_content =
        generate_docker_compose_content(dockerfile, dev_conf, exposed_services, debug).await?;
    let compose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    fs::write(compose_path, compose_content).map_err(Error::from)?;
    Ok(())
}

async fn generate_docker_compose_content(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: Vec<Service>,
    debug: bool,
) -> Result<String> {
    let mut devtool_env_vec: Vec<(String, Option<EnvTypes>)> = vec![
        (
            "POSTGRES_USER".into(),
            Some(EnvTypes::String("postgres".into())),
        ),
        (
            "POSTGRES_PASSWORD".into(),
            Some(EnvTypes::String("postgres".into())),
        ),
        (
            "POSTGRES_DB".into(),
            Some(EnvTypes::String("lenra_devtool".into())),
        ),
    ];
    let postgres_envs: [(String, Option<EnvTypes>); 3] =
        devtool_env_vec.clone().try_into().unwrap();

    devtool_env_vec.push((
        "POSTGRES_HOST".into(),
        Some(EnvTypes::String(POSTGRES_SERVICE_NAME.into())),
    ));
    devtool_env_vec.push((
        "OF_WATCHDOG_URL".into(),
        Some(EnvTypes::String(format!(
            "http://{}:{}",
            APP_SERVICE_NAME, OF_WATCHDOG_PORT
        ))),
    ));
    devtool_env_vec.push((
        "LENRA_API_URL".into(),
        Some(EnvTypes::String(format!(
            "http://{}:{}",
            DEVTOOL_SERVICE_NAME, DEVTOOL_PORT
        ))),
    ));
    devtool_env_vec.push((
        "MONGO_HOSTNAME".into(),
        Some(EnvTypes::String(MONGO_SERVICE_NAME.into())),
    ));
    let devtool_envs: [(String, Option<EnvTypes>); 7] = devtool_env_vec.try_into().unwrap();

    let mongo_envs: [(String, Option<EnvTypes>); 2] = [
        (
            "MONGO_INITDB_DATABASE".to_string(),
            Some(EnvTypes::String("test".into())),
        ),
        (
            "CONFIG".to_string(),
            Some(EnvTypes::String(format!(
                r#"{{"_id" : "rs0", "members" : [{{"_id" : 0,"host" : "{}:{}"}}]}}"#,
                MONGO_SERVICE_NAME, MONGO_PORT
            ))),
        ),
    ];

    let service_images = get_services_images(dev_conf).await;
    let mut app_ports = vec![];
    if exposed_services.contains(&Service::App) {
        app_ports.push(format!("{}:{}", OF_WATCHDOG_PORT, OF_WATCHDOG_PORT));
    }
    if debug {
        if let Some(conf) = dev_conf {
            if let Some(dofigen) = &conf.dofigen {
                if let Some(ports) = &dofigen.ports {
                    for port in ports {
                        app_ports.push(format!("{}:{}", port, port));
                    }
                }
            }
        }
    }

    let compose = Compose {
        services: Some(Services(
            [
                (
                    APP_SERVICE_NAME.into(),
                    Some(docker_compose_types::Service {
                        image: Some(service_images.app),
                        ports: if !app_ports.is_empty() { Some(app_ports)} else {None},
                        build_: Some(BuildStep::Advanced(AdvancedBuildStep {
                            context: "..".into(),
                            dockerfile: Some(dockerfile.to_str().unwrap().into()),
                            ..Default::default()
                        })),
                        user: Some(NON_ROOT_USER.into()),
                        deploy: Some(Deploy {
                            resources: Some(Resources {
                            limits: Some(Limits {
                                memory: Some(MEMORY_LIMIT.into()),
                                ..Default::default()
                            }),
                            reservations: Some(Limits {
                                memory: Some(MEMORY_RESERVATION.into()),
                                ..Default::default()
                            })
                        }),
                        ..Default::default()
                     }),
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
    let mut cmd = process::Command::from(COMPOSE_COMMAND.clone());
    cmd.arg("-f").arg(dockercompose_path).kill_on_drop(true);

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
    let default_app_image = format!(
        "{}{}",
        APP_BASE_IMAGE,
        current_dir_name().unwrap_or(APP_DEFAULT_IMAGE.to_string())
    );
    let default_app_tag = match get_current_branch().await {
        Ok(branch_name) => normalize_tag(branch_name),
        _ => APP_DEFAULT_IMAGE_TAG.to_string(),
    };

    let dev = dev_conf.clone().unwrap_or(Dev {
        ..Default::default()
    });
    ServiceImages {
        app: dev
            .app
            .unwrap_or(Image {
                ..Default::default()
            })
            .to_image(&default_app_image, &default_app_tag),
        devtool: dev
            .devtool
            .unwrap_or(Image {
                ..Default::default()
            })
            .to_image(DEVTOOL_IMAGE, DEVTOOL_DEFAULT_TAG),
        postgres: dev
            .postgres
            .unwrap_or(Image {
                ..Default::default()
            })
            .to_image(POSTGRES_IMAGE, POSTGRES_IMAGE_TAG),
        mongo: dev
            .mongo
            .unwrap_or(Image {
                ..Default::default()
            })
            .to_image(MONGO_IMAGE, MONGO_IMAGE_TAG),
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

fn get_compose_command() -> std::process::Command {
    match std::process::Command::new("docker-compose")
        .arg("version")
        .spawn()
    {
        Ok(_) => {
            debug!("Using 'docker-compose'");
            std::process::Command::new("docker-compose")
        }
        Err(e) => {
            if std::io::ErrorKind::NotFound != e.kind() {
                warn!(
                    "An unexpected error occured while runing 'docker-compose version' {}",
                    e
                );
            }
            debug!("Using 'docker compose'");
            let mut cmd = std::process::Command::new("docker");
            cmd.arg("compose");
            cmd
        }
    }
    .into()
}

trait CloneCommand {
    fn clone(&self) -> Self;
}

impl CloneCommand for std::process::Command {
    fn clone(&self) -> Self {
        let mut new = Self::new(self.get_program());
        new.args(self.get_args());
        new
    }
}

#[cfg(test)]
mod test_get_services_images {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::git;

    use super::*;
    use mocktopus::mocking::*;
    use regex::Regex;

    #[tokio::test]
    async fn basic() {
        let app_tag = "test";
        let images: ServiceImages = get_services_images(&Some(Dev {
            app: Some(Image {
                image: None,
                tag: Some(app_tag.into()),
            }),
            ..Default::default()
        }))
        .await;
        assert_eq!(
            images.get(&Service::App),
            format!("lenra/app/lenra_cli:{}", app_tag)
        );
        assert_eq!(
            images.get(&Service::Devtool),
            format!("{}:{}", DEVTOOL_IMAGE, DEVTOOL_DEFAULT_TAG)
        );
        assert_eq!(
            images.get(&Service::Postgres),
            format!("{}:{}", POSTGRES_IMAGE, POSTGRES_IMAGE_TAG)
        );
        assert_eq!(
            images.get(&Service::Mongo),
            format!("{}:{}", MONGO_IMAGE, MONGO_IMAGE_TAG)
        );
    }

    #[tokio::test]
    async fn branch_name() {
        git::get_current_branch
            .mock_safe(|| MockResult::Return(Box::pin(async move { Ok("test".to_string()) })));
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(
            images.get(&Service::App),
            "lenra/app/lenra_cli:test".to_string()
        );
    }

    #[tokio::test]
    async fn path_branch_name() {
        git::get_current_branch.mock_safe(|| {
            MockResult::Return(Box::pin(async move {
                Ok("prefixed/branch-name_withUnderscore".to_string())
            }))
        });
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(
            images.get(&Service::App),
            "lenra/app/lenra_cli:prefixed-branch-name_withUnderscore".to_string()
        );
    }

    #[tokio::test]
    async fn long_branch_name() {
        let branch_name =
            "prefixed/branch-name/with_many-many.many_many-many-many/underscore".to_string();
        let re = Regex::new(r"[^A-Za-z0-9._-]").unwrap();
        let tag = re.replace_all(branch_name.as_str(), "-").to_string();
        let mut hacher = DefaultHasher::new();
        tag.hash(&mut hacher);
        let hash = format!("{:X}", hacher.finish());
        let tag = format!(
            "{}{}",
            tag.chars().take(63 - hash.len()).collect::<String>(),
            hash
        );

        git::get_current_branch.mock_safe(move || {
            let branch = branch_name.clone();
            MockResult::Return(Box::pin(async move { Ok(branch) }))
        });
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(
            images.get(&Service::App),
            format!("lenra/app/lenra_cli:{}", tag)
        );
    }
}
