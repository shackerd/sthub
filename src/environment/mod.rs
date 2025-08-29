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
    /// Creates a new `JsonEnvironmentVarsTree` instance with the specified prefix.
    pub fn new(prefix: &str) -> Self {
        if !prefix.ends_with(SEPARATOR) {
            panic!("Prefix must end with '{SEPARATOR}'");
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
        let json_value = serde_json::to_value(root).unwrap();
        JsonEnvironmentVarsTree::convert_objects_to_arrays(json_value)
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

                    let mut converted = nested.into_iter().collect::<Map<String, Value>>();

                    submap.append(&mut converted);
                }
            }
        }
    }

    /// Converts objects to arrays when all keys are numeric indices (0, 1, 2, ...)
    fn convert_objects_to_arrays(value: Value) -> Value {
        match value {
            Value::Object(obj) => {
                // First, recursively process all nested values
                let processed_obj: Map<String, Value> = obj
                    .into_iter()
                    .map(|(k, v)| (k, JsonEnvironmentVarsTree::convert_objects_to_arrays(v)))
                    .collect();

                // Check if all keys are numeric indices starting from 0
                let keys: Vec<&String> = processed_obj.keys().collect();
                let mut numeric_keys: Vec<usize> = Vec::new();

                for key in &keys {
                    if let Ok(index) = key.parse::<usize>() {
                        numeric_keys.push(index);
                    } else {
                        // If any key is not numeric, return as object
                        return Value::Object(processed_obj);
                    }
                }

                // Sort numeric keys to check for consecutive indices
                numeric_keys.sort();

                // Check if keys form a consecutive sequence starting from 0
                let is_array = !numeric_keys.is_empty()
                    && numeric_keys[0] == 0
                    && numeric_keys.windows(2).all(|w| w[1] == w[0] + 1);

                if is_array {
                    // Convert to array
                    let mut array: Vec<Value> = vec![Value::Null; numeric_keys.len()];
                    for (key, value) in processed_obj {
                        if let Ok(index) = key.parse::<usize>() {
                            array[index] = value;
                        }
                    }
                    Value::Array(array)
                } else {
                    Value::Object(processed_obj)
                }
            }
            Value::Array(arr) => {
                // Recursively process array elements
                Value::Array(
                    arr.into_iter()
                        .map(JsonEnvironmentVarsTree::convert_objects_to_arrays)
                        .collect(),
                )
            }
            _ => value,
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

        // Test array notation
        unsafe { env::set_var("STHUB__MYARRAY__0", "first") };
        unsafe { env::set_var("STHUB__MYARRAY__1", "second") };
        unsafe { env::set_var("STHUB__MYARRAY__2", "third") };

        // Test mixed notation (should remain as object)
        unsafe { env::set_var("STHUB__MIXED__0", "zero") };
        unsafe { env::set_var("STHUB__MIXED__NAME", "name_value") };
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
            },
            "MYARRAY": ["first", "second", "third"],
            "MIXED": {
                "0": "zero",
                "NAME": "name_value"
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

    #[test]
    fn test_array_notation() {
        // Test basic array
        unsafe { env::set_var("TEST_ARRAY__ITEMS__0", "item0") };
        unsafe { env::set_var("TEST_ARRAY__ITEMS__1", "item1") };
        unsafe { env::set_var("TEST_ARRAY__ITEMS__2", "item2") };

        let environment = JsonEnvironmentVarsTree::new("TEST_ARRAY__");
        let tree = environment.build();

        let expected = json!({
            "ITEMS": ["item0", "item1", "item2"]
        });

        assert_eq!(tree, expected);
    }

    #[test]
    fn test_mixed_notation() {
        // Test mixed array and object keys
        unsafe { env::set_var("TEST_MIXED__DATA__0", "zero") };
        unsafe { env::set_var("TEST_MIXED__DATA__1", "one") };
        unsafe { env::set_var("TEST_MIXED__DATA__NAME", "name_val") };

        let environment = JsonEnvironmentVarsTree::new("TEST_MIXED__");
        let tree = environment.build();

        let expected = json!({
            "DATA": {
                "0": "zero",
                "1": "one",
                "NAME": "name_val"
            }
        });

        assert_eq!(tree, expected);
    }

    #[test]
    fn test_non_consecutive_array() {
        // Test non-consecutive indices (should remain as object)
        unsafe { env::set_var("TEST_SPARSE__DATA__0", "zero") };
        unsafe { env::set_var("TEST_SPARSE__DATA__2", "two") };

        let environment = JsonEnvironmentVarsTree::new("TEST_SPARSE__");
        let tree = environment.build();

        let expected = json!({
            "DATA": {
                "0": "zero",
                "2": "two"
            }
        });

        assert_eq!(tree, expected);
    }
}
