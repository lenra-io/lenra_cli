use std::fmt::Debug;

use async_trait::async_trait;
use clap::Args;

use crate::{app_checker, errors::Result};

use super::{CliCommand, CommandContext};

// pub const RULE_SEPARATOR: &str = ":";
// pub const VIEW: &str = "view";

#[derive(Args, Clone, Debug)]
pub struct Check {
    /// The strict mode also fails with warning rules.
    #[clap(long, action)]
    pub strict: bool,

    /// A list of rules to ignore
    #[clap(long)]
    pub ignore: Option<Vec<String>>,

    /// The rules
    #[clap()]
    pub rules: Vec<String>,
}

/// The check subcommands
// #[derive(Subcommand, Clone, Debug)]
// pub enum CheckCommandType {
//     // /// Checks the current project as an app
//     // App(CheckParameters),
//     /// Checks the current project as a template
//     Template(CheckParameters),
// }

#[async_trait]
impl CliCommand for Check {
    async fn run(&self, _context: CommandContext) -> Result<()> {
        // check that the app service is exposed
        // if get_service_published_ports(Service::App).await?.is_empty() {
        //     return Err(Error::ServiceNotExposed(Service::App));
        // }
        // check the app
        app_checker::check_app()?;
        Ok(())
    }
}

// fn ignore_rule(parts: Vec<String>, ignores: Vec<String>) -> bool {
//     let mut prefix = String::new();
//     for part in parts {
//         prefix.push_str(part.as_str());
//         if ignores.contains(&prefix) {
//             return true;
//         }
//         prefix.push_str(RULE_SEPARATOR);
//         if ignores.contains(&format!("{}*", prefix)) {
//             return true;
//         }
//     }
//     return false;
// }
