//! UX page heuristics (#1035)
//!
//! Advisory AXTree checks for common UX anti-patterns, reported under the
//! experimental `ux` indicator (`--experimental ux`):
//! - cookie or consent banner present
//! - dialog not marked modal, several dialogs at once
//! - call-to-action missing, or many competing
//!
//! They are unvalidated heuristics: no score, no verdict, no rule id, not in
//! the PDF. The findings carry a canonical [`UxHeuristicKind`] and the raw
//! value; [`ux_heuristic_text`] is the only source of their wording (#406).
//!
//! A font-diversity check is not part of this module: the accessibility tree
//! holds no font information, so it could only print the same advisory on
//! every page.
//!
//! Detection is language-independent in the sense that every keyword list
//! holds English and German wording: a page language is not known to the
//! tree, and a German page must not be flagged "no call to action" for its
//! wording. A keyword matches at the start of a word, never inside one
//! (`book` is not `Facebook`, `order` is not `border`).

use crate::accessibility::{AXNode, AXTree};
use crate::taxonomy::Severity;
use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};

/// Which heuristic a finding comes from. The stable check id in the JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UxHeuristicKind {
    /// A cookie or consent banner is on the page.
    CookieBanner,
    /// A dialog without `aria-modal`.
    DialogNotModal,
    /// More than one dialog in the tree at once.
    MultipleDialogs,
    /// No button or link with call-to-action wording.
    NoCta,
    /// Many buttons and links with call-to-action wording.
    ManyCtas,
}

/// A single UX heuristic finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxHeuristicFinding {
    /// Which heuristic reported it (the check id)
    pub kind: UxHeuristicKind,
    /// Severity of the finding
    pub severity: Severity,
    /// The counted value (dialogs, call-to-action elements) where the text
    /// names one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Canonical English text, produced by [`ux_heuristic_text`]
    pub message: String,
}

impl UxHeuristicFinding {
    fn new(kind: UxHeuristicKind, severity: Severity, count: Option<u32>) -> Self {
        Self {
            kind,
            severity,
            count,
            message: ux_heuristic_text(kind, count, true),
        }
    }
}

/// Result of all UX heuristic checks for a page
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UxHeuristics {
    /// All findings produced by the checks
    pub findings: Vec<UxHeuristicFinding>,
}

/// The only source of UX heuristic wording (#406): the analysis calls it with
/// `en = true` to bake canonical English into the finding, the terminal table
/// calls it with the run language. The wording is the `ux-heuristic-*`
/// message of `locales/{en,de}/ux-heuristics.ftl`.
pub fn ux_heuristic_text(kind: UxHeuristicKind, count: Option<u32>, en: bool) -> String {
    let key = match kind {
        UxHeuristicKind::CookieBanner => "ux-heuristic-cookie-banner",
        UxHeuristicKind::DialogNotModal => "ux-heuristic-dialog-not-modal",
        UxHeuristicKind::MultipleDialogs => "ux-heuristic-multiple-dialogs",
        UxHeuristicKind::NoCta => "ux-heuristic-no-cta",
        UxHeuristicKind::ManyCtas => "ux-heuristic-many-ctas",
    };
    crate::i18n::text_args(
        key,
        &[(
            "count",
            FluentValue::from(i64::from(count.unwrap_or_default())),
        )],
        en,
    )
}

/// Labels that name a cookie banner container (dialog, region, ...).
const COOKIE_CONTAINER_KEYWORDS: &[&str] = &[
    "cookie",
    "consent",
    "gdpr",
    "dsgvo",
    "einwilligung",
    "privacy settings",
    "privacy preferences",
    "datenschutzeinstellungen",
    "datenschutz-einstellungen",
];

/// Button labels that are the choices of a consent banner. A bare "accept" or
/// "consent" is not enough: it names every terms checkbox.
const COOKIE_BUTTON_PHRASES: &[&str] = &[
    "accept all",
    "accept cookies",
    "allow all",
    "allow cookies",
    "reject all",
    "reject cookies",
    "decline all",
    "decline cookies",
    "necessary only",
    "only necessary",
    "essential only",
    "alle akzeptieren",
    "alle zulassen",
    "alle ablehnen",
    "cookies akzeptieren",
    "cookies zulassen",
    "cookies ablehnen",
    "nur notwendige",
    "nur essenzielle",
    "accept and continue",
    "agree and continue",
    "akzeptieren und weiter",
    "zustimmen und weiter",
];

/// Roles a cookie banner container may have. The page footer
/// (`contentinfo`) and the links of a privacy policy are no banner.
const COOKIE_CONTAINER_ROLES: &[&str] =
    &["dialog", "alertdialog", "region", "complementary", "group"];

/// Wording of a call to action (a phrase starts at a word boundary).
const CTA_KEYWORDS: &[&str] = &[
    "buy",
    "order now",
    "order online",
    "order today",
    "place order",
    "place your order",
    "pre-order",
    "sign up",
    "register",
    "get started",
    "start now",
    "start free",
    "free trial",
    "subscribe",
    "download",
    "contact",
    "book now",
    "book a",
    "book your",
    "schedule",
    "add to cart",
    "checkout",
    "get a quote",
    "request a demo",
    "request a quote",
    "kaufen",
    "bestellen",
    "kontakt",
    "anfrage",
    "anfragen",
    "registrieren",
    "buchen",
    "testen",
    "herunterladen",
    "abonnieren",
    "in den warenkorb",
    "jetzt starten",
    "termin vereinbaren",
    "angebot anfordern",
];

/// The longest label that still counts as a call to action, in words.
const MAX_CTA_WORDS: usize = 6;

/// From this many call-to-action elements on a page they compete for attention.
const MANY_CTAS_FROM: usize = 5;

/// Run all UX heuristic checks against an AXTree and return the results.
pub fn analyze_ux_heuristics(ax_tree: &AXTree) -> UxHeuristics {
    let mut findings = Vec::new();

    check_cookie_banner(ax_tree, &mut findings);
    check_modals(ax_tree, &mut findings);
    check_ctas(ax_tree, &mut findings);

    UxHeuristics { findings }
}

/// Whether `phrase` occurs in `text` (both lower-case) starting at a word
/// boundary: `book` matches "Book now" and "Booking", not "Facebook".
fn contains_phrase(text: &str, phrase: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = text[from..].find(phrase) {
        let start = from + pos;
        let starts_word = text[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        if starts_word {
            return true;
        }
        from = start + phrase.len();
    }
    false
}

fn matches_any(text: &str, phrases: &[&str]) -> bool {
    let lower = text.to_lowercase();
    phrases.iter().any(|phrase| contains_phrase(&lower, phrase))
}

fn role_is(node: &AXNode, roles: &[&str]) -> bool {
    node.role
        .as_deref()
        .is_some_and(|role| roles.contains(&role))
}

// ---------------------------------------------------------------------------
// Cookie-banner detection
// ---------------------------------------------------------------------------

fn is_cookie_banner_node(node: &AXNode) -> bool {
    if node.ignored {
        return false;
    }
    if role_is(node, COOKIE_CONTAINER_ROLES) {
        let labelled = [node.name.as_deref(), node.description.as_deref()]
            .into_iter()
            .flatten()
            .any(|text| matches_any(text, COOKIE_CONTAINER_KEYWORDS));
        if labelled {
            return true;
        }
    }
    role_is(node, &["button"])
        && node
            .name
            .as_deref()
            .is_some_and(|name| matches_any(name, COOKIE_BUTTON_PHRASES))
}

fn check_cookie_banner(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    if ax_tree.iter().any(is_cookie_banner_node) {
        findings.push(UxHeuristicFinding::new(
            UxHeuristicKind::CookieBanner,
            Severity::Medium,
            None,
        ));
    }
}

// ---------------------------------------------------------------------------
// Modal / overlay detection
// ---------------------------------------------------------------------------

fn check_modals(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    let dialog_nodes: Vec<_> = ax_tree
        .iter()
        .filter(|n| role_is(n, &["dialog", "alertdialog"]) && !n.ignored)
        .collect();

    let dialog_count = dialog_nodes.len();

    if dialog_count == 0 {
        return;
    }

    // Check whether any dialog is missing aria-modal=true
    let missing_aria_modal = dialog_nodes
        .iter()
        .any(|n| !n.get_property_bool("modal").unwrap_or(false));

    if missing_aria_modal {
        findings.push(UxHeuristicFinding::new(
            UxHeuristicKind::DialogNotModal,
            Severity::Medium,
            None,
        ));
    }

    if dialog_count > 1 {
        findings.push(UxHeuristicFinding::new(
            UxHeuristicKind::MultipleDialogs,
            Severity::Low,
            Some(dialog_count as u32),
        ));
    }
}

// ---------------------------------------------------------------------------
// CTA detection
// ---------------------------------------------------------------------------

fn is_cta(node: &AXNode) -> bool {
    !node.ignored
        && role_is(node, &["button", "link"])
        && node.name.as_deref().is_some_and(|name| {
            // A teaser headline that happens to hold "register" is no call
            // to action: the label of one is short.
            name.split_whitespace().count() <= MAX_CTA_WORDS && matches_any(name, CTA_KEYWORDS)
        })
}

fn check_ctas(ax_tree: &AXTree, findings: &mut Vec<UxHeuristicFinding>) {
    let cta_count = ax_tree.iter().filter(|n| is_cta(n)).count();

    if cta_count == 0 {
        findings.push(UxHeuristicFinding::new(
            UxHeuristicKind::NoCta,
            Severity::Low,
            None,
        ));
    } else if cta_count >= MANY_CTAS_FROM {
        findings.push(UxHeuristicFinding::new(
            UxHeuristicKind::ManyCtas,
            Severity::Low,
            Some(cta_count as u32),
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn modal(id: &str, name: &str) -> AXNode {
        let mut n = node(id, "dialog", Some(name));
        n.properties = vec![AXProperty {
            name: "modal".to_string(),
            value: AXValue::Bool(true),
        }];
        n
    }

    fn kinds(nodes: Vec<AXNode>) -> Vec<UxHeuristicKind> {
        let mut all = vec![node("0", "WebArea", Some("Page"))];
        all.extend(nodes);
        analyze_ux_heuristics(&AXTree::from_nodes(all))
            .findings
            .into_iter()
            .map(|f| f.kind)
            .collect()
    }

    fn has(nodes: Vec<AXNode>, kind: UxHeuristicKind) -> bool {
        kinds(nodes).contains(&kind)
    }

    // --- cookie banner ---

    #[test]
    fn cookie_banner_found_by_container_label() {
        for (role, name) in [
            ("dialog", "Cookie consent"),
            ("alertdialog", "Wir verwenden Cookies"),
            ("region", "Cookie settings"),
            ("complementary", "Datenschutz-Einstellungen"),
        ] {
            assert!(
                has(
                    vec![node("1", role, Some(name))],
                    UxHeuristicKind::CookieBanner
                ),
                "{role}: {name}"
            );
        }
    }

    #[test]
    fn cookie_banner_found_by_container_description() {
        let mut dialog = node("1", "dialog", Some("Notice"));
        dialog.description = Some("We use cookies to improve the site".to_string());
        assert!(has(vec![dialog], UxHeuristicKind::CookieBanner));
    }

    #[test]
    fn cookie_banner_found_by_consent_choice_button() {
        for name in [
            "Accept all cookies",
            "Reject all",
            "Alle akzeptieren",
            "Nur notwendige Cookies",
            "Akzeptieren und weiter",
        ] {
            assert!(
                has(
                    vec![node("1", "button", Some(name))],
                    UxHeuristicKind::CookieBanner
                ),
                "{name}"
            );
        }
    }

    #[test]
    fn no_cookie_banner_on_look_alike_wording() {
        for (role, name) in [
            ("heading", "Welcome"),
            // the footer link and the terms checkbox of an ordinary page
            ("link", "Privacy Policy"),
            ("link", "Cookie settings"),
            ("button", "Cookie-Einstellungen"),
            ("checkbox", "I accept the terms and conditions"),
            ("button", "Accept"),
            ("checkbox", "I consent to being contacted"),
            ("contentinfo", "Cookie and privacy information"),
            ("heading", "About cookie recipes"),
            ("article", "Consent in medical research"),
        ] {
            assert!(
                !has(
                    vec![node("1", role, Some(name))],
                    UxHeuristicKind::CookieBanner
                ),
                "{role}: {name}"
            );
        }
    }

    #[test]
    fn an_ignored_cookie_banner_is_not_reported() {
        let mut banner = node("1", "dialog", Some("Cookie consent"));
        banner.ignored = true;
        assert!(!has(vec![banner], UxHeuristicKind::CookieBanner));
    }

    // --- modals ---

    #[test]
    fn dialog_without_aria_modal_is_flagged() {
        assert!(has(
            vec![node("1", "dialog", Some("Settings"))],
            UxHeuristicKind::DialogNotModal
        ));
    }

    #[test]
    fn dialog_with_aria_modal_is_not_flagged() {
        let found = kinds(vec![modal("1", "Settings")]);
        assert!(!found.contains(&UxHeuristicKind::DialogNotModal));
        assert!(!found.contains(&UxHeuristicKind::MultipleDialogs));
    }

    #[test]
    fn a_page_without_dialog_has_no_dialog_finding() {
        let found = kinds(vec![node("1", "heading", Some("Welcome"))]);
        assert!(!found.contains(&UxHeuristicKind::DialogNotModal));
        assert!(!found.contains(&UxHeuristicKind::MultipleDialogs));
    }

    #[test]
    fn an_ignored_dialog_is_not_counted() {
        let mut hidden = node("2", "dialog", Some("Hidden"));
        hidden.ignored = true;
        let found = kinds(vec![modal("1", "Shown"), hidden]);
        assert!(!found.contains(&UxHeuristicKind::DialogNotModal));
        assert!(!found.contains(&UxHeuristicKind::MultipleDialogs));
    }

    #[test]
    fn several_dialogs_are_flagged_with_their_count() {
        let tree = AXTree::from_nodes(vec![
            node("0", "WebArea", Some("Page")),
            modal("1", "A"),
            modal("2", "B"),
            modal("3", "C"),
        ]);
        let result = analyze_ux_heuristics(&tree);
        let finding = result
            .findings
            .iter()
            .find(|f| f.kind == UxHeuristicKind::MultipleDialogs)
            .expect("three dialogs are flagged");
        assert_eq!(finding.count, Some(3));
        assert_eq!(
            finding.message,
            "3 dialogs are in the page at the same time. Check that they do not cover each other."
        );
    }

    // --- CTAs ---

    #[test]
    fn no_cta_on_a_page_without_call_to_action_wording() {
        assert!(has(
            vec![
                node("1", "heading", Some("Welcome")),
                node("2", "link", Some("Home")),
                node("3", "link", Some("Startseite")),
            ],
            UxHeuristicKind::NoCta
        ));
    }

    #[test]
    fn a_call_to_action_is_found_in_either_language() {
        for name in [
            "Get Started",
            "Sign up for free",
            "Download the PDF",
            "Add to cart",
            "Book a table",
            "Order now",
            "Jetzt starten",
            "Kontakt aufnehmen",
            "In den Warenkorb",
            "Termin vereinbaren",
        ] {
            let found = kinds(vec![node("1", "button", Some(name))]);
            assert!(!found.contains(&UxHeuristicKind::NoCta), "{name}");
            assert!(!found.contains(&UxHeuristicKind::ManyCtas), "{name}");
        }
    }

    #[test]
    fn call_to_action_wording_inside_another_word_does_not_count() {
        for name in [
            "Facebook",
            "Python Books",
            "Out of order",
            "Open orders",
            "Hidden from tab order",
            "Border radius",
            "Restart the tour",
            "Reorder level",
            "Verkaufen Sie nicht",
            "Startseite",
        ] {
            assert!(
                has(vec![node("1", "link", Some(name))], UxHeuristicKind::NoCta),
                "{name}"
            );
        }
    }

    #[test]
    fn a_long_teaser_headline_is_no_call_to_action() {
        assert!(has(
            vec![node(
                "1",
                "link",
                Some("Article The Register Products Mozilla throws Thunderbolt at enterprise AI providers"),
            )],
            UxHeuristicKind::NoCta
        ));
        // six words still count
        assert!(!has(
            vec![node(
                "1",
                "link",
                Some("Download Wikipedia for Android or iOS")
            )],
            UxHeuristicKind::NoCta
        ));
    }

    #[test]
    fn only_buttons_and_links_count_as_call_to_action() {
        assert!(has(
            vec![
                node("1", "heading", Some("Buy now")),
                node("2", "StaticText", Some("Download")),
            ],
            UxHeuristicKind::NoCta
        ));
    }

    #[test]
    fn many_ctas_are_flagged_from_five() {
        let names = [
            ("button", "Buy Now"),
            ("link", "Sign Up"),
            ("button", "Download"),
            ("link", "Subscribe"),
            ("button", "Register"),
        ];
        let nodes: Vec<AXNode> = names
            .iter()
            .enumerate()
            .map(|(i, (role, name))| node(&format!("{}", i + 1), role, Some(name)))
            .collect();
        let tree_with = |n: usize| {
            let mut all = vec![node("0", "WebArea", Some("Page"))];
            all.extend(nodes.iter().take(n).cloned());
            analyze_ux_heuristics(&AXTree::from_nodes(all))
        };
        assert!(!tree_with(4)
            .findings
            .iter()
            .any(|f| f.kind == UxHeuristicKind::ManyCtas));
        let five = tree_with(5);
        let finding = five
            .findings
            .iter()
            .find(|f| f.kind == UxHeuristicKind::ManyCtas)
            .expect("five are flagged");
        assert_eq!(finding.count, Some(5));
    }

    #[test]
    fn an_ignored_call_to_action_is_not_counted() {
        let mut hidden = node("1", "button", Some("Buy now"));
        hidden.ignored = true;
        assert!(has(vec![hidden], UxHeuristicKind::NoCta));
    }

    // --- shape and text ---

    #[test]
    fn a_page_without_any_finding_has_an_empty_result() {
        let found = kinds(vec![
            node("1", "heading", Some("Welcome")),
            node("2", "button", Some("Get Started")),
        ]);
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn no_font_diversity_finding_comes_from_the_tree() {
        let json = serde_json::to_string(&analyze_ux_heuristics(&AXTree::from_nodes(vec![node(
            "0",
            "WebArea",
            Some("Page"),
        )])))
        .unwrap();
        assert!(!json.contains("font"), "{json}");
    }

    #[test]
    fn the_json_carries_the_kind_and_the_english_text() {
        let result = analyze_ux_heuristics(&AXTree::from_nodes(vec![
            node("0", "WebArea", Some("Page")),
            node("1", "dialog", Some("Cookie consent")),
        ]));
        let json = serde_json::to_value(&result).unwrap();
        let findings = json["findings"].as_array().unwrap();
        let cookie = findings
            .iter()
            .find(|f| f["kind"] == "cookie_banner")
            .expect("cookie banner");
        assert_eq!(cookie["severity"], "medium");
        assert!(cookie.get("count").is_none());
        assert_eq!(
            cookie["message"],
            ux_heuristic_text(UxHeuristicKind::CookieBanner, None, true)
        );
    }

    const KINDS: [UxHeuristicKind; 5] = [
        UxHeuristicKind::CookieBanner,
        UxHeuristicKind::DialogNotModal,
        UxHeuristicKind::MultipleDialogs,
        UxHeuristicKind::NoCta,
        UxHeuristicKind::ManyCtas,
    ];

    /// The English text holds no German umlauts or sharp s, the German text
    /// differs from it, and every message exists in both languages.
    #[test]
    fn texts_exist_in_both_languages_and_english_has_no_german() {
        for kind in KINDS {
            let en = ux_heuristic_text(kind, Some(7), true);
            let de = ux_heuristic_text(kind, Some(7), false);
            assert!(
                !en.starts_with("ux-heuristic"),
                "missing EN message: {kind:?}"
            );
            assert!(
                !de.starts_with("ux-heuristic"),
                "missing DE message: {kind:?}"
            );
            assert_ne!(en, de, "{kind:?}");
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN text of {kind:?} holds German characters: {en}"
            );
        }
    }

    /// `locales/{en,de}/ux-heuristics.ftl` define the same messages, one per
    /// kind plus the two terminal lines, and the English file holds no German.
    #[test]
    fn the_message_files_agree() {
        const EN: &str = include_str!("../../locales/en/ux-heuristics.ftl");
        const DE: &str = include_str!("../../locales/de/ux-heuristics.ftl");
        let ids = |ftl: &str| -> Vec<String> {
            ftl.lines()
                .filter(|line| !line.is_empty() && !line.starts_with([' ', '#']))
                .filter_map(|line| line.split_once(" =").map(|(id, _)| id.to_string()))
                .collect()
        };
        assert_eq!(ids(EN), ids(DE));
        assert_eq!(ids(EN).len(), KINDS.len() + 2);
        assert!(!EN.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
    }

    #[test]
    fn the_count_is_part_of_the_text_where_one_is_named() {
        let many = ux_heuristic_text(UxHeuristicKind::ManyCtas, Some(8), true);
        assert!(many.contains('8'), "{many}");
        let many_de = ux_heuristic_text(UxHeuristicKind::ManyCtas, Some(8), false);
        assert!(many_de.contains('8'), "{many_de}");
    }
}
