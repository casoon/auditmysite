//! Guard of the PDF diagnosis section's texts (#936): no text of the export
//! scenarios is a key or still holds a placeholder. The guards of
//! `locales/{de,en}/diagnosis.ftl` itself live in
//! `output::builder::single::diagnosis`.

use super::text_export::records;
use serde_json::Value;

/// Every string in a record.
fn strings<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::String(text) => out.push(text),
        Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
        Value::Object(map) => map.values().for_each(|item| strings(item, out)),
        _ => {}
    }
}

#[test]
fn no_scenario_leaves_a_key_or_a_placeholder_in_a_text() {
    for record in records() {
        let mut texts = Vec::new();
        strings(&record, &mut texts);
        for text in texts {
            assert!(
                !text.contains("diagnosis-")
                    && !text.contains("severity-")
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}
