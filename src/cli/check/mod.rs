use std::{cmp::Ordering, fmt::Debug};

use clap::{Args, Subcommand};
use colored::{Color, Colorize};
use log::{debug, info};
use serde_json::Value;

use crate::errors::{Error, Result};

use self::template::TemplateChecker;

use super::CliCommand;

mod template;

const RULE_SEPARATOR: &str = ":";

#[derive(Args, Clone)]
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

        let mut result: &CheckerLevel = &CheckerLevel::Ok;
        check_list.iter().for_each(|checker| {
            let errors = checker.check(params.clone());
            let name = checker.name.clone();

            let mut levels: Vec<CheckerLevel> = errors
                .iter()
                .map(|error| match error {
                    RuleError::Warning(msg) => {
                        println!("    {}", msg);
                        CheckerLevel::Warning
                    }
                    RuleError::Error(msg) => {
                        println!("    {}", msg);
                        CheckerLevel::Error
                    }
                })
                .collect();
            // levels.sort_by(|a, b| {
            //     if a == &CheckerLevel::Error {
            //         Ordering::Less
            //     } else if b == &CheckerLevel::Error {
            //         Ordering::Greater
            //     } else {
            //         Ordering::Equal
            //     }
            // });
            levels.sort();

            let level: &CheckerLevel = levels.get(0).unwrap_or(&CheckerLevel::Ok);
            // if level.cmp(&result) == Ordering::Less {
            //     result = level;
            // }
            println!(
                "{}",
                format!("{:20}: {:?}", name, level).color(level.color())
            );
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
                    rule.check(value.clone(), value.clone())
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckerLevel {
    Ok,
    Warning,
    Error,
}

impl CheckerLevel {
    fn color(&self) -> Color {
        match self {
            CheckerLevel::Ok => Color::Green,
            CheckerLevel::Warning => Color::Yellow,
            CheckerLevel::Error => Color::Red,
        }
    }
}

pub struct ValueChecker {
    name: String,
    expected: Value,
    loader: fn() -> Result<Value>,
}

impl ValueChecker {
    pub fn rules(&self) -> Vec<Rule<Value>> {
        vec![
            Rule {
                name: "additional-properties".into(),
                description: "Properties not expected in the result".into(),
                examples: vec![],
                check: |value, expected| {
                    check_additional_properties(value.clone(), expected.clone())
                },
            },
            Rule {
                name: "match".into(),
                description: "Checks that the date matches the expected one (ignoring addition properties)".into(),
                examples: vec![],
                check: |value, expected| {
                    check_additional_properties(value.clone(), expected.clone())
                },
            },
        ]
    }

    pub fn check(&self, ignores: Vec<String>) -> Vec<RuleError> {
        if ignore_rule(vec![self.name.clone()], ignores.clone()) {
            info!("Checker '{}' ignored", self.name);
            return vec![];
        }
        let res = (self.loader)();
        match res {
            Ok(value) => self
                .rules()
                .iter()
                .flat_map(|rule| {
                    if ignore_rule(vec![self.name.clone(), rule.name.clone()], ignores.clone()) {
                        info!("Rule '{}' ignored for checker '{}'", rule.name, self.name);
                        return vec![];
                    }

                    debug!("Check {}{}{}", self.name, RULE_SEPARATOR, rule.name);
                    rule.check(value.clone(), self.expected.clone())
                })
                .collect(),
            Err(err) => vec![RuleError::Error(format!(
                "Error loading {} checker data: {:?}",
                self.name, err
            ))],
        }
    }
}

enum MatchingErrorType {
    NotSameType,
    NotSameValue,
    AdditionnalProperty,
    MissingProperty,
}

struct MatchingError {
    pub path: String,
    pub error_type: MatchingErrorType,
}

trait Matching {
    fn match_type(&self, val: Value) -> bool;
    fn check_match(&self, expected: Value) -> Vec<MatchingError>;
}



impl Matching for Value {
    fn match_type(&self, val: Value) -> bool {
        match self {
            Value::Null => val.is_null(),
            Value::Bool(_) => val.is_boolean(),
            Value::Number(_) => val.is_number(),
            Value::String(_) => val.is_string(),
            Value::Array(_) => val.is_array(),
            Value::Object(_) => val.is_object(),
        }
    }

    fn check_match(&self, expected: Value) -> Vec<MatchingError> {
        if &expected == self {
            return vec![]
        }
        if !self.match_type(expected) {
            return vec![MatchingError { path: "".into(), error_type: MatchingErrorType::NotSameType }]
        }

        match self {
            Value::Array(array) => {
                let expected_array = expected.as_array().unwrap();
                let mut ret: Vec<MatchingError> = vec![];
                let common_length = if array.len()>expected_array.len() {
                    expected_array.len()
                }
                else {
                    array.len()
                };

                for i in 0..common_length {
                    let v = array.get(i).unwrap();
                    let expected_v = expected_array.get(i).unwrap();
                }
                ret
            },
            Value::Object(_) => val.is_object(),
            // Since equality have been tested before
            Value::Bool(_) => vec![MatchingError { path: "".into(), error_type: MatchingErrorType::NotSameValue }],
            Value::Number(_) => vec![MatchingError { path: "".into(), error_type: MatchingErrorType::NotSameValue }],
            Value::String(_) => vec![MatchingError { path: "".into(), error_type: MatchingErrorType::NotSameValue }],
            _ => panic!("Should not be reached"),
        }
    }
}

fn match_value(value: Value, expected: Value) -> Vec<RuleError> {
    if expected == value {
        return vec![]
    }
    else {
        value.
    }
    
    vec![]
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
    pub check: fn(T, T) -> Vec<RuleError>,
}

impl<T> Rule<T> {
    fn check(&self, param: T, expected: T) -> Vec<RuleError> {
        (self.check)(param, expected)
    }
}

pub fn call_app(request: Value) -> Result<Value> {
    ureq::post("http://localhost:8080")
        .send_json(request)
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)
}
