pub use clap::Args;
use colored::Colorize;

use serde_json::Value;

use crate::cli::check::Checker;

use super::{AppChecker, Rule, RuleLevel};

#[derive(Debug)]
pub struct TemplateChecker;

impl AppChecker for TemplateChecker {
    fn check_list(&self) -> Vec<Checker> {
        vec![Checker {
            name: "manifest".into(),
            action: || {
                Value::Null
            },
            rules: vec![
                Rule {
                    name: "rootWidget".into(),
                    description: "Checks that the 'rootWidget' property is defined in the manifest with 'main' as value".into(),
                    examples: vec![format!(r#"{{
  "{}": "{}"
}}"#, "rootWidget".underline().green(), "main".underline().green())],
                    level: RuleLevel::Error,
                    check: |value| {
                        println!("Check");
                        if let Value::Object(object) = value {
                            // object
                            println!("Test de la valeur {:?}", object);
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
