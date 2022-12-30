use crate::{
    docker_compose::{execute_compose_service_command, DEVTOOL_SERVICE_NAME},
    errors::Result,
};

pub fn stop_app_env() -> Result<()> {
    execute_compose_service_command(
        DEVTOOL_SERVICE_NAME,
        &[
            "bin/dev_tools",
            "rpc",
            "ApplicationRunner.Environment.DynamicSupervisor.stop_env(1)",
        ],
    )
}
