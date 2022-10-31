use std::fmt::Debug;

use clap::{Args, Subcommand};
use log::{debug, info};
use serde_json::Value;

use crate::errors::{Error, Result};

use self::template::TemplateChecker;

use super::CliCommand;

mod template;

const RULE_SEPARATOR: &str = ":";

#[derive(Args)]
pub struct Check {
    #[clap(subcommand)]
    command: CheckCommandType,
}

/// The check subcommands
#[derive(Subcommand, Clone)]
pub enum CheckCommandType {
    // /// Check the current project as an app
    // App(CheckParameters),
    /// Check the current project as a template
    Template(CheckParameters),
}

impl CliCommand for Check {
    fn run(&self) -> Result<()> {
        match self.command.clone() {
            CheckCommandType::Template(params) => {
                let template_checker = TemplateChecker;
                template_checker.check(params);
                Ok(())
            }
        }
    }
}

#[derive(Args, Default, Clone)]
pub struct CheckParameters {
    /// The strict mode also fails with warning rules.
    #[clap(long, action)]
    pub strict: bool,

    /// A list of rules to ignore
    #[clap(long)]
    pub ignore: Option<Vec<String>>,
}

pub trait AppChecker: Debug {
    fn check_list(&self) -> Vec<Checker>;

    fn check(&self, params: CheckParameters) {
        info!("Check with {:?}", self);
        // TODO: start app
        let check_list = self.check_list();

        debug!("Check list: {:?}", check_list);

        let mut fail: bool = false;
        check_list.iter().for_each(|checker| {
            let errors = checker.check(params.clone());
            // TODO: display ok if any no error
            // TODO: display errors
            println!("errors: {:?}", errors);
        });
    }
}

#[derive(Debug)]
pub struct Checker {
    name: String,
    action: fn() -> Result<Value>,
    rules: Vec<Rule<Value>>,
}

impl Checker {
    pub fn check(&self, params: CheckParameters) -> Vec<RuleError> {
        let ignores = params.ignore.unwrap_or(vec![]);
        if ignore_rule(vec![self.name.clone()], ignores.clone()) {
            info!("Checker '{}' ignored", self.name);
            return vec![];
        }
        let res = (self.action)();
        match res {
            Ok(value) => self
                .rules
                .iter()
                .flat_map(|rule| {
                    if ignore_rule(vec![self.name.clone(), rule.name.clone()], ignores.clone()) {
                        info!("Rule '{}' ignored for checker '{}'", rule.name, self.name);
                        return vec![];
                    }

                    debug!("Check {}{}{}", self.name, RULE_SEPARATOR, rule.name);
                    rule.check(value.clone())
                })
                .collect(),
            Err(err) => vec![RuleError::Error(format!(
                "Error loading {} checker data: {:?}",
                self.name, err
            ))],
        }
    }
}

fn ignore_rule(parts: Vec<String>, ignores: Vec<String>) -> bool {
    let mut prefix = String::new();
    for part in parts {
        prefix.push_str(part.as_str());
        if ignores.contains(&prefix) {
            return true;
        }
        prefix.push_str(RULE_SEPARATOR);
        if ignores.contains(&format!("{}*", prefix)) {
            return true;
        }
    }
    return false;
}

#[derive(Debug)]
pub enum RuleError {
    Warning(String),
    Error(String),
}

#[derive(Debug)]
pub struct Rule<T> {
    pub name: String,
    pub description: String,
    pub examples: Vec<String>,
    pub check: fn(T) -> Vec<RuleError>,
}

impl<T> Rule<T> {
    fn check(&self, param: T) -> Vec<RuleError> {
        (self.check)(param)
    }
}

pub fn call_app(request: Value) -> Result<Value> {
    ureq::post("http://localhost:8080")
        .send_json(request)
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)
}
