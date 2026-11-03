use std::io;
use std::io::Write;

use serde_json::Value;

const LINE_WIDTH: usize = 120;

#[cfg(test)]
mod tests;

pub(super) fn write(
    input: Vec<u8>,
    stable: bool,
    output: &mut dyn Write,
) -> io::Result<()> {
    let mut declarations: Vec<Value> = serde_json::from_slice(&input)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    drop(input);
    if stable {
        declarations.sort_by(|left, right| declaration_key(left).cmp(&declaration_key(right)));
    }
    if declarations.is_empty() {
        return output.write_all(b"[]\n");
    }
    output.write_all(b"[\n")?;
    let count = declarations.len();
    for (index, declaration) in declarations.into_iter().enumerate() {
        write!(output, "  {}", render(&declaration, 1, false, 2, false))?;
        output.write_all(if index + 1 < count { b",\n" } else { b"\n" })?;
    }
    output.write_all(b"]\n")
}

fn declaration_key(value: &Value) -> (bool, &str, &str) {
    let kind = value["kind"].as_str().unwrap_or("");
    (
        kind != "Comment",
        kind,
        value["name"].as_str().unwrap_or(""),
    )
}

fn fields(value: &serde_json::Map<String, Value>) -> Vec<(&str, &Value)> {
    let mut fields: Vec<_> = value
        .iter()
        .map(|(key, value)| (key.as_str(), value))
        .collect();
    fields.sort_by_key(|(key, _)| (!matches!(*key, "kind" | "name"), *key));
    fields
}

fn quoted(value: &str) -> String {
    Value::String(value.into()).to_string()
}

fn inline(value: &Value) -> String {
    match value {
        | Value::Object(object) => format!(
            "{{{}}}",
            fields(object)
                .into_iter()
                .map(|(key, value)| format!("{}: {}", quoted(key), inline(value)))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        | Value::Array(array) => {
            format!(
                "[{}]",
                array.iter().map(inline).collect::<Vec<_>>().join(", ")
            )
        },
        | _ => value.to_string(),
    }
}

fn render(
    value: &Value,
    depth: usize,
    type_context: bool,
    column: usize,
    comma: bool,
) -> String {
    if type_context {
        let candidate = inline(value);
        if column + candidate.chars().count() + usize::from(comma) <= LINE_WIDTH {
            return candidate;
        }
    }
    let prefix = "  ".repeat(depth + 1);
    let suffix = "  ".repeat(depth);
    match value {
        | Value::Object(object) if !object.is_empty() => {
            let fields = fields(object);
            let count = fields.len();
            let lines = fields
                .into_iter()
                .enumerate()
                .map(|(index, (key, child))| {
                    let label = format!("{prefix}{}: ", quoted(key));
                    let child = render(
                        child,
                        depth + 1,
                        type_context || matches!(key, "type" | "returnType" | "backingType"),
                        label.chars().count(),
                        index + 1 < count,
                    );
                    format!("{label}{child}")
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("{{\n{lines}\n{suffix}}}")
        },
        | Value::Array(array) if !array.is_empty() => {
            let count = array.len();
            let lines = array
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    let child = render(
                        child,
                        depth + 1,
                        type_context,
                        prefix.len(),
                        index + 1 < count,
                    );
                    format!("{prefix}{child}")
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("[\n{lines}\n{suffix}]")
        },
        | _ => inline(value),
    }
}
