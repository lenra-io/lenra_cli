pub use clap::Args;
use colored::Colorize;

use serde_json::{json, Value};

use crate::{cli::check::Checker, errors::Error};

use super::{call_app, AppChecker, Rule, RuleError, ValueChecker, RULE_SEPARATOR, WIDGET};

#[derive(Debug)]
pub struct TemplateChecker;

impl AppChecker for TemplateChecker {
    fn check_list(&self) -> Vec<ValueChecker> {
        vec![
            ValueChecker {
                name: "manifest".into(),
                loader: || call_app(json!({})).map_err(Error::from),
                expected: json!({
                    "manifest": {
                        "rootWidget": "main"
                    }
                }),
            },
            ValueChecker {
                name: format!("{}{}{}", WIDGET, RULE_SEPARATOR, "main"),
                loader: || {
                    call_app(json!({
                        "widget": "main",
                        "data": {},
                        "props": {},
                        "context": {}
                    }))
                    .map_err(Error::from)
                },
                expected: json!({
                  "type": "flex",
                  "direction": "vertical",
                  "scroll": true,
                  "spacing": 4,
                  "crossAxisAlignment": "center",
                  "children": [
                    {
                      "type": "widget",
                      "name": "menu",
                    },
                    {
                      "type": "widget",
                      "name": "home"
                    }
                  ]
                }),
            },
            ValueChecker {
                name: format!("{}{}{}", WIDGET, RULE_SEPARATOR, "menu"),
                loader: || {
                    call_app(json!({
                        "widget": "menu",
                        "data": {},
                        "props": {},
                        "context": {}
                    }))
                    .map_err(Error::from)
                },
                expected: json!({
                    "type": "container",
                    "decoration": {
                        "color": 0xFFFFFFFFu32,
                        "boxShadow": {
                            "blurRadius": 8,
                            "color": 0x1A000000,
                            "offset": {
                                "dx": 0,
                                "dy": 1
                            }
                        },
                    },
                    "padding": {
                        "top": 16,
                        "bottom": 16,
                        "left": 32,
                        "right": 32,
                    },
                    "child": {
                        "type": "flex",
                        "fillParent": true,
                        "mainAxisAlignment": "spaceBetween",
                        "crossAxisAlignment": "center",
                        "padding": { "right": 32, "top": 0, "bottom": 0, "left": 0 },
                        "children": [
                            {
                              "type": "container",
                              "constraints": {
                                "minWidth": 32,
                                "minHeight": 32,
                                "maxWidth": 32,
                                "maxHeight": 32,
                              },
                              "child": {
                                "type": "image",
                                "src": "logo.png"
                              },
                            },
                            {
                              "type": "flexible",
                              "child": {
                                "type": "container",
                                "child": {
                                  "type": "text",
                                  "value": "Hello World",
                                  "textAlign": "center",
                                  "style": {
                                    "fontWeight": "bold",
                                    "fontSize": 24,
                                  },
                                }
                              }
                            }
                        ]
                    },
                }),
            },
        ]
    }
}

mod test {
    use crate::cli::check::{template::TemplateChecker, AppChecker};

    // #[test]
    // fn check_list_size() {
    //     let template_checker = TemplateChecker;
    //     let check_list = template_checker.check_list();
    //     assert_eq!(
    //         check_list.len(),
    //         1,
    //         "The template checklist size is not correct"
    //     );
    // }

    // #[test]
    // fn check_unique_names() {
    //     let template_checker = TemplateChecker;
    //     let check_list = template_checker.check_list();
    //     let mut checker_names: Vec<String> = vec![];
    //     check_list.iter().for_each(|checker| {
    //         assert!(
    //             !checker_names.contains(&checker.name),
    //             "There is at least two checkers with the same name: {}",
    //             checker.name
    //         );
    //         checker_names.push(checker.name.clone());
    //         let mut rule_names: Vec<String> = vec![];

    //         checker.rules.iter().for_each(|rule| {
    //             assert!(
    //                 !rule_names.contains(&rule.name),
    //                 "There is at least two rules with the same name in the '{}' checker: {}",
    //                 checker.name,
    //                 rule.name
    //             );
    //             rule_names.push(rule.name.clone());
    //         });
    //     });
    // }
}
