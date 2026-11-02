#![cfg(feature = "headers")]

use std::fmt;
use std::fmt::Write as _;

use safer_ffi::headers::Language;
use safer_ffi::headers::{self};
use safer_ffi::prelude::*;
use serde_json::Value;

#[ffi_export]
pub const METADATA_TYPED: u32 = 4 * 1024;

#[ffi_export]
pub const METADATA_MAX: u64 = u64::MAX;

#[ffi_export]
pub const METADATA_MIN: i64 = i64::MIN;

#[ffi_export(untyped)]
pub const METADATA_TEXT: &str = "quote: \" slash: \\ newline: \n nul: \0 unicode: 🦀";

#[ffi_export(untyped)]
pub const METADATA_FLOAT: f64 = f64::INFINITY;

#[derive_ReprC]
#[repr(transparent)]
pub struct RawDebug {
    pub value: u8,
}

impl fmt::Debug for RawDebug {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for character in 0_u8..=31 {
            formatter.write_char(char::from(character))?;
        }
        formatter.write_str("\"\\🦀")
    }
}

#[ffi_export(untyped)]
pub const METADATA_RAW_DEBUG: RawDebug = RawDebug { value: 0 };

fn constant<'a>(
    entries: &'a [Value],
    name: &str,
) -> &'a Value {
    entries
        .iter()
        .find(|entry| entry["name"] == name)
        .expect("exported constant")
}

#[test]
fn metadata_preserves_constant_literals_and_explicit_type_information() {
    let mut output = Vec::new();
    headers::builder()
        .with_language(Language::Metadata)
        .to_writer(&mut output)
        .generate()
        .expect("metadata generation");
    let entries: Vec<Value> = serde_json::from_slice(&output).expect("valid JSON metadata");
    let typed = constant(&entries, "METADATA_TYPED");
    assert_eq!(typed["valueLiteral"], "4096");
    assert_eq!(typed["type"]["kind"], "u32");
    assert_eq!(typed["hasExplicitType"], true);
    assert_eq!(
        constant(&entries, "METADATA_MAX")["valueLiteral"],
        "18446744073709551615"
    );
    assert_eq!(
        constant(&entries, "METADATA_MIN")["valueLiteral"],
        "-9223372036854775808"
    );
    assert_eq!(constant(&entries, "METADATA_FLOAT")["valueLiteral"], "inf");
    let text = constant(&entries, "METADATA_TEXT");
    assert_eq!(text["valueLiteral"], format!("{METADATA_TEXT:?}"));
    assert_eq!(text["hasExplicitType"], false);
    let expected: String = (0_u8..=31)
        .map(char::from)
        .chain(['"', '\\', '🦀'])
        .collect();
    assert_eq!(
        constant(&entries, "METADATA_RAW_DEBUG")["valueLiteral"],
        expected
    );
}
