use log::debug;

use crate::{
    cli::CommandContext,
    docker_compose::{execute_compose_service_command, Service},
    errors::Result,
};

pub async fn stop_app_env(context: &mut CommandContext) -> Result<()> {
    debug!("Stop app environment");
    execute_compose_service_command(
        context,
        Service::Devtool,
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
