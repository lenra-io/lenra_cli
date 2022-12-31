use serde_yaml::Value;

#[derive(Clone)]
pub enum MatchingErrorType {
    NotSameType { actual: Value, expected: Value },
    NotSameValue { actual: Value, expected: Value },
    AdditionalProperty,
    MissingProperty,
}

#[derive(Clone)]
pub struct MatchingError {
    pub path: String,
    pub error_type: MatchingErrorType,
}

pub trait Matching {
    fn match_type(&self, val: &Value) -> bool;
    fn check_match(&self, expected: &Value) -> Vec<MatchingError>;
    fn type_name(&self) -> &str;
}

impl Matching for Value {
    fn match_type(&self, val: &Value) -> bool {
        match self {
            Value::Null => val.is_null(),
            Value::Bool(_) => val.is_bool(),
            Value::Number(_) => val.is_number(),
            Value::String(_) => val.is_string(),
            Value::Sequence(_) => val.is_sequence(),
            Value::Mapping(_) => val.is_mapping(),
            Value::Tagged(_) => false,
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
            Value::Sequence(array) => {
                let expected_array = expected.as_sequence().unwrap();
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
            Value::Mapping(object) => {
                let expected_object = expected.as_mapping().unwrap();
                let keys = object.keys();
                let expected_keys = expected_object.keys();
                let mut ret: Vec<MatchingError> = vec![];

                expected_keys.for_each(|key_value| {
                    let key = key_value.as_str().unwrap();
                    if object.contains_key(key_value) {
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

                keys.for_each(|key_value| {
                    let key = key_value.as_str().unwrap();
                    if !expected_object.contains_key(key_value) {
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
            Value::Sequence(_) => "array",
            Value::Mapping(_) => "object",
            Value::Tagged(_) => "tagged",
        }
    }
}
