//! Text sanitising for the PDF/UA renderer.
//!
//! Audited pages contribute their own text to the report — link labels, element
//! selectors, custom property names. That text regularly carries icon-font
//! glyphs from a Private Use Area, which no real font maps: Typst falls back to
//! `LastResort` and the PDF/UA export then aborts the entire report over a
//! single glyph. Everything handed to the renderer therefore passes through
//! here first.
//!
//! Private Use codepoints are dropped up front — without the site's own icon
//! font they carry no meaning. Anything else the fonts happen to lack (an
//! arrow, a symbol) cannot be predicted from the codepoint alone, so those are
//! learned from the renderer: [`render_pdf_sanitized`] reads the character out
//! of the compile error, drops it and retries.
//!
//! This is presentation-only. The canonical JSON keeps the original text — a
//! CSS selector must stay copy-pasteable even when it contains an odd glyph.

use std::collections::BTreeSet;

use renderreport::{Engine, RenderRequest};
use serde_json::Value;

/// How often a render may be retried after dropping an undisplayable character.
const MAX_FONT_RETRIES: usize = 8;

/// Render to PDF, dropping characters the fonts cannot display as they surface.
pub(super) fn render_pdf_sanitized(
    engine: &Engine,
    request: &mut RenderRequest,
) -> anyhow::Result<Vec<u8>> {
    let mut undisplayable: BTreeSet<char> = BTreeSet::new();

    for _ in 0..=MAX_FONT_RETRIES {
        sanitize_request(request, &undisplayable);
        let error = match engine.render_pdf(request) {
            Ok(bytes) => return Ok(bytes),
            Err(e) => e,
        };
        let Some(ch) = undisplayable_char(&error.to_string()) else {
            return Err(error.into());
        };
        tracing::warn!(
            "Dropping U+{:04X} from the report: no available font can display it",
            ch as u32
        );
        undisplayable.insert(ch);
    }

    Err(anyhow::anyhow!(
        "PDF rendering still hit undisplayable characters after dropping {}",
        undisplayable
            .iter()
            .map(|c| format!("U+{:04X}", *c as u32))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// Strip undisplayable characters from every string the renderer will typeset.
pub(super) fn sanitize_request(request: &mut RenderRequest, undisplayable: &BTreeSet<char>) {
    if let Some(title) = request.title.as_mut() {
        sanitize_string(title, undisplayable);
    }
    if let Some(subtitle) = request.subtitle.as_mut() {
        sanitize_string(subtitle, undisplayable);
    }
    for value in request.metadata.values_mut() {
        sanitize_string(value, undisplayable);
    }
    for component in &mut request.components {
        sanitize_value(component, undisplayable);
    }
}

fn sanitize_value(value: &mut Value, undisplayable: &BTreeSet<char>) {
    match value {
        Value::String(text) => sanitize_string(text, undisplayable),
        Value::Array(items) => {
            for item in items {
                sanitize_value(item, undisplayable);
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                sanitize_value(item, undisplayable);
            }
        }
        _ => {}
    }
}

fn sanitize_string(text: &mut String, undisplayable: &BTreeSet<char>) {
    if !text
        .chars()
        .any(|c| is_private_use(c) || undisplayable.contains(&c))
    {
        return;
    }
    let stripped: String = text
        .chars()
        .filter(|c| !is_private_use(*c) && !undisplayable.contains(c))
        .collect();
    *text = collapse_whitespace(&stripped);
}

/// Private Use Areas: BMP plus the two supplementary planes.
fn is_private_use(c: char) -> bool {
    matches!(c as u32, 0xE000..=0xF8FF | 0xF_0000..=0xF_FFFD | 0x10_0000..=0x10_FFFD)
}

/// Removing a glyph leaves the gap it sat in — e.g. `Mehr erfahren \u{e900}`.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = false;
    for c in text.chars() {
        let is_space = c == ' ' || c == '\t';
        if is_space && last_was_space {
            continue;
        }
        last_was_space = is_space;
        out.push(c);
    }
    out.trim().to_string()
}

/// Pull the offending character out of a Typst font error.
///
/// Typst formats the text with `{:?}`, so a printable character arrives
/// literally (``the text `"↳"` ``) while anything else arrives as a Rust escape
/// (``the text `"\u{e900}"` ``). Both forms reach us wrapped in further layers
/// of `Debug` output, hence the tolerant parsing.
fn undisplayable_char(error: &str) -> Option<char> {
    const PREFIX: &str = "the text ";
    const SUFFIX: &str = " could not be displayed";

    let start = error.find(PREFIX)? + PREFIX.len();
    let end = start + error[start..].find(SUFFIX)?;
    let quoted = error[start..end].trim_matches(|c| c == '`' || c == '"' || c == '\\');

    if let Some(hex) = quoted
        .strip_prefix("u{")
        .or_else(|| quoted.strip_prefix("\\u{"))
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    quoted.chars().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_private_use_glyphs_and_the_gap_they_leave() {
        let mut text = "Mehr erfahren \u{e900}".to_string();
        sanitize_string(&mut text, &BTreeSet::new());
        assert_eq!(text, "Mehr erfahren");
    }

    #[test]
    fn keeps_text_without_private_use_glyphs_untouched() {
        let mut text = "Mehr erfahren über Ähnliches — 100 % ok".to_string();
        sanitize_string(&mut text, &BTreeSet::new());
        assert_eq!(text, "Mehr erfahren über Ähnliches — 100 % ok");
    }

    #[test]
    fn strips_characters_the_renderer_reported() {
        let mut text = "id=\"--\u{21b3}-Pictograms\"".to_string();
        sanitize_string(&mut text, &BTreeSet::from(['\u{21b3}']));
        assert_eq!(text, "id=\"---Pictograms\"");
    }

    #[test]
    fn sanitizes_nested_component_values() {
        let mut request = RenderRequest::new("wcag-audit");
        request.components.push(serde_json::json!({
            "type": "finding",
            "rows": [{ "label": "Link \u{f1ea}", "value": 3 }],
        }));
        sanitize_request(&mut request, &BTreeSet::new());
        assert_eq!(request.components[0]["rows"][0]["label"], "Link");
        assert_eq!(request.components[0]["rows"][0]["value"], 3);
    }

    #[test]
    fn reads_escaped_codepoint_from_render_error() {
        let error = "Typst compilation failed: SourceDiagnostic { message: \"PDF/UA-1 error: \
                     the text `\\\"\\\\u{e900}\\\"` could not be displayed with font \
                     `\\\"LastResort\\\"`\" }";
        assert_eq!(undisplayable_char(error), Some('\u{e900}'));
    }

    #[test]
    fn reads_literal_character_from_render_error() {
        let error = "PDF/UA-1 error: the text `\"↳\"` could not be displayed with font \
                     `\"LastResort\"`";
        assert_eq!(undisplayable_char(error), Some('\u{21b3}'));
    }

    #[test]
    fn ignores_unrelated_render_errors() {
        assert_eq!(
            undisplayable_char("Typst compilation failed: unknown variable"),
            None
        );
    }
}
