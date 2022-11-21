use std::{cmp::Ordering, fmt::Debug};

use clap::{Args, Subcommand};
use colored::{Color, ColoredString, Colorize};
use log::{debug, info};
use serde_json::Value;

use crate::errors::{Error, Result};

use self::template::TemplateChecker;

use super::CliCommand;

mod template;

pub const RULE_SEPARATOR: &str = ":";
pub const WIDGET: &str = "widget";

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
    fn check_list(&self) -> Vec<ValueChecker>;

    fn check(&self, params: CheckParameters) {
        info!("Check with {:?}", self);
        // TODO: start app
        let check_list = self.check_list();

        debug!("Check list: {:?}", check_list);

        let mut result: &CheckerLevel = &CheckerLevel::Ok;
        check_list.iter().for_each(|checker| {
            let errors = checker.check(params.ignore.clone().unwrap_or(vec![]));
            let name = checker.name.clone();
            let mut messages: Vec<ColoredString> = vec![];

            let mut levels: Vec<CheckerLevel> = errors
                .iter()
                .map(|error| match error {
                    RuleError::Warning(err) => {
                        messages.push(
                            format!("    {}\n        {}", err.rule, err.message).color(CheckerLevel::Warning.color()),
                        );
                        CheckerLevel::Warning
                    }
                    RuleError::Error(err) => {
                        messages.push(
                            format!("    {}\n        {}", err.rule, err.message).color(CheckerLevel::Error.color()),
                        );
                        CheckerLevel::Error
                    }
                })
                .collect();
            levels.sort();

            let level: &CheckerLevel = levels.get(0).unwrap_or(&CheckerLevel::Ok);
            println!(
                "{}",
                format!("{:20}: {:?}", name, level).color(level.color())
            );
            messages.iter().for_each(|msg| println!("{}", msg));
        });
    }
}

#[derive(Debug)]
pub struct Checker {
    name: String,
    action: fn() -> Result<Value>,
    rules: Vec<Rule<Value>>,
}

// impl Checker {
//     pub fn check(&self, params: CheckParameters) -> Vec<RuleError> {
//         let ignores = params.ignore.unwrap_or(vec![]);
//         if ignore_rule(vec![self.name.clone()], ignores.clone()) {
//             info!("Checker '{}' ignored", self.name);
//             return vec![];
//         }
//         let res = (self.action)();
//         match res {
//             Ok(value) => self
//                 .rules
//                 .iter()
//                 .flat_map(|rule| {
//                     if ignore_rule(vec![self.name.clone(), rule.name.clone()], ignores.clone()) {
//                         info!("Rule '{}' ignored for checker '{}'", rule.name, self.name);
//                         return vec![];
//                     }

//                     debug!("Check {}{}{}", self.name, RULE_SEPARATOR, rule.name);
//                     rule.check(value.clone(), value.clone())
//                 })
//                 .collect(),
//             Err(err) => vec![RuleError::Error(ErrorData {
//                 message: format!("Error loading {} checker data: {:?}", self.name, err)
//             })],
//         }
//     }
// }

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

#[derive(Debug)]
pub struct ValueChecker {
    name: String,
    expected: Value,
    loader: fn() -> Result<Value>,
}

impl ValueChecker {
    pub fn rules(&self) -> Vec<Rule<Value>> {
        vec![Rule {
            name: "match".into(),
            description: "Checks that the data matches the expected one".into(),
            examples: vec![],
            check: |value, expected| {
                value
                    .check_match(&expected)
                    .iter()
                    .map(|err| match err.error_type {
                        MatchingErrorType::NotSameType => RuleError::Error(ErrorData {
                            rule: format!("{}{}{}", "sameType", RULE_SEPARATOR, err.path),
                            message: format!("Not matching type for {}", err.path),
                        }),
                        MatchingErrorType::NotSameValue => RuleError::Error(ErrorData {
                            rule: format!("{}{}{}", "sameValue", RULE_SEPARATOR, err.path),
                            message: format!("Not matching value for {}", err.path),
                        }),
                        MatchingErrorType::AdditionalProperty => RuleError::Warning(ErrorData {
                            rule: format!("{}{}{}", "additionalProperty", RULE_SEPARATOR, err.path),
                            message: format!("Additional property {}", err.path),
                        }),
                        MatchingErrorType::MissingProperty => RuleError::Error(ErrorData {
                            rule: format!("{}{}{}", "missingProperty", RULE_SEPARATOR, err.path),
                            message: format!("Missing property {}", err.path),
                        }),
                    })
                    .collect()
            },
        }]
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
                        .iter()
                        .map(|error| match error {
                            RuleError::Warning(data) => RuleError::Warning(ErrorData {
                                rule: format!("{}{}{}", self.name, RULE_SEPARATOR, data.rule),
                                message: data.message.clone(),
                            }),
                            RuleError::Error(data) => RuleError::Error(ErrorData {
                                rule: format!("{}{}{}", self.name, RULE_SEPARATOR, data.rule),
                                message: data.message.clone(),
                            }),
                        })
                        .collect()
                })
                .collect(),
            Err(err) => vec![RuleError::Error(ErrorData {
                rule: format!("{}{}{}", self.name, RULE_SEPARATOR, "unexpectedError"),
                message: format!("Error loading {} checker data: {:?}", self.name, err),
            })],
        }
    }
}

#[derive(Clone)]
enum MatchingErrorType {
    NotSameType,
    NotSameValue,
    AdditionalProperty,
    MissingProperty,
}

#[derive(Clone)]
struct MatchingError {
    pub path: String,
    pub error_type: MatchingErrorType,
}

trait Matching {
    fn match_type(&self, val: &Value) -> bool;
    fn check_match(&self, expected: &Value) -> Vec<MatchingError>;
}

impl Matching for Value {
    fn match_type(&self, val: &Value) -> bool {
        match self {
            Value::Null => val.is_null(),
            Value::Bool(_) => val.is_boolean(),
            Value::Number(_) => val.is_number(),
            Value::String(_) => val.is_string(),
            Value::Array(_) => val.is_array(),
            Value::Object(_) => val.is_object(),
        }
    }

    fn check_match(&self, expected: &Value) -> Vec<MatchingError> {
        if expected == self {
            return vec![];
        }
        if !self.match_type(expected) {
            return vec![MatchingError {
                path: "".into(),
                error_type: MatchingErrorType::NotSameType,
            }];
        }

        match self {
            Value::Array(array) => {
                let expected_array = expected.as_array().unwrap();
                let mut ret: Vec<MatchingError> = vec![];
                let common_length = if array.len() > expected_array.len() {
                    expected_array.len()
                } else {
                    array.len()
                };

                for i in 0..common_length {
                    let v = array.get(i).unwrap();
                    let expected_v = expected_array.get(i).unwrap();
                    v.check_match(expected_v)
                        .iter()
                        .map(|error| MatchingError {
                            path: if error.path.is_empty() {
                                format!("{}", i)
                            } else {
                                format!("{}.{}", i, error.path)
                            },
                            error_type: error.error_type.clone(),
                        })
                        .for_each(|error| ret.push(error));
                }
                for i in common_length..array.len() {
                    ret.push(MatchingError {
                        path: format!("{}", i),
                        error_type: MatchingErrorType::AdditionalProperty,
                    });
                }
                for i in common_length..expected_array.len() {
                    ret.push(MatchingError {
                        path: format!("{}", i),
                        error_type: MatchingErrorType::MissingProperty,
                    });
                }

                ret
            }
            Value::Object(object) => {
                let expected_object = expected.as_object().unwrap();
                let keys = object.keys();
                let expected_keys = expected_object.keys();
                let mut ret: Vec<MatchingError> = vec![];

                expected_keys.for_each(|key| {
                    if object.contains_key(key) {
                        let value = object.get(key).unwrap();
                        let expected_value = expected_object.get(key).unwrap();
                        value
                            .check_match(expected_value)
                            .iter()
                            .map(|error| MatchingError {
                                path: if error.path.is_empty() {
                                    key.into()
                                } else {
                                    format!("{}.{}", key, error.path)
                                },
                                error_type: error.error_type.clone(),
                            })
                            .for_each(|error| ret.push(error));
                    } else {
                        ret.push(MatchingError {
                            path: key.into(),
                            error_type: MatchingErrorType::MissingProperty,
                        });
                    }
                });

                keys.for_each(|key| {
                    if !expected_object.contains_key(key) {
                        ret.push(MatchingError {
                            path: key.into(),
                            error_type: MatchingErrorType::AdditionalProperty,
                        });
                    }
                });

                ret
            }
            // Since equality have been tested before
            Value::Bool(_) => vec![MatchingError {
                path: "".into(),
                error_type: MatchingErrorType::NotSameValue,
            }],
            Value::Number(_) => vec![MatchingError {
                path: "".into(),
                error_type: MatchingErrorType::NotSameValue,
            }],
            Value::String(_) => vec![MatchingError {
                path: "".into(),
                error_type: MatchingErrorType::NotSameValue,
            }],
            _ => panic!("Should not be reached"),
        }
    }
}

#[derive(Debug)]
struct ErrorData {
    rule: String,
    message: String,
}

#[derive(Debug)]
pub enum RuleError {
    Warning(ErrorData),
    Error(ErrorData),
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
