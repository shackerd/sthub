use serde_json::{Map, Value};
use std::{collections::BTreeMap, env};

const SEPARATOR: &str = "__";

/// This module provides functionality to build a JSON tree from environment variables
/// that start with a specified prefix. The environment variables are expected to be
/// structured in a nested format using a double underscore (`__`) as a separator.
pub struct JsonEnvironmentVarsTree {
    prefix: String,
}

impl JsonEnvironmentVarsTree {
    /// Creates a new `Environment` instance with the specified prefix.
    pub fn new(prefix: &str) -> Self {
        if !prefix.ends_with(SEPARATOR) {
            panic!("Prefix must end with '{}'", SEPARATOR);
        }
        JsonEnvironmentVarsTree {
            prefix: prefix.to_string(),
        }
    }

    /// Builds a JSON tree from the environment variables that start with the specified prefix.
    pub fn build(&self) -> Value {
        let vars: Vec<(String, String)> = env::vars().collect();
        let mut root = BTreeMap::new();

        for (key, value) in vars {
            if let Some(stripped) = key.strip_prefix(self.prefix.as_str()) {
                let parts: Vec<&str> = stripped.split(SEPARATOR).collect();
                JsonEnvironmentVarsTree::insert_nested(&mut root, &parts, value);
            }
        }
        serde_json::to_value(root).unwrap()
    }

    /// Inserts a value into a nested BTreeMap structure based on the parts of the key.
    fn insert_nested(map: &mut BTreeMap<String, Value>, parts: &[&str], value: String) {
        if let Some((first, rest)) = parts.split_first() {
            if rest.is_empty() {
                map.insert(first.to_string(), Value::String(value));
            } else {
                let entry = map
                    .entry(first.to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));

                if let Value::Object(submap) = entry {
                    let mut nested: BTreeMap<String, Value> = submap
                        .iter_mut()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    JsonEnvironmentVarsTree::insert_nested(&mut nested, rest, value);

                    let mut converted = nested
                        .into_iter()
                        .map(|(k, v)| (k, v))
                        .collect::<Map<String, Value>>();

                    submap.append(&mut converted);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use std::env;
    #[test]
    fn test_environment_tree() {
        // Set some environment variables for testing
        unsafe { env::set_var("STHUB__TEST__NESTED__VAR", "value1") };
        unsafe { env::set_var("STHUB__TEST__VAR", "value2") };
        unsafe { env::set_var("STHUB__ANOTHER__VAR", "value3") };
        unsafe { env::set_var("STHUB__TEST__NESTED__ANOTHER__VAR", "value4") };
        unsafe { env::set_var("STHUB__TEST__NESTED__VAR2", "value5") };
        let environment = JsonEnvironmentVarsTree::new("STHUB__");
        let tree = environment.build();
        let expected = json!({
            "TEST": {
                "NESTED": {
                    "VAR": "value1",
                    "ANOTHER": {
                        "VAR": "value4"
                    },
                    "VAR2": "value5"
                },
                "VAR": "value2"
            },
            "ANOTHER": {
                "VAR": "value3"
            }
        });
        assert_eq!(tree, expected);
    }

    #[test]
    fn test_new_environment() {
        let prefix = "STHUB__";
        let env = JsonEnvironmentVarsTree::new(prefix);
        assert_eq!(env.prefix, prefix);
    }

    #[test]
    #[should_panic(expected = "Prefix must end with '__'")]
    fn test_new_environment_invalid_prefix() {
        let prefix = "STHUB";
        JsonEnvironmentVarsTree::new(prefix);
    }
}
