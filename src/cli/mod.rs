use std::{future::Future, path::PathBuf};

use async_trait::async_trait;
pub use clap::{Args, Parser, Subcommand};
use loading::Loading;
use log::debug;

use crate::{
    config::{load_config_file, Application, DEFAULT_CONFIG_FILE},
    docker_compose::Service,
    errors::Result,
};

use self::{
    build::Build, check::Check, dev::Dev, logs::Logs, new::New, reload::Reload, start::Start,
    stop::Stop, update::Update, upgrade::Upgrade,
};

mod build;
mod check;
mod dev;
mod logs;
mod new;
mod reload;
mod start;
mod stop;
pub mod terminal;
mod update;
mod upgrade;

/// The Lenra command line interface
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The app configuration file.
    #[clap(global=true, parse(from_os_str), long, default_value = DEFAULT_CONFIG_FILE)]
    pub config: std::path::PathBuf,

    /// Exposes services ports.
    #[clap(global=true, long, value_enum, default_values = &[], default_missing_values = &["app", "postgres", "mongo"])]
    pub expose: Vec<Service>,

    /// Run the commands as verbose.
    #[clap(global = true, short, long, action)]
    pub verbose: bool,
}

#[async_trait]
pub trait CliCommand {
    async fn run(&self, context: &mut CommandContext) -> Result<()>;
    fn need_config(&self) -> bool {
        true
    }
}

/// The subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Create a new Lenra app project from a template
    New(New),
    /// Build your app in release mode
    Build(Build),
    /// Start your app previously built with the build command
    Start(Start),
    /// View output from the containers
    Logs(Logs),
    /// Stop your app previously started with the start command
    Stop(Stop),
    /// Start the app in an interactive mode
    Dev(Dev),
    /// Upgrade the app with the last template updates
    Upgrade(Upgrade),
    /// Update the tools Docker images
    Update(Update),
    /// Checks the running app
    Check(Check),
    /// Reload the app by rebuilding and restarting it
    Reload(Reload),
}

#[async_trait]
impl CliCommand for Command {
    async fn run(&self, context: &mut CommandContext) -> Result<()> {
        log::debug!("Run command {:?}", self);
        if self.need_config() {
            context.load_config()?;
        }
        match self {
            Command::New(new) => new.run(context),
            Command::Build(build) => build.run(context),
            Command::Start(start) => start.run(context),
            Command::Logs(logs) => logs.run(context),
            Command::Stop(stop) => stop.run(context),
            Command::Dev(dev) => dev.run(context),
            Command::Upgrade(upgrade) => upgrade.run(context),
            Command::Update(update) => update.run(context),
            Command::Check(check) => check.run(context),
            Command::Reload(reload) => reload.run(context),
        }
        .await
    }

    fn need_config(&self) -> bool {
        match self {
            Command::New(_) => false,
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommandContext {
    /// The app configuration file.
    pub config_path: std::path::PathBuf,

    /// The app configuration.
    pub config: Option<Application>,

    /// Exposes all services ports.
    pub expose: Vec<Service>,

    /// Run command as verbose.
    pub verbose: bool,
}

impl CommandContext {
    pub fn load_config(&mut self) -> Result<Application> {
        debug!("Load config from {:?}", self.config_path);
        let app: Application = load_config_file(&self.config_path)?;
        self.config = Some(app.clone());
        Ok(app)
    }

    /// Resolve a path relative to the current directory and base on the path property of the config.
    pub fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        let mut resolved_path = self.get_app_workdir();
        resolved_path.push(path);
        debug!("Resolved path {:?} to {:?}", path, resolved_path);
        resolved_path
    }

    pub fn get_app_workdir(&self) -> PathBuf {
        let mut workdir = self.config_path.clone();
        workdir.pop();
        if let Some(app_dir_path) = self.get_app_path_config() {
            workdir.push(app_dir_path);
        };
        workdir
    }

    fn get_app_path_config(&self) -> Option<PathBuf> {
        self.config
            .as_ref()
            .map(|app| app.path.clone().unwrap_or(PathBuf::from(".")))
    }
}

pub async fn loader<F, Fut, R>(text: &str, success: &str, fail: &str, task: F) -> Result<R>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<R>>,
{
    let loading = Loading::default();
    loading.text(text);
    let res = task().await;
    if res.is_ok() {
        loading.success(success);
    } else {
        loading.fail(fail);
    }
    loading.end();
    res
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use clap::{CommandFactory, FromArgMatches};

    use super::*;

    pub fn parse_command_line(line: String) -> Result<Cli, clap::Error> {
        let args = &mut line.split_whitespace().collect::<Vec<&str>>();
        let command = <Cli as CommandFactory>::command();
        let mut matches = command
            .clone()
            .try_get_matches_from(args.clone())
            .map_err(format_error)?;
        <Cli as FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error)
    }

    fn format_error(err: clap::Error) -> clap::Error {
        let mut command = <Cli as CommandFactory>::command();
        err.format(&mut command)
    }

    #[test]
    fn test_load_config() {
        let mut context = CommandContext::default();
        context.config_path = PathBuf::from("test/config/app_path.yml");
        let app = context.load_config().unwrap();
        assert_eq!(app.path, Some(PathBuf::from("test_app")));
    }

    macro_rules! resolve_path_tests {
        ($($name:ident: $value:expr,)*) => {
        mod resolve_path {
            use std::path::PathBuf;
            use super::super::*;
        $(
            #[test]
            fn $name() {
                let (config_path, config, expected) = $value;
                let mut context = CommandContext::default();
                context.config_path = PathBuf::from(config_path);
                context.config = config;
                let path = PathBuf::from(".lenra/compose.yml");
                let resolved_path = context.resolve_path(&path);
                assert_eq!(
                    resolved_path,
                    PathBuf::from(expected)
                );
            }
        )*
        }
        }
    }

    resolve_path_tests! {
        simple: (
            "",
            Some(Application {..Default::default()}),
            "./.lenra/compose.yml"
        ),
        app_path: (
            "",
            Some(Application {path: Some(PathBuf::from("test_app")), ..Default::default()}),
            "test_app/.lenra/compose.yml"
        ),
        app_path_and_config_file: (
            "test/config/lenra.yml",
            Some(Application {path: Some(PathBuf::from("test_app")), ..Default::default()}),
            "test/config/test_app/.lenra/compose.yml"
        ),
        app_path_in_parent_dir: (
            "",
            Some(Application {path: Some(PathBuf::from("../test_app")), ..Default::default()}),
            "../test_app/.lenra/compose.yml"
        ),
        app_path_in_parent_dir_and_config_file: (
            "test/config/lenra.yml",
            Some(Application {path: Some(PathBuf::from("../test_app")), ..Default::default()}),
            "test/config/../test_app/.lenra/compose.yml"
        ),
    }

    macro_rules! get_app_workdir_tests {
        ($($name:ident: $value:expr,)*) => {
        mod get_app_workdir {
            use std::path::PathBuf;
            use super::super::*;
        $(
            #[test]
            fn $name() {
                let (config_path, config, expected) = $value;
                let mut context = CommandContext::default();
                context.config_path = PathBuf::from(config_path);
                context.config = config;
                let workdir = context.get_app_workdir();
                assert_eq!(
                    workdir,
                    PathBuf::from(expected)
                );
            }
        )*
        }
        }
    }

    get_app_workdir_tests! {
        simple: (
            "",
            Some(Application {..Default::default()}),
            "."
        ),
        app_path: (
            "",
            Some(Application {path: Some(PathBuf::from("test_app")), ..Default::default()}),
            "test_app"
        ),
        app_path_and_config_file: (
            "test/config/lenra.yml",
            Some(Application {path: Some(PathBuf::from("test_app")), ..Default::default()}),
            "test/config/test_app"
        ),
        app_path_in_parent_dir: (
            "",
            Some(Application {path: Some(PathBuf::from("../test_app")), ..Default::default()}),
            "../test_app"
        ),
        app_path_in_parent_dir_and_config_file: (
            "test/config/lenra.yml",
            Some(Application {path: Some(PathBuf::from("../test_app")), ..Default::default()}),
            "test/config/../test_app"
        ),
    }

    #[test]
    fn test_get_app_path_config() {
        let mut context = CommandContext::default();
        let app = Application {
            path: Some(PathBuf::from("test_app")),
            ..Default::default()
        };
        context.config = Some(app);
        let app_path = context.get_app_path_config().unwrap();
        assert_eq!(app_path, PathBuf::from("test_app"));
    }
}
