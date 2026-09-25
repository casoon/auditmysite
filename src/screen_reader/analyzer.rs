use std::collections::HashSet;

use crate::i18n::I18n;

use super::navigator::{
    FormControlQuality, HeadingQuality, LandmarkQuality, LinkQuality, NavigationViews,
};
use super::types::{ReadingItem, SrAuditIssue};

/// How many consecutive announced items (see [`carries_announced_content`]) may
/// pass without a landmark, heading or focus target before the stretch is worth
/// reporting. Counted in real announcements, not in AX nodes.
const ANNOUNCEMENT_DESERT_THRESHOLD: usize = 15;
const TAB_STOP_WARNING_THRESHOLD: usize = 50;

/// Analyzes the reading sequence for screen-reader issues.
///
/// Two locales are threaded independently (#406):
/// - `detect_locale` drives *which* issues are produced — it loads the
///   page-language stopword list used to spot generic link/button names.
///   Using the run/output language here would silently stop detecting
///   generic names like "Hier" on German pages.
/// - `message_en` only controls the *language* of the produced messages.
pub fn analyze_reading_sequence(
    items: &[ReadingItem],
    views: &NavigationViews,
    detect_locale: &str,
    message_en: bool,
    has_disclosure_menu_pattern: bool,
) -> Vec<SrAuditIssue> {
    let stopwords = localized_stopwords(detect_locale);
    let en = message_en;
    let mut issues = Vec::new();

    detect_non_descriptive_interactive_names(items, &stopwords, en, &mut issues);
    detect_duplicated_accessible_name(items, &stopwords, en, &mut issues);
    detect_icon_font_contamination(items, en, &mut issues);
    detect_duplicate_link_texts(views, en, &mut issues);
    detect_announcement_deserts(items, en, &mut issues);
    detect_skipped_heading_levels(views, en, &mut issues);
    detect_heading_order_issues(views, en, &mut issues);
    detect_missing_required_landmarks(views, has_disclosure_menu_pattern, en, &mut issues);
    detect_unlabeled_duplicate_landmarks(views, en, &mut issues);
    detect_tab_stop_count(items, en, &mut issues);
    detect_empty_interactive_elements(items, en, &mut issues);
    detect_empty_form_labels(views, en, &mut issues);

    // Sanitize node references: drop empty strings (failed lookups, #480) and
    // synthetic negative AX node IDs (#481), which cannot be cross-referenced to
    // a real DOM node. Leaves a valid empty array when nothing real remains.
    for issue in &mut issues {
        issue.affected_node_ids.retain(|id| is_real_node_id(id));
    }

    issues
}

/// A node ID is reportable only when it can be resolved to a real DOM/AX node.
/// Empty strings come from failed lookups; Chrome emits negative AX node IDs for
/// synthetic nodes that have no backing DOM element.
fn is_real_node_id(id: &str) -> bool {
    let id = id.trim();
    !id.is_empty() && !id.starts_with('-')
}

fn localized_stopwords(locale: &str) -> HashSet<String> {
    I18n::new(locale)
        .or_else(|_| I18n::new("de"))
        .map(|i18n| {
            i18n.t("linktext-generic-stopwords")
                .split(',')
                .map(normalize_text)
                .filter(|word| !word.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Ambiguous "continuation" words — a wizard's "Weiter" (Next), an article
/// teaser's "Weiterlesen"/"Read more", a row's "Details" expander — are
/// often perfectly clear in the surrounding context they normally appear in,
/// but that context isn't visible to this flat, already-linearized item
/// list (plan/33-screen-reader-thresholds-unvalidated.md). Downgraded to
/// "low" rather than suppressed, since the checker still can't verify
/// context either way. Words that only describe the click mechanism itself
/// ("hier klicken"/"click here", "hier"/"here", "klicken"/"click", "link")
/// carry no meaning in any context and stay at the higher severity.
const AMBIGUOUS_CONTINUATION_WORDS: &[&str] = &[
    "weiter",
    "mehr",
    "more",
    "details",
    "mehr erfahren",
    "learn more",
    "weiterlesen",
    "read more",
    "view",
    "see more",
    "find out more",
    "discover",
];

fn detect_non_descriptive_interactive_names(
    items: &[ReadingItem],
    stopwords: &HashSet<String>,
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    for item in items.iter().filter(|item| {
        matches!(item.role.as_deref(), Some("button" | "link")) && !is_empty_name(&item.name)
    }) {
        let name = item.name.as_deref().unwrap_or_default();
        let normalized = normalize_text(name);
        if stopwords.contains(&normalized)
            || matches!(normalized.as_str(), "x" | "icon" | "bild" | "image")
        {
            let severity = if AMBIGUOUS_CONTINUATION_WORDS.contains(&normalized.as_str()) {
                "low"
            } else {
                "medium"
            };
            issues.push(SrAuditIssue {
                wcag_criterion: Some("2.4.4".into()),
                severity: severity.into(),
                affected_node_ids: vec![item.node_id.clone()],
                message: if en {
                    format!("Interactive name \"{name}\" is not meaningful without context.")
                } else {
                    format!("Interaktiver Name \"{name}\" ist ohne Kontext nicht aussagekräftig.")
                },
            });
        }
    }
}

/// Detects an accessible name whose words split exactly into two identical
/// halves back-to-back, e.g. "Kontakt Kontakt" or "AGB AGB" (confirmed live
/// on shop.satower-mosterei.de, 2026-09-04: a screen reader announces such
/// names twice in a row, which reads as a glitch rather than a real repeated
/// word). This operates on the already-resolved AXTree `name` -- not on the
/// DOM's `aria-label`/visible-text pair -- so it only fires when the *final*
/// accessible name actually contains the duplication (e.g. produced by
/// name-from-content concatenating a labelled icon's name with adjacent
/// visible text). An `aria-label` that simply repeats the visible text (e.g.
/// `aria-label="Kontakt"` on `<a>Kontakt</a>`) never reaches this check: per
/// the accessible-name computation, `aria-label` replaces content text
/// rather than concatenating with it, so the resolved name is "Kontakt"
/// once, not twice.
///
/// Two guardrails avoid flagging harmless repetition: the repeated phrase
/// must be at least 2 characters (excludes single icon glyphs, already
/// covered by `detect_icon_font_contamination`), and it must not be a
/// generic stopword (reuses the same localized `linktext-generic-stopwords`
/// list as `detect_non_descriptive_interactive_names` -- a doubled generic
/// word like "Mehr Mehr" is already covered by that check and is a weaker,
/// less actionable signal than a doubled real word).
fn detect_duplicated_accessible_name(
    items: &[ReadingItem],
    stopwords: &HashSet<String>,
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    for item in items.iter().filter(|item| {
        matches!(item.role.as_deref(), Some("button" | "link")) && !is_empty_name(&item.name)
    }) {
        let name = item.name.as_deref().unwrap_or_default();
        let Some(half) = duplicated_half(name) else {
            continue;
        };
        let normalized_half = normalize_text(&half);
        if normalized_half.chars().count() < 2 || stopwords.contains(&normalized_half) {
            continue;
        }
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.4".into()),
            severity: "medium".into(),
            affected_node_ids: vec![item.node_id.clone()],
            message: if en {
                format!(
                    "Accessible name \"{name}\" repeats \"{half}\" twice in a row. \
                     A screen reader announces it as \"{name}\", which sounds like a glitch."
                )
            } else {
                format!(
                    "Zugänglicher Name \"{name}\" wiederholt \"{half}\" zweimal hintereinander. \
                     Ein Screenreader kündigt ihn als \"{name}\" an, was wie ein Fehler wirkt."
                )
            },
        });
    }
}

/// Returns the repeated phrase when `name`'s whitespace-separated words split
/// into two identical (case-insensitive) halves back-to-back, e.g.
/// "Kontakt Kontakt" -> `Some("Kontakt")`. `None` for an odd word count or
/// fewer than 2 words, or when the two halves differ.
fn duplicated_half(name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() < 2 || !words.len().is_multiple_of(2) {
        return None;
    }
    let mid = words.len() / 2;
    let (first, second) = words.split_at(mid);
    let is_duplicate = first
        .iter()
        .zip(second.iter())
        .all(|(a, b)| a.to_lowercase() == b.to_lowercase());
    is_duplicate.then(|| first.join(" "))
}

fn detect_duplicate_link_texts(views: &NavigationViews, en: bool, issues: &mut Vec<SrAuditIssue>) {
    for link in views.links.iter().filter(|link| link.count > 1) {
        let quality = match (link.quality, en) {
            (LinkQuality::Empty, true) => "Empty link text",
            (LinkQuality::Empty, false) => "Leerer Linktext",
            (LinkQuality::NonDescriptive | LinkQuality::ContextDependent, true) => {
                "Context-dependent link text"
            }
            (LinkQuality::NonDescriptive | LinkQuality::ContextDependent, false) => {
                "Kontextabhängiger Linktext"
            }
            (LinkQuality::Good, _) => continue,
        };
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.4".into()),
            severity: "low".into(),
            affected_node_ids: link.node_ids.clone(),
            message: if en {
                format!(
                    "{} occurs {} times and clutters the screen reader's link list.",
                    quality, link.count
                )
            } else {
                format!(
                    "{} kommt {} mal vor und erschwert die Linkliste im Screenreader.",
                    quality, link.count
                )
            },
        });
    }
}

fn detect_announcement_deserts(items: &[ReadingItem], en: bool, issues: &mut Vec<SrAuditIssue>) {
    let mut segment_start = 0usize;
    let mut count = 0usize;
    let mut node_ids = Vec::new();

    for item in items {
        if is_orientation_item(item) {
            if count > ANNOUNCEMENT_DESERT_THRESHOLD {
                push_desert_issue(segment_start, count, &node_ids, en, issues);
            }
            segment_start = item.seq + 1;
            count = 0;
            node_ids.clear();
        } else if carries_announced_content(item) {
            count += 1;
            node_ids.push(item.node_id.clone());
        }
    }

    if count > ANNOUNCEMENT_DESERT_THRESHOLD {
        push_desert_issue(segment_start, count, &node_ids, en, issues);
    }
}

/// Whether a reading item is something a screen reader actually voices, as
/// opposed to a structural wrapper that only exists to hold children.
///
/// The reading order contains both. A `paragraph`, `figure`, `Figcaption`,
/// `list`, `group` or `generic` node carries no text of its own -- its content
/// lives in the `StaticText` children that follow it -- so counting the wrapper
/// *and* its text counts the same announcement two or three times over.
///
/// This matters only for the announcement-desert distance measure, which asks
/// "how much does a user hear before the next orientation point". Counting
/// wrappers made that distance a function of markup nesting rather than of
/// content: on www.sachsen-anhalt.de (2026-09-17) a news teaser card of four
/// announced items -- headline, date, teaser text, image credit -- was reported
/// as a 23-entry desert. Name/value presence is the role-agnostic test for
/// "has something to say", so a wrapper with an author-supplied name (an
/// `aria-label`led `group`, say) still counts.
fn carries_announced_content(item: &ReadingItem) -> bool {
    !is_empty_name(&item.name) || !is_empty_name(&item.value)
}

fn push_desert_issue(
    segment_start: usize,
    count: usize,
    node_ids: &[String],
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    issues.push(SrAuditIssue {
        // Heuristic structural finding (long region without orientation point).
        // Mapped to 1.3.6 (Identify Purpose) for a self-documenting criterion,
        // consistent with the missing-landmark issues. 1.3.6 is intentionally not
        // in the BFSG Level-A/AA list, so it does not create a BFSG violation.
        wcag_criterion: Some("1.3.6".into()),
        severity: "low".into(),
        affected_node_ids: node_ids.to_vec(),
        message: if en {
            format!(
                "Long section from sequence position {} without a landmark, heading or focus target ({} entries).",
                segment_start, count
            )
        } else {
            format!(
                "Langer Abschnitt ab Sequenzposition {} ohne Landmark, Überschrift oder Fokusziel ({} Einträge).",
                segment_start, count
            )
        },
    });
}

fn detect_skipped_heading_levels(
    views: &NavigationViews,
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    for heading in views
        .headings
        .iter()
        .filter(|heading| heading.quality == HeadingQuality::SkippedLevel)
    {
        // Heading level as "h6" (or "an unknown level" if the AX tree didn't
        // expose one) -- `heading.level` is `Option<u8>`, so this must not be
        // formatted with `{:?}`. That previously leaked "Some(6)" verbatim
        // into the message (confirmed live in the shop.satower-mosterei.de
        // report, 2026-08-31).
        let level = heading.level.map(|l| format!("h{l}")).unwrap_or_else(|| {
            if en {
                "an unknown level".into()
            } else {
                "einer unbekannten Ebene".into()
            }
        });
        issues.push(SrAuditIssue {
            // WCAG 2.4.6 (Headings and Labels) is about whether heading TEXT
            // is descriptive, not heading nesting order -- confirmed against
            // the W3C Understanding doc. The shared rule for the identical
            // check, `headings/skip-level` from `a11y-rules`, is tagged 1.3.1
            // (Info and Relationships) in `SHARED_RULES`, src/wcag/shared.rs;
            // mirrored here for consistency instead of a separate, incorrect
            // 2.4.6 citation.
            wcag_criterion: Some("1.3.1".into()),
            severity: "medium".into(),
            affected_node_ids: vec![heading.node_id.clone()],
            message: if en {
                format!(
                    "Heading level is skipped: {level} at sequence position {}.",
                    heading.seq
                )
            } else {
                format!(
                    "Überschriftenebene wird übersprungen: {level} an Sequenzposition {}.",
                    heading.seq
                )
            },
        });
    }
}

fn detect_unlabeled_duplicate_landmarks(
    views: &NavigationViews,
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    let affected: Vec<String> = views
        .landmarks
        .iter()
        .filter(|landmark| landmark.quality == LandmarkQuality::UnlabeledDuplicate)
        .map(|landmark| landmark.node_id.clone())
        .collect();

    if affected.len() > 1 {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("1.3.1".into()),
            severity: "medium".into(),
            affected_node_ids: affected,
            message: if en {
                "Several landmarks of the same type without a name are indistinguishable in the screen reader's landmark list.".into()
            } else {
                "Mehrere gleichartige Landmarken ohne Namen sind in der Screenreader-Landmarkliste nicht unterscheidbar.".into()
            },
        });
    }

    for landmark in views
        .landmarks
        .iter()
        .filter(|landmark| landmark.quality == LandmarkQuality::MissingMain)
    {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("1.3.1".into()),
            severity: "medium".into(),
            affected_node_ids: vec![landmark.node_id.clone()],
            message: if en {
                "No main landmark detectable in the screen reader's landmark list.".into()
            } else {
                "Kein Main-Landmark in der Screenreader-Landmarkliste erkennbar.".into()
            },
        });
    }
}

fn detect_missing_required_landmarks(
    views: &NavigationViews,
    has_disclosure_menu_pattern: bool,
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    const REQUIRED_EN: &[(&str, &str)] = &[
        (
            "banner",
            "No header area (banner landmark) present. Screen readers cannot fully navigate the page structure.",
        ),
        (
            "navigation",
            "No navigation landmark present. Keyboard users cannot jump directly to the navigation.",
        ),
        (
            "contentinfo",
            "No footer landmark (contentinfo) present. The page structure is incomplete for screen readers.",
        ),
    ];
    const REQUIRED_DE: &[(&str, &str)] = &[
        (
            "banner",
            "Kein Header-Bereich (banner-Landmark) vorhanden. Screen Reader können die Seitenstruktur nicht vollständig navigieren.",
        ),
        (
            "navigation",
            "Keine Navigations-Landmark vorhanden. Tastaturnutzer können nicht direkt zur Navigation springen.",
        ),
        (
            "contentinfo",
            "Keine Fußzeilen-Landmark (contentinfo) vorhanden. Die Seitenstruktur ist für Screen Reader unvollständig.",
        ),
    ];

    let required = if en { REQUIRED_EN } else { REQUIRED_DE };
    for (role, message) in required {
        let present = views
            .landmarks
            .iter()
            .any(|l| l.role == *role && l.quality != LandmarkQuality::MissingMain);
        if !present {
            // A recognized disclosure/hamburger-menu trigger means the nav is
            // most likely just collapsed at capture time, not truly absent —
            // the static AXTree doesn't reflect the post-toggle DOM (#504).
            if *role == "navigation" && has_disclosure_menu_pattern {
                issues.push(SrAuditIssue {
                    wcag_criterion: Some("1.3.6".into()),
                    severity: "low".into(),
                    affected_node_ids: vec![],
                    message: navigation_behind_disclosure_message(en),
                });
                continue;
            }
            issues.push(SrAuditIssue {
                wcag_criterion: Some("1.3.6".into()),
                severity: "medium".into(),
                affected_node_ids: vec![],
                message: message.to_string(),
            });
        }
    }
}

fn navigation_behind_disclosure_message(en: bool) -> String {
    if en {
        "No navigation landmark present in the captured accessibility tree, but a disclosure/menu \
         trigger was recognized — the navigation is likely collapsed rather than missing. Confirm \
         it becomes reachable once expanded."
            .to_string()
    } else {
        "Keine Navigations-Landmark im erfassten Accessibility-Tree vorhanden, aber ein Ausklapp-/\
         Menü-Element wurde erkannt — die Navigation ist vermutlich eingeklappt statt fehlend. \
         Prüfen, ob sie nach dem Ausklappen erreichbar ist."
            .to_string()
    }
}

fn detect_icon_font_contamination(items: &[ReadingItem], en: bool, issues: &mut Vec<SrAuditIssue>) {
    let affected: Vec<String> = items
        .iter()
        .filter(|item| {
            matches!(item.role.as_deref(), Some("button" | "link"))
                && item.name.as_deref().is_some_and(contains_pua)
        })
        .map(|item| item.node_id.clone())
        .collect();

    if !affected.is_empty() {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.4".into()),
            severity: "medium".into(),
            affected_node_ids: affected,
            message: if en {
                "Link or button name contains icon-font characters (Unicode Private Use Area). Screen readers read these out as cryptic character codes.".into()
            } else {
                "Link- oder Button-Name enthält Icon-Font-Zeichen (Unicode Private Use Area). Screen Reader lesen diese als kryptische Zeichencodes vor.".into()
            },
        });
    }
}

fn contains_pua(text: &str) -> bool {
    text.chars().any(|c| ('\u{E000}'..='\u{F8FF}').contains(&c))
}

fn detect_heading_order_issues(views: &NavigationViews, en: bool, issues: &mut Vec<SrAuditIssue>) {
    let headings = &views.headings;

    // First non-empty heading must be H1.
    if let Some(first) = headings.iter().find(|h| h.quality != HeadingQuality::Empty) {
        if first.level != Some(1) {
            issues.push(SrAuditIssue {
                wcag_criterion: Some("1.3.1".into()),
                severity: "medium".into(),
                affected_node_ids: vec![first.node_id.clone()],
                message: if en {
                    format!(
                        "First heading in the document is H{} instead of H1 (sequence position {}).",
                        first.level.unwrap_or(0),
                        first.seq
                    )
                } else {
                    format!(
                        "Erste Überschrift im Dokument ist H{} statt H1 (Sequenzposition {}).",
                        first.level.unwrap_or(0),
                        first.seq
                    )
                },
            });
        }
    }

    // H1 must not appear after H2/H3 in document order.
    let first_sub_seq = headings
        .iter()
        .find(|h| h.level.map(|l| l >= 2).unwrap_or(false))
        .map(|h| h.seq);
    let first_h1 = headings.iter().find(|h| h.level == Some(1));

    if let (Some(sub_seq), Some(h1)) = (first_sub_seq, first_h1) {
        if sub_seq < h1.seq {
            issues.push(SrAuditIssue {
                wcag_criterion: Some("1.3.1".into()),
                severity: "medium".into(),
                affected_node_ids: vec![h1.node_id.clone()],
                message: if en {
                    format!(
                        "H1 first appears at sequence position {} — after an H2/H3 heading.",
                        h1.seq
                    )
                } else {
                    format!(
                        "H1 erscheint erst an Sequenzposition {} — nach einer H2/H3-Überschrift.",
                        h1.seq
                    )
                },
            });
        }
    }
}

fn detect_tab_stop_count(items: &[ReadingItem], en: bool, issues: &mut Vec<SrAuditIssue>) {
    let tab_stop_count = items.iter().filter(|item| item.tab_stop).count();
    // Shared keyword list with `patterns::skip_link` (#513: this check used
    // to keep its own narrower copy that missed common, entirely idiomatic
    // phrasings like "Zum Inhalt springen" — no "skip"/"überspring" substring,
    // but a real, working skip link).
    let has_skip_link = items.iter().any(|item| {
        item.role.as_deref() == Some("link")
            && item
                .name
                .as_deref()
                .is_some_and(crate::patterns::skip_link::is_skip_link_text)
    });

    if tab_stop_count > TAB_STOP_WARNING_THRESHOLD && !has_skip_link {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.1".into()),
            severity: "medium".into(),
            affected_node_ids: items
                .iter()
                .filter(|item| item.tab_stop)
                .map(|item| item.node_id.clone())
                .collect(),
            message: if en {
                format!(
                    "{} tab stops without a detectable skip link hamper keyboard and screen reader navigation.",
                    tab_stop_count
                )
            } else {
                format!(
                    "{} Tab-Stops ohne erkennbaren Skip-Link erschweren Tastatur- und Screenreader-Navigation.",
                    tab_stop_count
                )
            },
        });
    }
}

fn detect_empty_interactive_elements(
    items: &[ReadingItem],
    en: bool,
    issues: &mut Vec<SrAuditIssue>,
) {
    let affected: Vec<String> = items
        .iter()
        .filter(|item| {
            matches!(item.role.as_deref(), Some("button" | "link")) && is_empty_name(&item.name)
        })
        .map(|item| item.node_id.clone())
        .collect();

    if !affected.is_empty() {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("4.1.2".into()),
            severity: "high".into(),
            affected_node_ids: affected,
            message: if en {
                "Interactive elements without an accessible name are not announced intelligibly by a screen reader.".into()
            } else {
                "Interaktive Elemente ohne zugänglichen Namen werden im Screenreader nicht verständlich angekündigt.".into()
            },
        });
    }
}

fn detect_empty_form_labels(views: &NavigationViews, en: bool, issues: &mut Vec<SrAuditIssue>) {
    let affected: Vec<String> = views
        .form_controls
        .iter()
        .filter(|control| control.quality == FormControlQuality::EmptyLabel)
        .map(|control| control.node_id.clone())
        .collect();

    if !affected.is_empty() {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("3.3.2".into()),
            severity: "high".into(),
            affected_node_ids: affected,
            message: if en {
                "Form fields without a label are not intelligible in the screen reader's form list.".into()
            } else {
                "Formularfelder ohne Label sind in der Screenreader-Formularliste nicht verständlich.".into()
            },
        });
    }
}

fn is_orientation_item(item: &ReadingItem) -> bool {
    item.tab_stop
        || matches!(
            item.role.as_deref(),
            Some(
                "heading"
                    | "banner"
                    | "navigation"
                    | "main"
                    | "contentinfo"
                    | "complementary"
                    | "search"
                    | "region"
                    | "form"
            )
        )
}

fn is_empty_name(name: &Option<String>) -> bool {
    name.as_deref().is_none_or(|name| name.trim().is_empty())
}

fn normalize_text(text: &str) -> String {
    text.trim().to_lowercase()
}

pub fn name_quality_score(items: &[ReadingItem], detect_locale: &str) -> u32 {
    let stopwords = localized_stopwords(detect_locale);
    let interactive: Vec<_> = items
        .iter()
        .filter(|item| item.tab_stop || matches!(item.role.as_deref(), Some("button" | "link")))
        .collect();
    if interactive.is_empty() {
        return 100;
    }

    let good = interactive
        .iter()
        .filter(|item| {
            let Some(name) = item
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
            else {
                return false;
            };
            !stopwords.contains(&normalize_text(name))
                && !matches!(
                    normalize_text(name).as_str(),
                    "x" | "icon" | "bild" | "image"
                )
        })
        .count();

    ((good as f64 / interactive.len() as f64) * 100.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::{analyze_reading_sequence, name_quality_score};
    use crate::screen_reader::{navigation_views, ReadingItem};

    fn item(
        seq: usize,
        role: &str,
        name: Option<&str>,
        tab_stop: bool,
        states: Vec<&str>,
    ) -> ReadingItem {
        ReadingItem {
            seq,
            role: Some(role.to_string()),
            name: name.map(String::from),
            description: None,
            value: None,
            states: states.into_iter().map(String::from).collect(),
            tab_stop,
            depth: 0,
            node_id: format!("node-{seq}"),
        }
    }

    #[test]
    fn announcement_desert_ignores_structural_wrappers() {
        // A news teaser card as Chrome exposes it: a handful of announced texts
        // wrapped in figure/paragraph/group/generic containers that carry no
        // text of their own. Counting the wrappers turned four announcements
        // into a 23-entry "desert" on www.sachsen-anhalt.de (2026-09-17).
        let mut items = vec![
            item(0, "heading", Some("News"), false, vec!["level=2"]),
            item(
                1,
                "StaticText",
                Some("Finanzminister begrusst Studierende"),
                false,
                vec![],
            ),
            item(2, "time", None, false, vec![]),
            item(3, "StaticText", Some("14.09.2026"), false, vec![]),
            item(
                4,
                "StaticText",
                Some("In Sachsen-Anhalt haben 100 Menschen"),
                false,
                vec![],
            ),
        ];
        // 20 empty structural wrappers -- far past the threshold if counted.
        for seq in 5..25 {
            let role = ["figure", "generic", "paragraph", "group"][seq % 4];
            items.push(item(seq, role, None, false, vec![]));
        }
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", true, false);

        assert!(
            !issues
                .iter()
                .any(|issue| issue.message.contains("Long section")),
            "empty wrappers must not add up to a desert: {:?}",
            issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn announcement_desert_still_fires_on_real_uninterrupted_content() {
        let mut items = vec![item(0, "heading", Some("Kapitel"), false, vec!["level=1"])];
        for seq in 1..=20 {
            items.push(item(
                seq,
                "StaticText",
                Some("Ein Absatz mit echtem Text"),
                false,
                vec![],
            ));
        }
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", true, false);

        let desert = issues
            .iter()
            .find(|issue| issue.message.contains("Long section"))
            .expect("20 announced items without an orientation point is a desert");
        assert!(
            desert.message.contains("20 entries"),
            "the count must be announcements, not AX nodes: {}",
            desert.message
        );
    }

    #[test]
    fn detects_non_descriptive_and_empty_interactive_names() {
        let items = vec![
            item(0, "link", Some("Hier"), true, vec![]),
            item(1, "button", None, true, vec![]),
            item(2, "button", Some("Menü öffnen"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(issues
            .iter()
            .any(|issue| issue.wcag_criterion.as_deref() == Some("2.4.4")));
        assert!(issues
            .iter()
            .any(|issue| issue.wcag_criterion.as_deref() == Some("4.1.2")));
        assert_eq!(name_quality_score(&items, "de"), 33);
    }

    #[test]
    fn detects_skipped_heading_and_duplicate_landmarks() {
        let items = vec![
            item(0, "navigation", None, false, vec![]),
            item(1, "main", Some("Inhalt"), false, vec![]),
            item(2, "navigation", None, false, vec![]),
            item(3, "heading", Some("Start"), false, vec!["level=1"]),
            item(4, "heading", Some("Deep"), false, vec!["level=3"]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("Landmarken")));
        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("Überschriftenebene")));
    }

    #[test]
    fn skipped_heading_level_is_tagged_1_3_1_not_2_4_6_and_has_no_debug_leak() {
        // Regression (shop.satower-mosterei.de, 2026-08-31): the message
        // formatted `heading.level` (Option<u8>) via `{:?}`, leaking
        // "Some(6)" verbatim, and tagged the issue as WCAG 2.4.6 (which is
        // about descriptive heading/label *text*, not heading nesting order
        // -- verified against the W3C Understanding doc). The shared rule for
        // the identical check, `headings/skip-level`, is tagged 1.3.1 in
        // `SHARED_RULES` (src/wcag/shared.rs); this module must match it.
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "heading", Some("Start"), false, vec!["level=1"]),
            item(2, "heading", Some("Deep"), false, vec!["level=3"]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", true, false);

        let skipped = issues
            .iter()
            .find(|issue| issue.message.contains("skipped"))
            .expect("expected a skipped-heading-level issue");
        assert_eq!(skipped.wcag_criterion.as_deref(), Some("1.3.1"));
        assert!(
            !skipped.message.contains("Some("),
            "message leaks Option debug format: {}",
            skipped.message
        );
        assert!(skipped.message.contains("h3"));
    }

    #[test]
    fn does_not_flag_icon_button_with_accessible_name() {
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
            item(4, "button", Some("Suche öffnen"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(issues.is_empty());
    }

    #[test]
    fn english_locale_messages_carry_no_german_umlauts() {
        // Guard against German leaking into EN reports (#406): build a scenario
        // that triggers many detectors and assert the English messages contain
        // no German umlauts/ß.
        let mut items = vec![
            item(0, "link", Some("Hier"), true, vec![]),
            item(1, "button", None, true, vec![]),
            item(2, "main", Some("Content"), false, vec!["level=3"]),
            item(3, "heading", Some("Deep"), false, vec!["level=3"]),
        ];
        // Many tab stops without a skip link → tab-stop warning.
        for seq in 4..40 {
            items.push(item(
                seq,
                "link",
                Some(&format!("Link {seq}")),
                true,
                vec![],
            ));
        }
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "en", true, false);
        assert!(!issues.is_empty(), "scenario should produce issues");
        for issue in &issues {
            assert!(
                !issue.message.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN message contains German umlaut: {}",
                issue.message
            );
        }
    }

    #[test]
    fn recognizes_idiomatic_german_skip_link_phrasing_not_just_ueberspring() {
        // Regression test for #513: "Zum Inhalt springen" is a common,
        // entirely idiomatic German skip-link phrasing — it contains neither
        // "skip" nor "überspring", the two substrings the check used to
        // require. A real, working skip link must not trigger the tab-stop
        // warning just because of that narrower keyword list.
        let mut items = vec![item(0, "link", Some("Zum Inhalt springen"), true, vec![])];
        for seq in 1..60 {
            items.push(item(
                seq,
                "link",
                Some(&format!("Link {seq}")),
                true,
                vec![],
            ));
        }
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let tab_stop_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.wcag_criterion.as_deref() == Some("2.4.1"))
            .collect();
        assert!(
            tab_stop_issues.is_empty(),
            "a real skip link phrased as 'Zum Inhalt springen' must suppress the \
             tab-stop warning; got {tab_stop_issues:?}"
        );
    }

    #[test]
    fn detects_missing_required_landmarks() {
        let items = vec![item(0, "main", Some("Inhalt"), false, vec![])];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let landmark_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.wcag_criterion.as_deref() == Some("1.3.6"))
            .collect();
        assert_eq!(
            landmark_issues.len(),
            3,
            "expected issues for banner, navigation, contentinfo"
        );
    }

    #[test]
    fn downgrades_missing_navigation_when_disclosure_menu_pattern_recognized() {
        let items = vec![item(0, "main", Some("Inhalt"), false, vec![])];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, true);

        let nav_issue = issues
            .iter()
            .find(|i| i.wcag_criterion.as_deref() == Some("1.3.6") && i.message.contains("Menü"))
            .expect("expected a downgraded navigation issue");
        assert_eq!(nav_issue.severity, "low");

        // Banner/contentinfo are still reported at their normal severity.
        assert!(issues
            .iter()
            .any(|i| i.wcag_criterion.as_deref() == Some("1.3.6") && i.severity == "medium"));
    }

    #[test]
    fn no_landmark_issues_when_all_required_present() {
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Main nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(!issues
            .iter()
            .any(|i| i.wcag_criterion.as_deref() == Some("1.3.6")));
    }

    #[test]
    fn detects_icon_font_pua_in_link_name() {
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("\u{E003}"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(issues.iter().any(|i| i.message.contains("Icon-Font")));
    }

    #[test]
    fn does_not_flag_link_with_normal_text() {
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("Kontakt"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(!issues.iter().any(|i| i.message.contains("Icon-Font")));
    }

    #[test]
    fn ambiguous_continuation_word_is_downgraded_to_low_severity() {
        // plan/33-screen-reader-thresholds-unvalidated.md: a wizard's
        // "Weiter" (Next) button is common and often clear from its
        // surrounding context, which this flat item list can't see — flag
        // it, but not at the same severity as a name that is never
        // meaningful in any context.
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "button", Some("Weiter"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let issue = issues
            .iter()
            .find(|i| i.wcag_criterion.as_deref() == Some("2.4.4"))
            .expect("expected a non-descriptive-name issue for \"Weiter\"");
        assert_eq!(issue.severity, "low");
    }

    #[test]
    fn click_mechanism_word_stays_at_medium_severity() {
        // Unlike "Weiter"/"Mehr", a name that only describes the act of
        // clicking ("hier klicken"/"click here") carries no meaning
        // regardless of context and must stay at the higher severity.
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("Hier klicken"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let issue = issues
            .iter()
            .find(|i| i.wcag_criterion.as_deref() == Some("2.4.4"))
            .expect("expected a non-descriptive-name issue for \"Hier klicken\"");
        assert_eq!(issue.severity, "medium");
    }

    #[test]
    fn long_article_with_periodic_links_does_not_trigger_a_desert() {
        // A realistic long-form article: one heading, then paragraphs that
        // periodically contain an inline citation link — the link's
        // `tab_stop` resets the desert counter, same as a landmark/heading
        // would (plan/33-screen-reader-thresholds-unvalidated.md, scenario
        // "long clean single-topic article").
        let mut items = vec![item(0, "heading", Some("Artikel"), false, vec!["level=1"])];
        let mut seq = 1;
        for para in 0..20 {
            items.push(item(
                seq,
                "StaticText",
                Some(&format!("Absatz {para} mit echtem Flie\u{df}text.")),
                false,
                vec![],
            ));
            seq += 1;
            if para % 3 == 0 {
                items.push(item(seq, "link", Some("Quelle"), true, vec![]));
                seq += 1;
            }
        }
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", true, false);

        assert!(
            !issues.iter().any(|i| i.message.contains("Long section")),
            "periodic inline links should reset the desert counter: {:?}",
            issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn detects_h2_before_h1() {
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
            item(57, "heading", Some("Sub"), false, vec!["level=2"]),
            item(128, "heading", Some("Title"), false, vec!["level=1"]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let order_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.wcag_criterion.as_deref() == Some("1.3.1"))
            .collect();
        // "erste Überschrift nicht H1" + "H1 erscheint nach H2"
        assert!(
            order_issues.len() >= 2,
            "expected heading-order issues, got: {:?}",
            order_issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_heading_order_issue_when_h1_is_first() {
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
            item(10, "heading", Some("Title"), false, vec!["level=1"]),
            item(20, "heading", Some("Sub"), false, vec!["level=2"]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(!issues.iter().any(|i| {
            i.wcag_criterion.as_deref() == Some("1.3.1")
                && (i.message.contains("erste Überschrift") || i.message.contains("H1 erscheint"))
        }));
    }

    #[test]
    fn detects_duplicated_accessible_name_on_footer_style_link() {
        // Regression fixture resembling the confirmed live bug
        // (shop.satower-mosterei.de, 2026-09-04): a footer link's resolved
        // accessible name is the visible text repeated twice back-to-back
        // (e.g. a labelled icon glyph concatenated with adjacent visible
        // text via name-from-content), so a screen reader announces
        // "Kontakt Kontakt".
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
            item(4, "link", Some("Kontakt Kontakt"), true, vec![]),
            item(5, "link", Some("AGB AGB"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        let duplicated: Vec<_> = issues
            .iter()
            .filter(|i| i.message.contains("wiederholt"))
            .collect();
        assert_eq!(
            duplicated.len(),
            2,
            "expected a duplicated-name issue for both links, got: {:?}",
            issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
        assert!(duplicated
            .iter()
            .all(|i| i.wcag_criterion.as_deref() == Some("2.4.4")));
        assert!(duplicated
            .iter()
            .any(|i| i.affected_node_ids == vec!["node-4".to_string()]));
        assert!(duplicated
            .iter()
            .any(|i| i.affected_node_ids == vec!["node-5".to_string()]));
    }

    #[test]
    fn does_not_flag_generic_stopword_repeated_as_duplicated_name() {
        // FP guardrail: a doubled *generic* word (already covered, more
        // usefully, by the non-descriptive-name check) must not also fire
        // the duplicated-name check -- "Mehr" is in the German
        // linktext-generic-stopwords list.
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("Mehr Mehr"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(!issues.iter().any(|i| i.message.contains("wiederholt")));
    }

    #[test]
    fn does_not_flag_non_duplicate_name_with_shared_prefix() {
        // FP guardrail: names must split into two *identical* halves, not
        // merely share a word -- "Kontakt Kontaktformular" is a normal,
        // non-repeating name and must not be flagged.
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("Kontakt Kontaktformular"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de", false, false);

        assert!(!issues.iter().any(|i| i.message.contains("wiederholt")));
    }
}
