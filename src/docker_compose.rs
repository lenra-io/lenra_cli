use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Command, Compose, DependsCondition, DependsOnOptions, Deploy,
    EnvTypes, Environment, Healthcheck, HealthcheckTest, Limits, Resources, Services,
};
use lazy_static::lazy_static;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::{convert::TryInto, env, fs, path::PathBuf};
use strum::Display;
use tokio::process;

use crate::cli::CommandContext;
use crate::command::{get_command_output, is_inherit_stdio, run_command};
use crate::config::{DevToolConf, Image, ImageConf};
use crate::docker::normalize_tag;
use crate::errors::Error;
use crate::{
    config::{Dev, DOCKERCOMPOSE_DEFAULT_PATH},
    errors::Result,
    git::get_current_branch,
};

pub const APP_SERVICE_NAME: &str = "app";
pub const DEVTOOL_SERVICE_NAME: &str = "devtool";
pub const POSTGRES_SERVICE_NAME: &str = "postgres";
pub const MONGO_SERVICE_NAME: &str = "mongo";
const APP_BASE_IMAGE: &str = "lenra/app/";
const APP_DEFAULT_IMAGE: &str = "my";
const APP_DEFAULT_IMAGE_TAG: &str = "latest";
const DEVTOOL_IMAGE: &str = "lenra/devtools";
const DEVTOOL_DEFAULT_TAG: &str = "latest";
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_IMAGE_TAG: &str = "13";
const MONGO_IMAGE: &str = "mongo";
const MONGO_IMAGE_TAG: &str = "5";
pub const OF_WATCHDOG_PORT: u16 = 8080;
pub const DEVTOOL_WEB_PORT: u16 = 4000;
pub const DEVTOOL_API_PORT: u16 = 4001;
pub const DEVTOOL_OAUTH_PORT: u16 = 4444;
pub const MONGO_PORT: u16 = 27017;
pub const POSTGRES_PORT: u16 = 5432;
pub const NON_ROOT_USER: &str = "12000";
const MEMORY_RESERVATION: &str = "128M";
const MEMORY_LIMIT: &str = "256M";

lazy_static! {
    static ref COMPOSE_COMMAND: std::process::Command = get_compose_command();
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInformations {
    service: Service,
    #[serde(rename = "ID")]
    id: String,
    name: String,
    image: String,
    command: String,
    project: String,
    state: ServiceState,
    status: String,
    created: String,
    health: String,
    exit_code: String,
    publishers: Vec<Publisher>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Publisher {
    #[serde(rename = "URL")]
    url: String,
    target_port: u16,
    published_port: u16,
    protocol: String,
}

#[derive(clap::ValueEnum, Serialize, Deserialize, Clone, Debug, PartialEq, Display)]
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

#[derive(clap::ValueEnum, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ServiceState {
    Running,
    Paused,
    Restarting,
    Removing,
    Dead,
    Created,
    Exited,
}

/// Generates the docker-compose.yml file
pub async fn generate_docker_compose(
    context: &mut CommandContext,
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    debug: bool,
) -> Result<()> {
    let compose_path: PathBuf = context.resolve_path(&DOCKERCOMPOSE_DEFAULT_PATH.iter().collect());
    let compose_content =
        generate_docker_compose_content(dockerfile, dev_conf, &context.expose, debug).await?;
    fs::write(compose_path, compose_content).map_err(Error::from)?;
    Ok(())
}

fn devtool_log_level(dev_conf: &Option<Dev>) -> Option<String> {
    dev_conf.clone()?.devtool?.log_level.clone()
}

async fn generate_docker_compose_content(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: &Vec<Service>,
    debug: bool,
) -> Result<String> {
    let compose =
        generate_docker_compose_struct(dockerfile, dev_conf, exposed_services, debug).await?;
    serde_yaml::to_string(&compose).map_err(Error::from)
}

async fn generate_docker_compose_struct(
    dockerfile: PathBuf,
    dev_conf: &Option<Dev>,
    exposed_services: &Vec<Service>,
    debug: bool,
) -> Result<Compose> {
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
            DEVTOOL_SERVICE_NAME, DEVTOOL_API_PORT
        ))),
    ));
    devtool_env_vec.push((
        "MONGO_HOSTNAME".into(),
        Some(EnvTypes::String(MONGO_SERVICE_NAME.into())),
    ));
    devtool_env_vec.push((
        "LOG_LEVEL".into(),
        devtool_log_level(dev_conf).map(|level| EnvTypes::String(level)),
    ));
    let devtool_envs: [(String, Option<EnvTypes>); 8] = devtool_env_vec.try_into().unwrap();

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
        app_ports.push(port_to_port_binding(OF_WATCHDOG_PORT));
    }
    if debug {
        if let Some(conf) = dev_conf {
            if let Some(dofigen) = &conf.dofigen {
                if let Some(ports) = &dofigen.ports {
                    for port in ports {
                        app_ports.push(port_to_port_binding(*port));
                    }
                }
            }
        }
    }

    Ok(Compose {
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
                        ports: Some(vec![DEVTOOL_WEB_PORT, DEVTOOL_API_PORT, DEVTOOL_OAUTH_PORT].into_iter().map(port_to_port_binding).collect()),
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
                            ports: if exposed_services.contains(&Service::Postgres) {Some(vec![port_to_port_binding(POSTGRES_PORT)])} else {None},
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
                            ports: if exposed_services.contains(&Service::Mongo) {Some(vec![port_to_port_binding(MONGO_PORT)])} else {None},
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
    })
}

fn port_to_port_binding(port: u16) -> String {
    format!("{port}:{port}")
}

pub fn create_compose_command(context: &mut CommandContext) -> process::Command {
    let dockercompose_path: PathBuf =
        context.resolve_path(&DOCKERCOMPOSE_DEFAULT_PATH.iter().collect());
    let mut cmd = process::Command::from(COMPOSE_COMMAND.clone());
    cmd.arg("-f").arg(dockercompose_path).kill_on_drop(true);
    if is_inherit_stdio() {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    cmd
}

pub async fn compose_up(context: &mut CommandContext) -> Result<()> {
    run_command(
        create_compose_command(context)
            .arg("up")
            .arg("-d")
            .arg("--wait"),
        Some(context.verbose),
    )
    .await?;
    Ok(())
}

pub async fn compose_down(context: &mut CommandContext) -> Result<()> {
    run_command(
        create_compose_command(context).arg("down").arg("--volumes"),
        Some(context.verbose),
    )
    .await?;
    Ok(())
}

pub async fn compose_build(context: &mut CommandContext) -> Result<()> {
    run_command(
        create_compose_command(context)
            .arg("build")
            // Use Buildkit to improve performance
            .env("DOCKER_BUILDKIT", "1"),
        Some(context.verbose),
    )
    .await?;
    Ok(())
}

pub async fn compose_pull(context: &mut CommandContext, services: &Vec<Service>) -> Result<()> {
    log::debug!("Pulling services: {:?}", services);
    let mut command = create_compose_command(context);
    command.arg("pull");
    services.iter().for_each(|service| {
        command.arg(service.to_str());
    });

    run_command(&mut command, Some(context.verbose)).await?;
    Ok(())
}

/// List all the current Docker Compose running services
pub async fn list_running_services(context: &mut CommandContext) -> Result<Vec<Service>> {
    let services: Vec<Service> = get_command_output(
        create_compose_command(context)
            .arg("ps")
            .arg("--services")
            .arg("--filter")
            .arg("status=running"),
    )
    .await
    .map(|output| {
        output
            .lines()
            .map(|service| match service.trim() {
                APP_SERVICE_NAME => Some(Service::App),
                DEVTOOL_SERVICE_NAME => Some(Service::Devtool),
                POSTGRES_SERVICE_NAME => Some(Service::Postgres),
                MONGO_SERVICE_NAME => Some(Service::Mongo),
                _ => None,
            })
            .filter_map(|service| service)
            .collect()
    })?;
    Ok(services)
}

// /// Get the given Docker Compose service information
// pub async fn get_service_informations(
//     context: &mut CommandContext,
//     service: Service,
// ) -> Result<ServiceInformations> {
//     let output = get_command_output(
//         create_compose_command(context)
//             .arg("ps")
//             .arg(service.to_str())
//             .arg("--format")
//             .arg("json"),
//     )
//     .await?;
//     let infos: ServiceInformations = serde_yaml::from_str(output.as_str())?;
//     Ok(infos)
// }

// /// Get the given Docker Compose service published port
// pub async fn get_service_published_ports(
//     context: &mut CommandContext,
//     service: Service,
// ) -> Result<Vec<u16>> {
//     let infos = get_service_informations(context, service).await?;
//     let ports = infos
//         .publishers
//         .iter()
//         .map(|publisher| publisher.published_port)
//         .unique()
//         .collect();
//     Ok(ports)
// }

pub async fn execute_compose_service_command(
    context: &mut CommandContext,
    service: Service,
    cmd: &[&str],
) -> Result<String> {
    let mut command = create_compose_command(context);
    command.arg("exec").arg(service.to_str());

    cmd.iter().for_each(|&part| {
        command.arg(part);
    });

    get_command_output(&mut command).await
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
    let default_app_tag = match get_current_branch(None).await {
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
            .unwrap_or(DevToolConf {
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

fn get_compose_command() -> std::process::Command {
    match std::process::Command::new("docker-compose")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
        assert_eq!(images.app, format!("lenra/app/lenra_cli:{}", app_tag));
        assert_eq!(
            images.devtool,
            format!("{}:{}", DEVTOOL_IMAGE, DEVTOOL_DEFAULT_TAG)
        );
        assert_eq!(
            images.postgres,
            format!("{}:{}", POSTGRES_IMAGE, POSTGRES_IMAGE_TAG)
        );
        assert_eq!(images.mongo, format!("{}:{}", MONGO_IMAGE, MONGO_IMAGE_TAG));
    }

    #[tokio::test]
    async fn branch_name() {
        git::get_current_branch
            .mock_safe(|_| MockResult::Return(Box::pin(async move { Ok("test".to_string()) })));
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(images.app, "lenra/app/lenra_cli:test".to_string());
    }

    #[tokio::test]
    async fn path_branch_name() {
        git::get_current_branch.mock_safe(|_| {
            MockResult::Return(Box::pin(async move {
                Ok("prefixed/branch-name_withUnderscore".to_string())
            }))
        });
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(
            images.app,
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

        git::get_current_branch.mock_safe(move |_| {
            let branch = branch_name.clone();
            MockResult::Return(Box::pin(async move { Ok(branch) }))
        });
        let images: ServiceImages = get_services_images(&None).await;
        assert_eq!(images.app, format!("lenra/app/lenra_cli:{}", tag));
    }
}

#[cfg(test)]
mod dev_tests {
    use super::*;

    #[tokio::test]
    async fn devtool_log_level() {
        let devtool = DevToolConf {
            log_level: Some("debug".into()),
            ..Default::default()
        };
        let dev_conf = Some(Dev {
            devtool: Some(devtool.clone()),
            ..Default::default()
        });
        // generate docker compose content
        let compose =
            generate_docker_compose_struct(PathBuf::from("Dockerfile"), &dev_conf, &vec![], false)
                .await
                .unwrap();
        let services = compose.services.unwrap().0;
        let devtool_service = services
            .get(DEVTOOL_SERVICE_NAME)
            .unwrap()
            .as_ref()
            .unwrap();
        let devtool_env = devtool_service.environment.as_ref().unwrap();
        match devtool_env {
            Environment::KvPair(envs) => {
                let env_var = envs.get("LOG_LEVEL").unwrap().as_ref().unwrap();
                match env_var {
                    EnvTypes::String(value) => {
                        assert_eq!(value, "debug");
                    }
                    _ => {
                        panic!("Unexpected env var type");
                    }
                }
            }
            Environment::List(envs) => {
                let env_var = envs
                    .into_iter()
                    .find(|env_var| env_var.starts_with("LOG_LEVEL="))
                    .unwrap();
                assert_eq!(env_var, "LOG_LEVEL=debug");
            }
        }
    }
}
