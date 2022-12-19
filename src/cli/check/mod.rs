use std::fmt::Debug;

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
                template_checker.check(params)
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

    /// The rules
    #[clap()]
    pub rules: Vec<String>,
}

pub trait AppChecker: Debug {
    fn check_list(&self) -> Vec<ValueChecker>;

    fn check(&self, params: CheckParameters) -> Result<()> {
        info!("Check with {:?}", self);
        // TODO: start app
        let check_list = self.check_list();

        debug!("Check list: {:?}", check_list);

        let mut fail: bool = false;
        check_list
            .iter()
            .filter(|checker| params.rules.is_empty() || params.rules.contains(&checker.name))
            .for_each(|checker| {
                let errors = checker.check(params.ignore.clone().unwrap_or(vec![]));
                let name = checker.name.clone();
                let mut messages: Vec<ColoredString> = vec![];

                let mut levels: Vec<CheckerLevel> = errors
                    .iter()
                    .map(|error| {
                        let lvl = match error.level {
                            RuleErrorLevel::Warning => CheckerLevel::Warning,
                            RuleErrorLevel::Error => CheckerLevel::Error,
                        };
                        messages.push(
                            format!("    {}\n        {}", error.rule, error.message)
                                .color(lvl.color()),
                        );
                        lvl
                    })
                    .collect();
                levels.sort();
                levels.reverse();

                let level: &CheckerLevel = levels.get(0).unwrap_or(&CheckerLevel::Ok);
                println!(
                    "{}",
                    format!("{:20}: {:?}", name, level).color(level.color())
                );
                messages.iter().for_each(|msg| println!("{}", msg));
                if level == &CheckerLevel::Error
                    || (level == &CheckerLevel::Warning && params.strict)
                {
                    fail = true;
                }
            });
        if fail {
            return Err(Error::CheckError);
        }
        Ok(())
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
                    .map(|err| match err.error_type.clone() {
                        MatchingErrorType::NotSameType { actual, expected } => RuleError {
                            rule: format!("{}{}{}", "sameType", RULE_SEPARATOR, err.path),
                            message: format!(
                                "Not matching type for {}: got {} but expected {}",
                                err.path,
                                actual.type_name(),
                                expected.type_name()
                            ),
                            level: RuleErrorLevel::Error,
                        },
                        MatchingErrorType::NotSameValue { actual, expected } => RuleError {
                            rule: format!("{}{}{}", "sameValue", RULE_SEPARATOR, err.path),
                            message: format!(
                                "Not matching value for {}: got {} but expected {}",
                                err.path, actual, expected
                            ),
                            level: RuleErrorLevel::Error,
                        },
                        MatchingErrorType::AdditionalProperty => RuleError {
                            rule: format!("{}{}{}", "additionalProperty", RULE_SEPARATOR, err.path),
                            message: format!("Additional property {}", err.path),
                            level: RuleErrorLevel::Warning,
                        },
                        MatchingErrorType::MissingProperty => RuleError {
                            rule: format!("{}{}{}", "missingProperty", RULE_SEPARATOR, err.path),
                            message: format!("Missing property {}", err.path),
                            level: RuleErrorLevel::Error,
                        },
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
                        .map(|error| RuleError {
                            rule: format!("{}{}{}", self.name, RULE_SEPARATOR, error.rule),
                            message: error.message.clone(),
                            level: error.level.clone(),
                        })
                        .filter(|error| {
                            !ignore_rule(
                                error
                                    .rule
                                    .split(RULE_SEPARATOR)
                                    .map(|str| str.into())
                                    .collect(),
                                ignores.clone(),
                            )
                        })
                        .collect()
                })
                .collect(),
            Err(err) => vec![RuleError {
                rule: format!("{}{}{}", self.name, RULE_SEPARATOR, "unexpectedError"),
                message: format!("Error loading {} checker data: {:?}", self.name, err),
                level: RuleErrorLevel::Error,
            }],
        }
    }
}

#[derive(Clone)]
enum MatchingErrorType {
    NotSameType { actual: Value, expected: Value },
    NotSameValue { actual: Value, expected: Value },
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
    fn type_name(&self) -> &str;
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
                error_type: MatchingErrorType::NotSameType {
                    actual: self.clone(),
                    expected: expected.clone(),
                },
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
            Value::Number(number) => {
                let result = if number.is_f64() || expected.is_f64() {
                    number.as_f64().unwrap().eq(&expected.as_f64().unwrap())
                } else if number.is_i64() || expected.is_i64() {
                    number.as_i64().unwrap().eq(&expected.as_i64().unwrap())
                } else {
                    number.as_u64().unwrap().eq(&expected.as_u64().unwrap())
                };

                if !result {
                    vec![MatchingError {
                        path: "".into(),
                        error_type: MatchingErrorType::NotSameValue {
                            actual: self.clone(),
                            expected: expected.clone(),
                        },
                    }]
                } else {
                    vec![]
                }
            }
            Value::Null => panic!("Should not be reached"),
            // Since equality have been tested before
            _ => vec![MatchingError {
                path: "".into(),
                error_type: MatchingErrorType::NotSameValue {
                    actual: self.clone(),
                    expected: expected.clone(),
                },
            }],
        }
    }

    fn type_name(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleError {
    rule: String,
    message: String,
    level: RuleErrorLevel,
}

#[derive(Debug, Clone)]
pub enum RuleErrorLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
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
