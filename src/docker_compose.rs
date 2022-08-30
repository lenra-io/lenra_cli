use std::{fs, path::PathBuf};

use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Compose, DependsCondition, DependsOnOptions, Environment,
    Healthcheck, HealthcheckTest, Service, Services,
};

use crate::config::{DOCKERCOMPOSE_DEFAULT_PATH, Dev};

pub const APP_SERVICE_NAME: &str = "app";
pub const DEVTOOL_SERVICE_NAME: &str = "devtool";
pub const POSTGRES_SERVICE_NAME: &str = "postgres";
const DEVTOOL_IMAGE: &str = "lenra/devtools";
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_IMAGE_TAG: &str = "13";
const OF_WATCHDOG_PORT: u16 = 8080;
const DEVTOOL_PORT: u16 = 4000;

/// Generates the docker-compose.yml file
pub fn generate_docker_compose(dockerfile: PathBuf, dev_conf: &Dev) {
    let compose_content = generate_docker_compose_content(dockerfile, dev_conf);
    let compose_path: PathBuf = DOCKERCOMPOSE_DEFAULT_PATH.iter().collect();
    fs::write(compose_path, compose_content).expect("Unable to write the docker-compose file");
}

fn generate_docker_compose_content(dockerfile: PathBuf, dev_conf: &Dev) -> String {
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
            ("POSTGRES_HOST".to_string(), Some(POSTGRES_SERVICE_NAME.to_string())),
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
    let compose = Compose {
        services: Some(Services(
            [
                (
                    APP_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some("lenra/my-app".into()),
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
                        image: Some(format!("{}:{}", DEVTOOL_IMAGE, dev_conf.devtool_tag)),
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
                        image: Some(format!("{}:{}", POSTGRES_IMAGE, POSTGRES_IMAGE_TAG)),
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
