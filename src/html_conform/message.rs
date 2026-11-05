//! Run-language text of `html-conform` findings (#902, #406).
//!
//! The checker's messages are third-party English prose with parser-internal
//! noise (for example an "unexpected element" message that names the XHTML
//! namespace URI of `span` and a line and column position). The stored message stays exactly that (canonical English in
//! the JSON). A German report shows [`message_text`] instead: the most common
//! messages are mapped by rule id and shape, every other one falls back to the
//! localized title of the rule's family (`recommendation.rs`), so no English
//! checker prose reaches a German report.
//!
//! The wording of the shared checker itself belongs to barrierlab
//! (`html-conform`); this module only maps, it never rewrites its output.

use crate::html_conform::recommendation;

/// The message of a finding in the requested language. `en = true` returns the
/// stored canonical English unchanged.
pub fn message_text(rule_id: &str, message: &str, en: bool) -> String {
    if en {
        return message.to_string();
    }
    let bare = recommendation::bare_id(rule_id);
    let body = strip_position(message).0;

    if bare == "schema.html5" {
        if let Some(name) = quoted_after(body, "unexpected element ") {
            return format!(
                "Unerwartetes Element <{}> an dieser Stelle.",
                local_name(&name)
            );
        }
        if let Some(name) = quoted_after(body, "unexpected attribute ") {
            return format!("Unerwartetes Attribut „{}“.", local_name(&name));
        }
        if let Some(rest) = body.strip_prefix("invalid value ") {
            if let Some((value, attr)) = rest.split_once(" for attribute ") {
                return format!(
                    "Ungültiger Wert „{}“ für das Attribut „{}“.",
                    unquote(value),
                    unquote(attr)
                );
            }
        }
    }
    if let Some(role) = bare.strip_prefix("roles.unnecessary-") {
        return format!("Die Rolle „{role}“ ist für dieses Element überflüssig.");
    }
    if bare == "headings.section-lacks-heading" {
        return "Abschnitt ohne Überschrift: Eine Überschrift (h2 bis h6) ergänzen oder ein div \
                verwenden, wenn keine Überschrift nötig ist."
            .to_string();
    }
    family_title(rule_id)
}

/// The localized title of the rule's family, the fallback for messages without
/// a mapping. Never English checker prose.
fn family_title(rule_id: &str) -> String {
    let family = recommendation::family(rule_id).unwrap_or("generic");
    crate::i18n::text(&format!("html-conform-{family}-title"), false)
}

/// The message split into its text and the `line:column` it ends with
/// (` at 190:12271`), when it carries one.
pub fn strip_position(message: &str) -> (&str, Option<(u32, u32)>) {
    if let Some((body, tail)) = message.rsplit_once(" at ") {
        if let Some((line, col)) = tail.trim().split_once(':') {
            if let (Ok(line), Ok(col)) = (line.parse(), col.parse()) {
                return (body, Some((line, col)));
            }
        }
    }
    (message, None)
}

/// The element a message names (`unexpected element `{ns}span``), without the
/// namespace. `None` for messages that name no element.
pub fn element_name(message: &str) -> Option<String> {
    let body = strip_position(message).0;
    // The German rendering of the same message (`Unerwartetes Element <span> ...`).
    if let Some(rest) = body.strip_prefix("Unerwartetes Element <") {
        return rest.split('>').next().map(str::to_string);
    }
    quoted_after(body, "unexpected element ").map(|n| local_name(&n))
}

/// `line:column` of a finding location as `(line, column)`.
pub fn parse_location(location: &str) -> Option<(u32, u32)> {
    let (line, col) = location.trim().split_once(':')?;
    Some((line.parse().ok()?, col.parse().ok()?))
}

/// The backtick-quoted name that follows `prefix`.
fn quoted_after(body: &str, prefix: &str) -> Option<String> {
    let rest = body.strip_prefix(prefix)?.trim();
    let inner = rest.strip_prefix('`')?.split('`').next()?;
    (!inner.is_empty()).then(|| inner.to_string())
}

fn unquote(text: &str) -> &str {
    text.trim().trim_matches('`')
}

/// `{http://www.w3.org/1999/xhtml}span` -> `span`.
fn local_name(name: &str) -> String {
    match name.rsplit_once('}') {
        Some((_, local)) => local.to_string(),
        None => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS: &str = "{http://www.w3.org/1999/xhtml}";

    #[test]
    fn canonical_english_is_returned_unchanged() {
        let m = format!("unexpected element `{NS}span` at 190:12271");
        assert_eq!(message_text("html_conform.schema.html5", &m, true), m);
    }

    #[test]
    fn parser_noise_is_not_shown_in_german() {
        let m = format!("unexpected element `{NS}span` at 190:12271");
        let de = message_text("html_conform.schema.html5", &m, false);
        assert_eq!(de, "Unerwartetes Element <span> an dieser Stelle.");
        assert!(!de.contains("xhtml") && !de.contains("12271"));
    }

    #[test]
    fn attribute_messages_are_mapped() {
        let de = message_text(
            "html_conform.schema.html5",
            "invalid value `search` for attribute `type` at 281:508",
            false,
        );
        assert_eq!(de, "Ungültiger Wert „search“ für das Attribut „type“.");
    }

    #[test]
    fn assertions_are_mapped_by_rule_id() {
        let de = message_text(
            "html_conform.assertion.roles.unnecessary-list",
            "The \"list\" role is unnecessary for this element.",
            false,
        );
        assert_eq!(de, "Die Rolle „list“ ist für dieses Element überflüssig.");
        let de = message_text(
            "html_conform.assertion.headings.section-lacks-heading",
            "Section lacks heading. Consider using \"h2\"-\"h6\" elements",
            false,
        );
        assert!(de.starts_with("Abschnitt ohne Überschrift"));
    }

    #[test]
    fn unmapped_messages_fall_back_to_the_family_title_not_english_prose() {
        let de = message_text(
            "html_conform.assertion.elements.img-missing-alt",
            "An img element must have an alt attribute, except under certain conditions.",
            false,
        );
        assert_eq!(de, "Bildern eine Textalternative geben");
        let de = message_text("html_conform.parser.html5", "Unexpected token", false);
        assert!(!de.contains("Unexpected"), "{de}");
    }

    #[test]
    fn element_name_and_position_are_read() {
        let m = format!("unexpected element `{NS}style` at 245:66");
        assert_eq!(element_name(&m).as_deref(), Some("style"));
        assert_eq!(strip_position(&m).1, Some((245, 66)));
        assert_eq!(parse_location("190:12271"), Some((190, 12271)));
        assert_eq!(parse_location("schema.html5"), None);
        assert_eq!(element_name("Section lacks heading."), None);
    }
}
