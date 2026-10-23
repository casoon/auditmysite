//! Easy-language ("Leichte Sprache" / plain-language) availability detection.
//!
//! Source: Bundesfachstelle für Barrierefreiheit, "Barrierefreie Redaktion"
//! guide (reviewed 2026-09-03). Detects whether the audited page offers
//! itself in, or links to, an easy/plain-language version, via three
//! deterministic markers:
//!
//! 1. A `lang`/`hreflang` value using the `-x-simplified` BCP-47 private-use
//!    subtag (e.g. `de-x-simplified`) — the established marker for a
//!    simplified-language variant, generic across primary languages.
//! 2. Link text matching a localized "Leichte Sprache" / "Easy language" /
//!    "Easy-to-read" / "Plain language" phrase list (same locale-merge
//!    approach as the `linktext-generic-stopwords` mechanism, #299).
//! 3. A CSS class name containing a recognized naming convention
//!    (`leichte-sprache`, `einfache-sprache`, `easy-language`,
//!    `plain-language` — deterministic substring match).
//!
//! Presence/linkage only — this makes no attempt to assess the quality of
//! the easy-language content itself (would require semantic evaluation of
//! the target page, out of scope).
//!
//! Advisory-only and score-neutral by construction: this only ever calls
//! `PatternAnalysis::add_recognized` (a positive signal), never emits a
//! `Violation` and is not part of the WCAG rule engine — the same guarantee
//! every other detector in this module already has (see `mod.rs` doc
//! comment: `recognized` never feeds `AccessibilityScorer`/the overall
//! score, only `violations` does).
//!
//! Unlike its sibling detectors, this one needs raw DOM access (CSS class
//! names and `<link hreflang>` in `<head>` are not represented in the
//! AXTree at all), so `detect` takes the live `Page` in addition to the
//! `AXTree` and is called separately (and asynchronously) from
//! `audit::pipeline`, not from `patterns::analyze`.

use chromiumoxide::Page;
use serde::Deserialize;
use tracing::warn;

use crate::accessibility::AXTree;
use crate::i18n::I18n;

use super::{PatternAnalysis, PatternConfidence, PatternMessage};

/// CSS class name substrings recognized as an easy-language marker.
const CSS_CLASS_MARKERS: &[&str] = &[
    "leichte-sprache",
    "einfache-sprache",
    "easy-language",
    "plain-language",
];

/// Locales whose link-text marker list is always checked, regardless of the
/// report's output language — sites frequently mix languages (same rationale
/// as `a11y_journey::link_inventory::SUPPORTED_LOCALES`).
const SUPPORTED_LOCALES: &[&str] = &["de", "en"];

const EASY_LANGUAGE_JS: &str = r#"
(() => {
  const langTags = new Set();
  const docLang = document.documentElement.getAttribute('lang');
  if (docLang) langTags.add(docLang);
  for (const el of document.querySelectorAll('[lang], [hreflang]')) {
    const lang = el.getAttribute('lang');
    const hreflang = el.getAttribute('hreflang');
    if (lang) langTags.add(lang);
    if (hreflang) langTags.add(hreflang);
  }
  const classNames = [];
  for (const el of document.querySelectorAll('[class]')) {
    const raw = typeof el.className === 'string'
      ? el.className
      : (el.className && el.className.baseVal) || '';
    if (raw) classNames.push(raw);
  }
  return { lang_tags: Array.from(langTags), class_names: classNames };
})()
"#;

#[derive(Debug, Default, Deserialize)]
struct RawSignals {
    #[serde(default)]
    lang_tags: Vec<String>,
    #[serde(default)]
    class_names: Vec<String>,
}

/// Whether a BCP-47 language tag uses the `-x-simplified` private-use
/// subtag (e.g. `de-x-simplified`, `en-x-simplified`) — checked generically
/// against the primary subtag rather than hardcoding `de`.
fn is_simplified_lang_tag(value: &str) -> bool {
    let value = value.trim().to_lowercase();
    let mut parts = value.split('-');
    let primary = parts.next().unwrap_or("");
    let is_primary_subtag =
        (2..=3).contains(&primary.len()) && primary.chars().all(|c| c.is_ascii_alphabetic());
    is_primary_subtag
        && matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some("x"), Some("simplified"), None)
        )
}

/// Load easy-language link-text markers from FTL for the given locale.
/// Falls back to an empty list if the key is missing (mirrors
/// `a11y_journey::link_inventory::stopwords_for_locale`).
fn markers_for_locale(locale: &str) -> Vec<String> {
    let Ok(i18n) = I18n::new(locale) else {
        return Vec::new();
    };
    let raw = i18n.t("easy-language-linktext-markers");
    if raw == "easy-language-linktext-markers" {
        return Vec::new(); // key missing — I18n returns the key itself as fallback
    }
    raw.split(',').map(|s| s.trim().to_lowercase()).collect()
}

fn linktext_markers() -> Vec<String> {
    SUPPORTED_LOCALES
        .iter()
        .flat_map(|loc| markers_for_locale(loc))
        .collect()
}

fn has_matching_link_text(tree: &AXTree, markers: &[String]) -> bool {
    if markers.is_empty() {
        return false;
    }
    tree.links().iter().any(|link| {
        link.name.as_deref().is_some_and(|name| {
            let lower = name.to_lowercase();
            markers.iter().any(|m| lower.contains(m.as_str()))
        })
    })
}

/// Canonical text for the EasyLanguage positive signal (#406 message-baked
/// pattern): `detect` bakes this with `en=true` into
/// `RecognizedPattern.message` (JSON stays canonical English); the PDF
/// builder (`output::builder::single::executive::build_positive_signals`)
/// re-derives it with the run locale.
pub(crate) fn message_text(en: bool) -> &'static str {
    if en {
        "The page offers itself in, or links to, an easy-language (\"Leichte Sprache\") version of its content."
    } else {
        "Die Seite bietet sich selbst in Leichter Sprache an oder verlinkt auf eine entsprechende Version."
    }
}

/// Detect easy-language availability signals and, if any matched, add a
/// single `EasyLanguage` positive signal to `out`. Idempotent — does nothing
/// if an `EasyLanguage` signal is already recognized.
pub(crate) async fn detect(page: &Page, tree: &AXTree, out: &mut PatternAnalysis) {
    if out.has_recognized("EasyLanguage") {
        return;
    }

    let mut lang_marker = false;
    let mut css_class_marker = false;

    match page.evaluate(EASY_LANGUAGE_JS).await {
        Ok(eval) => {
            if let Some(value) = eval.value() {
                match serde_json::from_value::<RawSignals>(value.clone()) {
                    Ok(raw) => {
                        lang_marker = raw.lang_tags.iter().any(|t| is_simplified_lang_tag(t));
                        css_class_marker = raw.class_names.iter().any(|classes| {
                            let lower = classes.to_lowercase();
                            CSS_CLASS_MARKERS.iter().any(|m| lower.contains(m))
                        });
                    }
                    Err(e) => warn!("Failed to parse easy-language signals JSON: {}", e),
                }
            }
        }
        Err(e) => warn!("Easy-language detection JS failed: {}", e),
    }

    let markers = linktext_markers();
    let link_text_marker = has_matching_link_text(tree, &markers);

    if !(lang_marker || css_class_marker || link_text_marker) {
        return;
    }

    let confidence = if lang_marker || link_text_marker {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };

    out.add_recognized("EasyLanguage", PatternMessage::EasyLanguage, confidence);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn link(id: &str, name: &str) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".into()),
            name: Some(name.into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("root".into()),
            backend_dom_node_id: None,
        }
    }

    fn root_with_children(child_ids: Vec<&str>) -> AXNode {
        AXNode {
            node_id: "root".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("WebArea".into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: child_ids.into_iter().map(String::from).collect(),
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_is_simplified_lang_tag() {
        assert!(is_simplified_lang_tag("de-x-simplified"));
        assert!(is_simplified_lang_tag("DE-X-SIMPLIFIED"));
        assert!(is_simplified_lang_tag("en-x-simplified"));
        assert!(!is_simplified_lang_tag("de"));
        assert!(!is_simplified_lang_tag("de-DE"));
        assert!(!is_simplified_lang_tag("de-x-simplified-extra"));
        assert!(!is_simplified_lang_tag(""));
    }

    #[test]
    fn test_has_matching_link_text_de() {
        let tree = AXTree::from_nodes(vec![
            root_with_children(vec!["1", "2"]),
            link("1", "Home"),
            link("2", "Leichte Sprache"),
        ]);
        let markers = linktext_markers();
        assert!(has_matching_link_text(&tree, &markers));
    }

    #[test]
    fn test_has_matching_link_text_en() {
        let tree = AXTree::from_nodes(vec![
            root_with_children(vec!["1", "2"]),
            link("1", "Home"),
            link("2", "Easy language"),
        ]);
        let markers = linktext_markers();
        assert!(has_matching_link_text(&tree, &markers));
    }

    #[test]
    fn test_no_matching_link_text() {
        let tree = AXTree::from_nodes(vec![root_with_children(vec!["1"]), link("1", "Home")]);
        let markers = linktext_markers();
        assert!(!has_matching_link_text(&tree, &markers));
    }

    #[test]
    fn test_message_text_localized() {
        assert!(message_text(true).contains("easy-language"));
        assert!(message_text(false).contains("Leichter Sprache"));
    }
}
