#![cfg(feature = "headers")]

use safer_ffi::headers::Language;
use safer_ffi::headers::{self};
use safer_ffi::prelude::*;
use serde_json::Value;
use serde_json::json;

#[ffi_export]
pub const ALIAS_METADATA_CONSTANT: u32 = 1;

/// A count with an exported C identity.
#[derive_ReprC(rename = "Count")]
#[repr(transparent)]
pub struct Count {
    pub value: u32,
}

#[derive_ReprC(rename = "OtherCount")]
#[repr(transparent)]
pub struct OtherCount {
    pub value: u32,
}

#[derive_ReprC(rename = "ChainedCount")]
#[repr(transparent)]
pub struct ChainedCount {
    pub value: Count,
}

#[derive_ReprC(rename = "GenericCount")]
#[repr(transparent)]
pub struct GenericCount<T: ReprC> {
    pub value: T,
}

#[derive_ReprC]
#[repr(C)]
pub struct AliasFields {
    pub count: Count,
    pub other: OtherCount,
}

#[ffi_export]
pub fn metadata_alias_roundtrip(
    value: Count,
    other: OtherCount,
    chained: ChainedCount,
    pointer: *const Count,
    mutable: *mut OtherCount,
    optional: Option<repr_c::Box<Count>>,
    array: [Count; 3],
    slice: c_slice::Ref<'_, Count>,
    boxed: c_slice::Box<Count>,
    vector: repr_c::Vec<Count>,
    callback: Option<extern "C" fn(Count) -> OtherCount>,
    fields: AliasFields,
    generic: GenericCount<u32>,
) -> Count {
    let _ = (
        other, chained, pointer, mutable, optional, array, slice, boxed, vector, callback, fields,
    );
    value
}

fn generate(language: Language) -> String {
    let mut output = Vec::new();
    headers::builder()
        .with_language(language)
        .to_writer(&mut output)
        .generate()
        .expect("header generation");
    String::from_utf8(output).expect("UTF-8 header")
}

fn entry<'a>(
    entries: &'a [Value],
    kind: &str,
    name: &str,
) -> &'a Value {
    entries
        .iter()
        .find(|entry| entry["kind"] == kind && entry["name"] == name)
        .expect("exported declaration")
}

fn alias_names(value: &Value) -> Vec<&str> {
    let mut names = Vec::new();
    if let Some(object) = value.as_object() {
        if object.get("kind").and_then(Value::as_str) == Some("Alias") {
            names.push(object["name"].as_str().expect("alias name"));
        }
        for (key, child) in object {
            if key != "name" && key != "comment" {
                names.extend(alias_names(child));
            }
        }
    } else if let Some(array) = value.as_array() {
        for child in array {
            names.extend(alias_names(child));
        }
    }
    names
}

#[test]
fn alias_metadata_preserves_definitions_and_every_nested_use_site() {
    let entries: Vec<Value> =
        serde_json::from_str(&generate(Language::Metadata)).expect("JSON metadata");
    assert_eq!(
        entry(&entries, "Alias", "Count")["type"],
        json!({"kind": "u32"})
    );
    assert_eq!(
        entry(&entries, "Alias", "OtherCount")["type"],
        json!({"kind": "u32"})
    );
    assert_eq!(
        entry(&entries, "Alias", "ChainedCount")["type"],
        json!({"kind": "Alias", "name": "Count"})
    );
    assert_eq!(
        entry(&entries, "Alias", "Count")["comment"]["lines"][0],
        "A count with an exported C identity."
    );
    let function = entry(&entries, "Function", "metadata_alias_roundtrip");
    for parameter in function["valueParameters"].as_array().expect("parameters") {
        let name = parameter["name"].as_str().expect("parameter name");
        if name != "fields" {
            assert!(
                !alias_names(&parameter["type"]).is_empty(),
                "alias erased from {name}"
            );
        }
    }
    let callback = &function["valueParameters"][10]["type"];
    assert_eq!(alias_names(callback), ["OtherCount", "Count"]);
    assert_eq!(
        function["returnType"],
        json!({"kind": "Alias", "name": "Count"})
    );
    let fields = entry(&entries, "Struct", "AliasFields");
    assert_eq!(alias_names(&fields["fields"][0]["type"]), ["Count"]);
    assert_eq!(alias_names(&fields["fields"][1]["type"]), ["OtherCount"]);
}

#[test]
fn c_and_csharp_keep_their_existing_alias_handling() {
    assert!(generate(Language::C).contains("typedef uint32_t Count_t;"));
    assert!(generate(Language::CSharp).contains("UInt32 metadata_alias_roundtrip"));
}

fn resolve_aliases(
    value: &Value,
    aliases: &std::collections::BTreeMap<String, Value>,
) -> Value {
    match value {
        | Value::Object(object) if object.get("kind").and_then(Value::as_str) == Some("Alias") => {
            let name = object["name"].as_str().expect("alias name");
            resolve_aliases(&aliases[name], aliases)
        },
        | Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, child)| (key.clone(), resolve_aliases(child, aliases)))
                .collect(),
        ),
        | Value::Array(array) => Value::Array(
            array
                .iter()
                .map(|child| resolve_aliases(child, aliases))
                .collect(),
        ),
        | _ => value.clone(),
    }
}

#[test]
fn consumers_can_resolve_aliases_to_their_underlying_types() {
    let preserved: Vec<Value> =
        serde_json::from_str(&generate(Language::Metadata)).expect("alias JSON");
    let aliases = preserved
        .iter()
        .filter(|entry| entry["kind"] == "Alias")
        .map(|entry| {
            (
                entry["name"].as_str().expect("name").to_owned(),
                entry["type"].clone(),
            )
        })
        .collect();
    let resolved: Vec<Value> = preserved
        .iter()
        .filter(|entry| entry["kind"] != "Alias")
        .map(|entry| resolve_aliases(entry, &aliases))
        .collect();
    assert!(alias_names(&Value::Array(resolved.clone())).is_empty());
    let function = entry(&resolved, "Function", "metadata_alias_roundtrip");
    for parameter in &function["valueParameters"].as_array().expect("parameters")[..3] {
        assert_eq!(parameter["type"], json!({"kind": "u32"}));
    }
    assert_eq!(function["returnType"], json!({"kind": "u32"}));
    let fields = entry(&resolved, "Struct", "AliasFields");
    for field in fields["fields"].as_array().expect("fields") {
        assert_eq!(field["type"], json!({"kind": "u32"}));
    }
}
