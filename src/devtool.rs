use log::debug;

use crate::{
    docker_compose::{execute_compose_service_command, DEVTOOL_SERVICE_NAME},
    errors::Result,
};

pub async fn stop_app_env() -> Result<()> {
    debug!("Stop app environment");
    execute_compose_service_command(
        DEVTOOL_SERVICE_NAME,
        &[
            "bin/dev_tools",
            "rpc",
            "ApplicationRunner.Environment.DynamicSupervisor.stop_env(1)",
        ],
    )
    .await?;
    debug!("App environment stopped");
    Ok(())
}
