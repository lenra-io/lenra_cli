pub use clap::Args;
use colored::Colorize;

use serde_json::{json, Value};

use crate::{cli::check::Checker, errors::Error};

use super::{call_app, AppChecker, Rule, RuleError};

#[derive(Debug)]
pub struct TemplateChecker;

impl AppChecker for TemplateChecker {
    fn check_list(&self) -> Vec<Checker> {
        vec![Checker {
            name: "manifest".into(),
            action: || call_app(json!({})).map_err(Error::from),
            rules: vec![
                Rule {
                    name: "additionalRootProperties".into(),
                    description: "Checks if the manifest response has additional properties".into(),
                    examples: vec![],
                    check: |value| {
                        let allowed_keys = vec!["manifest".to_string()];
                        if let Value::Object(object) = value.clone() {
                            let additionnal_keys: Vec<&String> = object.keys().into_iter().filter(|&key| !allowed_keys.contains(key)).collect();
                            if additionnal_keys.is_empty() {
                                vec![]
                            }
                            else {
                                additionnal_keys.iter().map(|key| RuleError::Warning(format!("The manifest response contains an unmanaged property: {}", format!("{}", key).yellow()))).collect()
                            }
                        }
                        else {
                            vec![RuleError::Error(format!("The manifest response is not an object: {}", format!("{}", value).red()))]
                        }
                    }
                },
                Rule {
                    name: "additionalManifestProperties".into(),
                    description: "Checks if the manifest field has additional properties".into(),
                    examples: vec![],
                    check: |value| {
                        let allowed_keys = vec!["rootWidget".to_string()];
                        if let Some(manifest) = value.get("manifest").and_then(|val| val.as_object()) {
                            let additionnal_keys: Vec<&String> = manifest.keys().into_iter().filter(|&key| !allowed_keys.contains(key)).collect();
                            if additionnal_keys.is_empty() {
                                vec![]
                            }
                            else {
                                additionnal_keys.iter().map(|key| RuleError::Warning(format!("The manifest field contains an unmanaged property: {}", format!("{}", key).yellow()))).collect()
                            }
                        }
                        else {
                            vec![RuleError::Error(format!("The manifest field is not found or is not an object: {}", format!("{}", value.to_string()).red()))]
                        }
                    }
                },
                Rule {
                    name: "rootWidget".into(),
                    description: "Checks that the 'rootWidget' property is defined in the manifest with 'main' as value".into(),
                    examples: vec![format!(r#"{{
"{}": "{}"
}}"#, "rootWidget".underline().green(), "main".underline().green())],
                    check: |value| {
                        if let Value::Object(object) = value.clone() {
                            if let Some(manifest) = object.get("manifest") {
                                if let Value::Object(manifest_object) = manifest.clone() {
                                    if let Some(val) = manifest_object.get("rootWidget") {
                                        match val {
                                            Value::String(widget_name) => {
                                                if widget_name=="main" {
                                                    vec![]
                                                }
                                                else {
                                                    vec![RuleError::Error(format!("Wrong root widget value: {}", widget_name.red()))]
                                                }
                                            },
                                            _ => vec![RuleError::Error(format!("The root widget value is not a string: {}", format!("{}", val).red()))],
                                        }
                                    }
                                    else {
                                        vec![RuleError::Error(format!("The rootWidget field is not found in the manifest: {}", format!("{}", manifest).red()))]
                                    }
                                }
                                else {
                                    vec![RuleError::Error(format!("The manifest field is not an object: {}", format!("{}", manifest).red()))]
                                }
                            }
                            else {
                                vec![RuleError::Error(format!("The manifest field is not found in the manifest: {}", format!("{}", value).red()))]
                            }
                        }
                        else {
                            vec![RuleError::Error(format!("The manifest response is not an object: {}", format!("{}", value).red()))]
                        }
                    }
                },
            ],
        }]
    }
}

mod test {
    use crate::cli::check::{template::TemplateChecker, AppChecker};

    #[test]
    fn check_list_size() {
        let template_checker = TemplateChecker;
        let check_list = template_checker.check_list();
        assert_eq!(
            check_list.len(),
            1,
            "The template checklist size is not correct"
        );
    }

    #[test]
    fn check_unique_names() {
        let template_checker = TemplateChecker;
        let check_list = template_checker.check_list();
        let mut checker_names: Vec<String> = vec![];
        check_list.iter().for_each(|checker| {
            assert!(
                !checker_names.contains(&checker.name),
                "There is at least two checkers with the same name: {}",
                checker.name
            );
            checker_names.push(checker.name.clone());
            let mut rule_names: Vec<String> = vec![];

            checker.rules.iter().for_each(|rule| {
                assert!(
                    !rule_names.contains(&rule.name),
                    "There is at least two rules with the same name in the '{}' checker: {}",
                    checker.name,
                    rule.name
                );
                rule_names.push(rule.name.clone());
            });
        });
    }
}
