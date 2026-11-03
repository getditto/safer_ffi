#![cfg(feature = "headers")]

use std::collections::BTreeSet;
use std::io;
use std::io::Write;

use safer_ffi::headers::Definer;
use safer_ffi::headers::Language;
use safer_ffi::headers::{self};
use safer_ffi::prelude::*;
use serde_json::Value;

#[derive_ReprC]
#[repr(transparent)]
pub struct MetadataCount {
    pub value: u32,
}

#[derive_ReprC]
#[repr(C)]
#[doc = "Quotes: \" braces: {} backslash: \\ tab: \t Unicode: 🦀"]
pub struct MetadataZ {
    pub count: MetadataCount,
}

#[derive_ReprC]
#[repr(C)]
pub struct MetadataA {
    pub count: MetadataCount,
}

#[ffi_export]
pub const METADATA_OUTPUT_LIMIT: u32 = 4096;

#[ffi_export]
pub fn metadata_a_use_z(value: MetadataZ) -> u32 {
    value.count.value
}

#[ffi_export]
pub fn metadata_z_use_a(value: MetadataA) -> u32 {
    value.count.value
}

#[ffi_export]
pub fn metadata_callback(
    callback: Option<extern "C" fn(*const MetadataZ, *mut MetadataA, u64, f64, bool) -> bool>
) {
    let _ = callback;
}

fn generate(
    language: Language,
    stable: bool,
) -> String {
    let mut output = Vec::new();
    headers::builder()
        .with_language(language)
        .with_stable_header(stable)
        .to_writer(&mut output)
        .generate()
        .expect("metadata generation");
    String::from_utf8(output).expect("UTF-8 output")
}

fn declarations(text: &str) -> Vec<Value> {
    serde_json::from_str(text).expect("valid JSON metadata")
}

fn key(value: &Value) -> (bool, &str, &str) {
    let kind = value["kind"].as_str().expect("declaration kind");
    (
        kind != "Comment",
        kind,
        value["name"].as_str().unwrap_or(""),
    )
}

#[test]
fn stable_metadata_sorts_complete_declarations_and_preserves_streaming_contents() {
    let stable = declarations(&generate(Language::Metadata, true));
    assert!(stable.windows(2).all(|pair| key(&pair[0]) <= key(&pair[1])));
    let mut streaming = declarations(&generate(Language::Metadata, false));
    let position = |kind, name| {
        streaming
            .iter()
            .position(|entry| entry["kind"] == kind && entry["name"] == name)
            .expect("declaration")
    };
    assert!(position("Struct", "MetadataZ") < position("Function", "metadata_a_use_z"));
    streaming.sort_by(|left, right| key(left).cmp(&key(right)));
    assert_eq!(stable, streaming);
}

#[test]
fn metadata_is_readable_and_preserves_comment_text() {
    let output = generate(Language::Metadata, true);
    let entries = declarations(&output);
    let z = entries
        .iter()
        .find(|entry| entry["name"] == "MetadataZ")
        .expect("struct");
    assert_eq!(
        z["comment"]["lines"][0],
        "Quotes: \" braces: {} backslash: \\ tab: \t Unicode: 🦀"
    );
    assert!(output.starts_with("[\n  {\n    \"kind\": \"Comment\","));
    assert!(output.contains("\"type\": {\"kind\": \"u32\"}"));
    assert!(output.contains("\"type\": {\n"));
    for line in output.lines().filter(|line| line.chars().count() > 120) {
        assert!(line.trim().starts_with('"') && line.trim().trim_end_matches(',').ends_with('"'));
    }
}

#[test]
fn c_headers_keep_types_before_their_first_use() {
    let output = generate(Language::C, true);
    assert!(
        output.find("MetadataZ_t").expect("type")
            < output.find("metadata_a_use_z").expect("function")
    );
}

#[derive(Default)]
struct TrackingDefiner<W> {
    names: BTreeSet<String>,
    output: W,
}

impl<W: Write> Definer for TrackingDefiner<W> {
    fn insert(
        &mut self,
        name: &str,
    ) -> bool {
        self.names.insert(name.into())
    }

    fn out(&mut self) -> &mut dyn Write {
        &mut self.output
    }
}

#[test]
fn metadata_preserves_custom_definer_state() {
    let mut definer = TrackingDefiner::default();
    headers::builder()
        .with_language(Language::Metadata)
        .generate_with_definer(&mut definer)
        .expect("metadata generation");
    assert!(definer.names.contains("metadata_a_use_z"));
    assert!(!declarations(&String::from_utf8(definer.output).expect("UTF-8")).is_empty());
}

#[test]
fn custom_definers_match_default_generation() {
    for language in [
        Language::C,
        Language::CSharp,
        Language::Lua,
        Language::Metadata,
    ] {
        for stable in [false, true] {
            let mut output = Vec::new();
            let mut names = BTreeSet::new();
            language
                .with_output(&mut output, stable, |out| {
                    let mut definer = TrackingDefiner {
                        names: BTreeSet::new(),
                        output: out,
                    };
                    headers::builder()
                        .with_language(language)
                        .with_stable_header(stable)
                        .generate_with_definer(&mut definer)?;
                    names = definer.names;
                    Ok(())
                })
                .expect("header generation");
            assert!(names.contains("metadata_a_use_z"));
            assert_eq!(
                String::from_utf8(output).expect("UTF-8"),
                generate(language, stable),
                "{language:?}, stable={stable}",
            );
        }
    }
}

#[derive(Default)]
struct TracingDefiner {
    inner: TrackingDefiner<Vec<u8>>,
    offsets: Vec<(usize, String)>,
}

impl Definer for TracingDefiner {
    fn insert(
        &mut self,
        name: &str,
    ) -> bool {
        if !self.inner.insert(name) {
            return false;
        }
        let marker = format!("/* defining {name} */\n");
        self.offsets.push((self.inner.output.len(), marker.clone()));
        self.inner.output.extend_from_slice(marker.as_bytes());
        true
    }

    fn out(&mut self) -> &mut dyn Write {
        self.inner.out()
    }
}

#[test]
fn custom_definer_writes_and_offsets_follow_generation() {
    for language in [Language::C, Language::CSharp, Language::Lua] {
        let mut definer = TracingDefiner::default();
        headers::builder()
            .with_language(language)
            .generate_with_definer(&mut definer)
            .expect("header generation");
        let output = String::from_utf8(definer.inner.output).expect("UTF-8");
        assert_eq!(
            output.lines().next(),
            generate(language, true).lines().next()
        );
        assert!(!definer.offsets.is_empty());
        for (offset, marker) in definer.offsets {
            assert!(offset > 0);
            assert!(output[offset..].starts_with(&marker));
        }
    }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(
        &mut self,
        _: &[u8],
    ) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn generation_propagates_output_errors() {
    for language in [
        Language::C,
        Language::CSharp,
        Language::Lua,
        Language::Metadata,
    ] {
        let error = headers::builder()
            .with_language(language)
            .to_writer(FailingWriter)
            .generate()
            .expect_err("writer fails");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }
}

#[test]
fn streaming_output_errors_stop_before_definitions_are_generated() {
    for language in [Language::C, Language::CSharp, Language::Lua] {
        let mut names = BTreeSet::new();
        let error = language
            .with_output(&mut FailingWriter, true, |out| {
                let mut definer = TrackingDefiner {
                    names: BTreeSet::new(),
                    output: out,
                };
                let result = headers::builder()
                    .with_language(language)
                    .generate_with_definer(&mut definer);
                names = definer.names;
                result
            })
            .expect_err("writer fails");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert!(
            names.is_empty(),
            "{language:?} buffered generation before writing"
        );
    }
}

struct FlushRejectingWriter<'a>(&'a mut Vec<u8>);

impl Write for FlushRejectingWriter<'_> {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> io::Result<usize> {
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Other))
    }
}

#[test]
fn generation_leaves_flushing_the_destination_to_the_caller() {
    for language in [
        Language::C,
        Language::CSharp,
        Language::Lua,
        Language::Metadata,
    ] {
        let mut output = Vec::new();
        headers::builder()
            .with_language(language)
            .to_writer(FlushRejectingWriter(&mut output))
            .generate()
            .expect("generation writes without flushing the caller's destination");
        assert_eq!(
            String::from_utf8(output).expect("UTF-8"),
            generate(language, true)
        );
    }
}
