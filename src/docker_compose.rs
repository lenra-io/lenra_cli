use std::path::PathBuf;

use docker_compose_types::{
    AdvancedBuildStep, BuildStep, Compose, DependsOnOptions, Environment, Service, Services,
};

pub const APP_SERVICE_NAME: &str = "app";
pub const DEVTOOL_SERVICE_NAME: &str = "devtool";
pub const POSTGRES_SERVICE_NAME: &str = "postgres";
const DEVTOOL_IMAGE: &str = "lenra/devtools:beta";
const DEVTOOL_PORT: u16 = 4000;

pub fn generate_docker_compose_file(dockerfile: &PathBuf) -> String {
    let postgres_envs = [
        ("POSTGRES_USER".to_string(), Some("postgres".to_string())),
        (
            "POSTGRES_PASSWORD".to_string(),
            Some("postgres".to_string()),
        ),
        ("POSTGRES_HOST".to_string(), Some("postgres".to_string())),
        ("POSTGRES_DB".to_string(), Some("lenra_devtool".to_string())),
    ];
    let compose = Compose {
        services: Some(Services(
            [
                (
                    APP_SERVICE_NAME.into(),
                    Some(Service {
                        build_: Some(BuildStep::Advanced(AdvancedBuildStep {
                            context: "..".into(),
                            dockerfile: Some(dockerfile.to_str().unwrap().into()),
                            ..Default::default()
                        })),
                        depends_on: Some(DependsOnOptions::Simple(vec![
                            DEVTOOL_SERVICE_NAME.into()
                        ])),
                        ..Default::default()
                    }),
                ),
                (
                    DEVTOOL_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some(DEVTOOL_IMAGE.into()),
                        ports: Some(vec![format!("{}", DEVTOOL_PORT)]),
                        environment: Some(Environment::KvPair(postgres_envs.clone().into())),
                        depends_on: Some(DependsOnOptions::Simple(vec![
                            POSTGRES_SERVICE_NAME.into()
                        ])),
                        ..Default::default()
                    }),
                ),
                (
                    POSTGRES_SERVICE_NAME.into(),
                    Some(Service {
                        image: Some(DEVTOOL_IMAGE.into()),
                        environment: Some(Environment::KvPair(postgres_envs.into())),
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
