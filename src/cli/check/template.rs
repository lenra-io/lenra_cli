pub use clap::Args;
use colored::Colorize;

use serde_json::{Value, json};

use crate::{cli::check::Checker, errors::Error};

use super::{AppChecker, Rule, RuleError, call_app};

#[derive(Debug)]
pub struct TemplateChecker;

impl AppChecker for TemplateChecker {
    fn check_list(&self) -> Vec<Checker> {
        vec![Checker {
            name: "manifest".into(),
            action: || call_app(json!({})).map_err(Error::from),
            rules: vec![
                            Rule {
                                name: "rootWidget".into(),
                                description: "Checks that the 'rootWidget' property is defined in the manifest with 'main' as value".into(),
                                examples: vec![format!(r#"{{
  "{}": "{}"
}}"#, "rootWidget".underline().green(), "main".underline().green())],
                                check: |value| {
                                    println!("Check");
                                    if let Value::Object(object) = value.clone() {
                                        if let Some(val) = object.get("rootWidget") {
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
                                            vec![RuleError::Error(format!("The rootWidget field is not found in the manifest: {}", format!("{}", value).red()))]
                                        }
                                    }
                                    else {
                                        vec![RuleError::Error(format!("The manifest result is not an object: {}", format!("{}", value).red()))]
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
}
