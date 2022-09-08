use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Compose, DependsCondition, DependsOnOptions, Environment,
    Healthcheck, HealthcheckTest, Service, Services,
};

use crate::{
    config::{Dev, DOCKERCOMPOSE_DEFAULT_PATH},
    git::get_current_branch,
};

pub const APP_SERVICE_NAME: &str = "app";
pub const DEVTOOL_SERVICE_NAME: &str = "devtool";
pub const POSTGRES_SERVICE_NAME: &str = "postgres";
const APP_BASE_IMAGE: &str = "lenra/app/";
const APP_DEFAULT_IMAGE: &str = "my";
const APP_DEFAULT_IMAGE_TAG: &str = "latest";
const DEVTOOL_IMAGE: &str = "lenra/devtools";
const DEVTOOL_DEFAULT_TAG: &str = "beta";
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_IMAGE_TAG: &str = "13";
const OF_WATCHDOG_PORT: u16 = 8080;
const DEVTOOL_PORT: u16 = 4000;

/// Generates the docker-compose.yml file
pub fn generate_docker_compose(dockerfile: PathBuf, dev_conf: &Option<Dev>) {
    let compose_content = generate_docker_compose_content(dockerfile, dev_conf);
    let compose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    fs::write(compose_path, compose_content).expect("Unable to write the docker-compose file");
}

fn generate_docker_compose_content(dockerfile: PathBuf, dev_conf: &Option<Dev>) -> String {
    let postgres_envs = [
        ("POSTGRES_USER".to_string(), Some("postgres".to_string())),
        (
            "POSTGRES_PASSWORD".to_string(),
            Some("postgres".to_string()),
        ),
        ("POSTGRES_DB".to_string(), Some("lenra_devtool".to_string())),
    ];
    let devtool_envs: [(String, Option<String>); 6] = [
        postgres_envs.clone(),
        [
            (
                "POSTGRES_HOST".to_string(),
                Some(POSTGRES_SERVICE_NAME.to_string()),
            ),
            (
                "OF_WATCHDOG_URL".to_string(),
                Some(format!("http://{}:{}", APP_SERVICE_NAME, OF_WATCHDOG_PORT)),
            ),
            (
                "LENRA_API_URL".to_string(),
                Some(format!("http://{}:{}", DEVTOOL_SERVICE_NAME, DEVTOOL_PORT)),
            ),
        ],
    ]
    .concat()
    .try_into()
    .unwrap();

    let default_app_image = current_dir_name().unwrap_or(APP_DEFAULT_IMAGE.to_string());
    let default_app_tag = get_current_branch().unwrap_or(APP_DEFAULT_IMAGE_TAG.to_string());

    let service_images = if let Some(dev) = dev_conf {
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
        }
    } else {
        ServiceImages {
            app: format!(
                "{}{}:{}",
                APP_BASE_IMAGE, default_app_image, default_app_tag
            ),
            devtool: format!("{}:{}", DEVTOOL_IMAGE, DEVTOOL_DEFAULT_TAG),
            postgres: format!("{}:{}", POSTGRES_IMAGE, POSTGRES_IMAGE_TAG),
        }
    };

    let compose = Compose {
        services: Some(Services(
            [
                (
                    APP_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some(service_images.app),
                        build_: Some(BuildStep::Advanced(AdvancedBuildStep {
                            context: "..".into(),
                            dockerfile: Some(dockerfile.to_str().unwrap().into()),
                            ..Default::default()
                        })),
                        depends_on: Some(DependsOnOptions::Conditional(
                            [(
                                DEVTOOL_SERVICE_NAME.into(),
                                DependsCondition {
                                    condition: "service_healthy".into(),
                                },
                            )]
                            .into(),
                        )),
                        // TODO: Add resources management  when managed by the docker-compose-types lib
                        ..Default::default()
                    }),
                ),
                (
                    DEVTOOL_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some(service_images.devtool),
                        ports: Some(vec![format!("{}:{}", DEVTOOL_PORT, DEVTOOL_PORT)]),
                        environment: Some(Environment::KvPair(devtool_envs.into())),
                        depends_on: Some(DependsOnOptions::Conditional(
                            [(
                                POSTGRES_SERVICE_NAME.into(),
                                DependsCondition {
                                    condition: "service_healthy".into(),
                                },
                            )]
                            .into(),
                        )),
                        healthcheck: Some(Healthcheck {
                            test: Some(HealthcheckTest::Multiple(vec![
                                "CMD".into(),
                                "wget".into(),
                                "--spider".into(),
                                "-q".into(),
                                "http://localhost:4000".into(),
                            ])),
                            interval: Some("10s".into()),
                            timeout: Some("5s".into()),
                            retries: 5,
                            disable: false,
                        }),
                        ..Default::default()
                    }),
                ),
                (
                    POSTGRES_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some(service_images.postgres),
                        environment: Some(Environment::KvPair(postgres_envs.into())),
                        healthcheck: Some(Healthcheck {
                            test: Some(HealthcheckTest::Multiple(vec![
                                "CMD".into(),
                                "pg_isready".into(),
                                "-U".into(),
                                "postgres".into(),
                            ])),
                            interval: Some("5s".into()),
                            timeout: None,
                            retries: 5,
                            disable: false,
                        }),
                        ..Default::default()
                    }),
                ),
            ]
            .into(),
        )),
        ..Default::default()
    };
    serde_yaml::to_string(&compose).expect("Error generating the docker-compose file content")
}

pub fn create_compose_command() -> Command {
    let dockercompose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    let mut cmd = Command::new("docker");

    cmd.arg("compose").arg("-f").arg(dockercompose_path);

    cmd
}

pub fn compose_up() {
    let mut command = create_compose_command();

    command
        .arg("up")
        .arg("-d")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    log::debug!("cmd: {:?}", command);
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

pub fn compose_build() {
    let mut command = create_compose_command();
    command.arg("build");

    // Use Buildkit to improve performance
    command.env("DOCKER_BUILDKIT", "1");

    // Display std out & err
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    log::debug!("Build: {:?}", command);
    let output = command.output().expect("Failed building the Docker image");
    if !output.status.success() {
        panic!(
            "An error occured while building the Docker image:\n{}\n{}",
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        )
    }
}

pub fn execute_compose_service_command(service: &str, cmd: &[&str]) {
    let mut command = create_compose_command();

    command.arg("exec").arg(service);

    cmd.iter().for_each(|&part| {
        command.arg(part);
        ()
    });

    let output = command
        .output()
        .expect("Failed to execute the docker-compose exec command");

    if !output.status.success() {
        panic!(
            "docker-compose exec exited with code {}:\n\tcmd: {:?}\n\tstdout: {}\n\tstderr: {}",
            output.status.code().unwrap(),
            command,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        )
    }
}

fn current_dir_name() -> Option<String> {
    if let Ok(path) = env::current_dir() {
        path.file_name()
            .map(|name| String::from(name.to_str().unwrap()))
    } else {
        None
    }
}

struct ServiceImages {
    app: String,
    devtool: String,
    postgres: String,
}
