use serde_json::json;
use serde_json::Value;

use super::write;
use super::LINE_WIDTH;

fn format(
    value: &Value,
    stable: bool,
) -> String {
    let mut output = Vec::new();
    write(
        serde_json::to_vec(value).expect("JSON input"),
        stable,
        &mut output,
    )
    .expect("formatted JSON");
    String::from_utf8(output).expect("UTF-8 output")
}

fn fixtures() -> Vec<Value> {
    let controls: String = (0u8..=31).map(char::from).collect();
    vec![
        json!([]),
        json!([
            {"kind": "Comment", "lines": [controls, "Quotes: \" backslash: \\ Unicode: 🦀"]},
            {"kind": "Const", "name": "MAX", "valueLiteral": "18446744073709551615"},
            {"kind": "Struct", "name": "Z", "fields": [], "extra": {
                "null": null, "bool": true, "integer": u64::MAX,
                "negative": i64::MIN, "float": 1234.5, "object": {}, "array": []
            }},
            {"kind": "Function", "name": "callback", "returnType": {"kind": "bool"},
                "valueParameters": [{"name": "callback", "type": {
                    "kind": "Function", "returnType": {"kind": "u32"},
                    "valueParameters": [{"type": {"kind": "Optional", "type": {
                        "kind": "Pointer", "isMutable": false,
                        "type": {"kind": "Struct", "name": "A sufficiently long nested type name"}
                    }}}]
                }}]
            }
        ]),
    ]
}

#[test]
fn formatting_preserves_values_and_array_order() {
    for input in fixtures() {
        let output = format(&input, false);
        assert_eq!(
            serde_json::from_str::<Value>(&output).expect("JSON output"),
            input
        );
    }
}

#[test]
fn formatting_is_idempotent_with_and_without_sorting() {
    for input in fixtures() {
        for stable in [false, true] {
            let first = format(&input, stable);
            let parsed: Value = serde_json::from_str(&first).expect("JSON output");
            assert_eq!(format(&parsed, stable), first);
        }
    }
}

#[test]
fn stable_order_is_independent_of_declaration_discovery() {
    let first = json!([
        {"kind": "Struct", "name": "Z"},
        {"kind": "Function", "name": "b"},
        {"kind": "Comment", "lines": ["banner"]},
        {"kind": "Function", "name": "a"}
    ]);
    let mut reversed = first.as_array().expect("array").clone();
    reversed.reverse();
    let output = format(&first, true);
    assert_eq!(format(&Value::Array(reversed), true), output);
    let parsed: Vec<Value> = serde_json::from_str(&output).expect("JSON output");
    assert_eq!(
        parsed
            .iter()
            .map(|entry| entry["name"].as_str())
            .collect::<Vec<_>>(),
        [None, Some("a"), Some("b"), Some("Z")]
    );
}

#[test]
fn inline_width_includes_indentation_and_trailing_commas() {
    let prefix = "    \"type\": ";
    let overhead = "{\"kind\": \"Opaque\", \"name\": \"\"}".chars().count();
    for character in ['T', '🦀'] {
        for comma in [false, true] {
            let at_limit = LINE_WIDTH - prefix.len() - overhead - usize::from(comma);
            for extra in [0, 1] {
                let name: String = std::iter::repeat(character)
                    .take(at_limit + extra)
                    .collect();
                let mut declaration = json!({"kind": "Alias", "name": "A",
                    "type": {"kind": "Opaque", "name": name}});
                if comma {
                    declaration["valueLiteral"] = json!("irrelevant");
                }
                let output = format(&json!([declaration]), false);
                let line = output
                    .lines()
                    .find(|line| line.starts_with(prefix))
                    .expect("type line");
                if extra == 0 {
                    assert_eq!(line.chars().count(), LINE_WIDTH);
                    assert!(line.ends_with(if comma { "}," } else { "}" }));
                } else {
                    assert_eq!(line, "    \"type\": {");
                }
            }
        }
    }
}
