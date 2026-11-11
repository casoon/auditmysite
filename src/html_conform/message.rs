//! Run-language text of `html-conform` findings (#902, #406).
//!
//! The stored `message` stays the checker's canonical English (the JSON). A
//! German report shows [`message_text`] instead. Schema findings carry their
//! structured arguments ([`SchemaDetail`], local element and attribute names
//! without the XHTML namespace), from which the text is written. Findings
//! without them (other rule families, or a report read back from JSON) are
//! mapped by rule id and message shape; every other one falls back to the
//! localized title of the rule's family (`recommendation.rs`), so no English
//! checker prose reaches a German report.
//!
//! The wording of the shared checker itself belongs to barrierlab
//! (`html-conform`); this module only maps, it never rewrites its output.

use crate::html_conform::recommendation;

/// What a schema finding reports, with its raw arguments: the canonical kind
/// behind the checker's English message (`html_conform::FindingDetail`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaDetail {
    /// An element is not allowed at this position.
    UnexpectedElement { element: String },
    /// Text is not allowed inside `parent`.
    UnexpectedText { parent: String },
    /// `attribute` is not allowed on `element`.
    UnexpectedAttribute { attribute: String, element: String },
    /// `attribute` on `element` has a value the schema rejects.
    InvalidAttributeValue {
        attribute: String,
        value: String,
        element: String,
    },
    /// `element` ended while required content was still missing.
    MissingContent { element: String },
}

impl SchemaDetail {
    /// The checker's detail, `None` for a kind this version does not know
    /// (`FindingDetail` is non-exhaustive); such a finding uses the fallback.
    pub fn from_checker(detail: &html_conform::FindingDetail) -> Option<Self> {
        use html_conform::FindingDetail as D;
        Some(match detail {
            D::UnexpectedElement { element } => Self::UnexpectedElement {
                element: element.clone(),
            },
            D::UnexpectedText { parent } => Self::UnexpectedText {
                parent: parent.clone(),
            },
            D::UnexpectedAttribute { attribute, element } => Self::UnexpectedAttribute {
                attribute: attribute.clone(),
                element: element.clone(),
            },
            D::InvalidAttributeValue {
                attribute,
                value,
                element,
            } => Self::InvalidAttributeValue {
                attribute: attribute.clone(),
                value: value.clone(),
                element: element.clone(),
            },
            D::MissingContent { element } => Self::MissingContent {
                element: element.clone(),
            },
            _ => return None,
        })
    }

    /// The German text. The checker leaves the element empty when the error
    /// has no enclosing element; the sentence then omits it.
    fn german(&self) -> String {
        let on = |element: &str| {
            if element.is_empty() {
                String::new()
            } else {
                format!(" am Element <{element}>")
            }
        };
        match self {
            Self::UnexpectedElement { element } => {
                format!("Unerwartetes Element <{element}> an dieser Stelle.")
            }
            Self::UnexpectedText { parent } if parent.is_empty() => {
                "Text ist an dieser Stelle nicht erlaubt.".to_string()
            }
            Self::UnexpectedText { parent } => {
                format!("Text ist an dieser Stelle in <{parent}> nicht erlaubt.")
            }
            Self::UnexpectedAttribute { attribute, element } => {
                format!("Unerwartetes Attribut „{attribute}“{}.", on(element))
            }
            Self::InvalidAttributeValue {
                attribute,
                value,
                element,
            } => format!(
                "Ungültiger Wert „{value}“ für das Attribut „{attribute}“{}.",
                on(element)
            ),
            Self::MissingContent { element } if element.is_empty() => {
                "Erforderlicher Inhalt fehlt.".to_string()
            }
            Self::MissingContent { element } => {
                format!("Dem Element <{element}> fehlt erforderlicher Inhalt.")
            }
        }
    }
}

/// The message of a finding in the requested language. `en = true` returns the
/// stored canonical English unchanged; otherwise `detail`, when present, is the
/// text source, and the message is parsed only for findings without one.
pub(crate) fn message_text(
    rule_id: &str,
    message: &str,
    detail: Option<&SchemaDetail>,
    en: bool,
) -> String {
    if en {
        return message.to_string();
    }
    if let Some(detail) = detail {
        return detail.german();
    }
    let bare = recommendation::bare_id(rule_id);
    let body = strip_position(message).0;

    if bare == "schema.html5" {
        if body == "unexpected text" {
            return "Text ist an dieser Stelle nicht erlaubt.".to_string();
        }
        if body == "missing required content" {
            return "Erforderlicher Inhalt fehlt.".to_string();
        }
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
pub(crate) fn strip_position(message: &str) -> (&str, Option<(u32, u32)>) {
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
#[cfg(any(feature = "pdf", test))]
pub(crate) fn element_name(message: &str) -> Option<String> {
    let body = strip_position(message).0;
    // The German rendering of the same message (`Unerwartetes Element <span> ...`).
    if let Some(rest) = body.strip_prefix("Unerwartetes Element <") {
        return rest.split('>').next().map(str::to_string);
    }
    quoted_after(body, "unexpected element ").map(|n| local_name(&n))
}

/// `line:column` of a finding location as `(line, column)`.
#[cfg(any(feature = "pdf", test))]
pub(crate) fn parse_location(location: &str) -> Option<(u32, u32)> {
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
        assert_eq!(message_text("html_conform.schema.html5", &m, None, true), m);
    }

    #[test]
    fn parser_noise_is_not_shown_in_german() {
        let m = format!("unexpected element `{NS}span` at 190:12271");
        let de = message_text("html_conform.schema.html5", &m, None, false);
        assert_eq!(de, "Unerwartetes Element <span> an dieser Stelle.");
        assert!(!de.contains("xhtml") && !de.contains("12271"));
    }

    #[test]
    fn attribute_messages_are_mapped() {
        let de = message_text(
            "html_conform.schema.html5",
            "invalid value `search` for attribute `type` at 281:508",
            None,
            false,
        );
        assert_eq!(de, "Ungültiger Wert „search“ für das Attribut „type“.");
    }

    #[test]
    fn assertions_are_mapped_by_rule_id() {
        let de = message_text(
            "html_conform.assertion.roles.unnecessary-list",
            "The \"list\" role is unnecessary for this element.",
            None,
            false,
        );
        assert_eq!(de, "Die Rolle „list“ ist für dieses Element überflüssig.");
        let de = message_text(
            "html_conform.assertion.headings.section-lacks-heading",
            "Section lacks heading. Consider using \"h2\"-\"h6\" elements",
            None,
            false,
        );
        assert!(de.starts_with("Abschnitt ohne Überschrift"));
    }

    #[test]
    fn unmapped_messages_fall_back_to_the_family_title_not_english_prose() {
        let de = message_text(
            "html_conform.assertion.elements.img-missing-alt",
            "An img element must have an alt attribute, except under certain conditions.",
            None,
            false,
        );
        assert_eq!(de, "Bildern eine Textalternative geben");
        let de = message_text("html_conform.parser.html5", "Unexpected token", None, false);
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

    /// Every kind with the English message the checker writes for it.
    fn schema_cases() -> Vec<(SchemaDetail, String)> {
        vec![
            (
                SchemaDetail::UnexpectedElement {
                    element: "span".into(),
                },
                format!("unexpected element `{NS}span` at 1:85"),
            ),
            (
                SchemaDetail::UnexpectedText {
                    parent: "ul".into(),
                },
                "unexpected text at 1:67".into(),
            ),
            (
                SchemaDetail::UnexpectedAttribute {
                    attribute: "bogus".into(),
                    element: "div".into(),
                },
                "unexpected attribute `bogus` at 1:165".into(),
            ),
            (
                SchemaDetail::InvalidAttributeValue {
                    attribute: "width".into(),
                    value: "100%".into(),
                    element: "img".into(),
                },
                "invalid value `100%` for attribute `width` at 1:104".into(),
            ),
            (
                SchemaDetail::MissingContent {
                    element: "dl".into(),
                },
                "missing required content at 1:187".into(),
            ),
        ]
    }

    #[test]
    fn german_text_is_written_from_the_detail() {
        let expected = [
            "Unerwartetes Element <span> an dieser Stelle.",
            "Text ist an dieser Stelle in <ul> nicht erlaubt.",
            "Unerwartetes Attribut „bogus“ am Element <div>.",
            "Ungültiger Wert „100%“ für das Attribut „width“ am Element <img>.",
            "Dem Element <dl> fehlt erforderlicher Inhalt.",
        ];
        for ((detail, message), expected) in schema_cases().iter().zip(expected) {
            let de = message_text("html_conform.schema.html5", message, Some(detail), false);
            assert_eq!(de, expected);
        }
    }

    #[test]
    fn english_text_stays_the_checkers_message_for_every_kind() {
        for (detail, message) in schema_cases() {
            let en = message_text("html_conform.schema.html5", &message, Some(&detail), true);
            assert_eq!(en, message);
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß„“".contains(c)),
                "German leaked into EN: {en}"
            );
        }
    }

    #[test]
    fn empty_element_names_are_left_out_of_the_german_text() {
        let cases = [
            (
                SchemaDetail::UnexpectedText {
                    parent: String::new(),
                },
                "Text ist an dieser Stelle nicht erlaubt.",
            ),
            (
                SchemaDetail::MissingContent {
                    element: String::new(),
                },
                "Erforderlicher Inhalt fehlt.",
            ),
            (
                SchemaDetail::UnexpectedAttribute {
                    attribute: "x".into(),
                    element: String::new(),
                },
                "Unerwartetes Attribut „x“.",
            ),
        ];
        for (detail, expected) in cases {
            assert_eq!(detail.german(), expected);
        }
    }

    #[test]
    fn findings_without_detail_fall_back_to_the_message() {
        // A report read back from JSON has no detail.
        for (message, expected) in [
            (
                "unexpected text at 1:67",
                "Text ist an dieser Stelle nicht erlaubt.",
            ),
            (
                "missing required content at 1:187",
                "Erforderlicher Inhalt fehlt.",
            ),
        ] {
            assert_eq!(
                message_text("html_conform.schema.html5", message, None, false),
                expected
            );
        }
    }
}
