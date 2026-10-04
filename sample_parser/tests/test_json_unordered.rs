mod common_lark_utils;
use common_lark_utils::{json_err_test, lark_str_test_many};
use serde_json::{json, Value};

fn unordered(mut schema: Value) -> String {
    schema["x-guidance"] = json!({ "unordered_properties": true });
    format!("start: %json {schema}")
}

#[test]
fn test_unordered_required_properties_in_any_order() {
    let lark = unordered(json!({
        "type": "object",
        "properties": {
            "answer": { "type": "boolean" },
            "confidence": { "type": "number" },
            "reason": { "type": "string" }
        },
        "required": ["answer", "confidence", "reason"],
        "additionalProperties": false
    }));
    lark_str_test_many(
        &lark,
        &[
            r#"{"answer": true, "confidence": 0.5, "reason": "r"}"#,
            r#"{"reason": "r", "answer": false, "confidence": 1}"#,
            r#"{"confidence": 0, "reason": "r", "answer": true}"#,
            r#"{"reason": "r", "confidence": 1, "answer": true}"#,
        ],
        &[
            // a required property missing
            r#"{"reason": "r", "answer": true}"#,
            // a property twice
            r#"{"answer": true, "answer": false, "confidence": 1, "reason": "r"}"#,
            // a property the schema does not name
            r#"{"answer": true, "confidence": 1, "reason": "r", "extra": 1}"#,
            // a value of the wrong type is still refused, in any position
            r#"{"reason": 1, "answer": true, "confidence": 1}"#,
        ],
    );
}

#[test]
fn test_unordered_optional_properties() {
    let lark = unordered(json!({
        "type": "object",
        "properties": {
            "a": { "type": "integer" },
            "b": { "type": "integer" },
            "c": { "type": "integer" }
        },
        "required": ["b"],
        "additionalProperties": false
    }));
    lark_str_test_many(
        &lark,
        &[
            r#"{"b": 1}"#,
            r#"{"c": 3, "b": 1}"#,
            r#"{"b": 1, "a": 2}"#,
            r#"{"c": 3, "a": 2, "b": 1}"#,
        ],
        &[r#"{}"#, r#"{"a": 2}"#, r#"{"c": 3, "a": 2}"#, r#"{"b": 1, "b": 1}"#],
    );
}

#[test]
fn test_unordered_additional_properties_follow_named() {
    let lark = unordered(json!({
        "type": "object",
        "properties": {
            "a": { "type": "integer" },
            "b": { "type": "integer" }
        },
        "required": ["a", "b"],
        "additionalProperties": { "type": "string" }
    }));
    lark_str_test_many(
        &lark,
        &[
            r#"{"b": 2, "a": 1}"#,
            r#"{"b": 2, "a": 1, "x": "y"}"#,
            r#"{"a": 1, "b": 2, "x": "y", "z": "w"}"#,
        ],
        &[
            // a named key cannot appear again as an additional one
            r#"{"a": 1, "b": 2, "a": "y"}"#,
            // additional entries follow the named ones
            r#"{"x": "y", "a": 1, "b": 2}"#,
        ],
    );
}

#[test]
fn test_unordered_nested_objects() {
    let lark = unordered(json!({
        "type": "object",
        "properties": {
            "results": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "res": { "type": "integer" },
                        "reason": { "type": "string" }
                    },
                    "required": ["res", "reason"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["results"],
        "additionalProperties": false
    }));
    lark_str_test_many(
        &lark,
        &[r#"{"results": [{"reason": "r", "res": 1}, {"res": 0, "reason": "s"}]}"#],
        &[r#"{"results": [{"reason": "r"}]}"#],
    );
}

#[test]
fn test_ordered_by_default() {
    let schema = json!({
        "type": "object",
        "properties": {
            "a": { "type": "integer" },
            "b": { "type": "integer" }
        },
        "required": ["a", "b"],
        "additionalProperties": false
    });
    lark_str_test_many(
        &format!("start: %json {schema}"),
        &[r#"{"a": 1, "b": 2}"#],
        &[r#"{"b": 2, "a": 1}"#],
    );
}

#[test]
fn test_unordered_refuses_an_object_too_wide() {
    let properties: serde_json::Map<String, Value> = (0..=llguidance::MAX_UNORDERED_PROPERTIES)
        .map(|i| (format!("p{i}"), json!({ "type": "integer" })))
        .collect();
    json_err_test(
        &json!({
            "type": "object",
            "properties": properties,
            "x-guidance": { "unordered_properties": true }
        }),
        "with unordered_properties at most 12 are supported",
    );
}

#[test]
fn test_unordered_at_the_cap() {
    let n = llguidance::MAX_UNORDERED_PROPERTIES;
    let properties: serde_json::Map<String, Value> = (0..n)
        .map(|i| (format!("p{i}"), json!({ "type": "integer" })))
        .collect();
    let required: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
    let lark = unordered(json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    }));
    let reversed = (0..n)
        .rev()
        .map(|i| format!("\"p{i}\": {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let missing_one = (1..n)
        .map(|i| format!("\"p{i}\": {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    lark_str_test_many(
        &lark,
        &[&format!("{{{reversed}}}")],
        &[&format!("{{{missing_one}}}")],
    );
}
