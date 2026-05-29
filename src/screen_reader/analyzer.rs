use std::collections::HashSet;

use crate::i18n::I18n;

use super::navigator::{
    FormControlQuality, HeadingQuality, LandmarkQuality, LinkQuality, NavigationViews,
};
use super::types::{ReadingItem, SrAuditIssue};

const ANNOUNCEMENT_DESERT_THRESHOLD: usize = 15;
const TAB_STOP_WARNING_THRESHOLD: usize = 50;

pub fn analyze_reading_sequence(
    items: &[ReadingItem],
    views: &NavigationViews,
    locale: &str,
) -> Vec<SrAuditIssue> {
    let stopwords = localized_stopwords(locale);
    let mut issues = Vec::new();

    detect_non_descriptive_interactive_names(items, &stopwords, &mut issues);
    detect_icon_font_contamination(items, &mut issues);
    detect_duplicate_link_texts(views, &mut issues);
    detect_announcement_deserts(items, &mut issues);
    detect_skipped_heading_levels(views, &mut issues);
    detect_heading_order_issues(views, &mut issues);
    detect_missing_required_landmarks(views, &mut issues);
    detect_unlabeled_duplicate_landmarks(views, &mut issues);
    detect_tab_stop_count(items, &mut issues);
    detect_empty_interactive_elements(items, &mut issues);
    detect_empty_form_labels(views, &mut issues);

    issues
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

fn detect_non_descriptive_interactive_names(
    items: &[ReadingItem],
    stopwords: &HashSet<String>,
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
            issues.push(SrAuditIssue {
                wcag_criterion: Some("2.4.4".into()),
                severity: "medium".into(),
                affected_node_ids: vec![item.node_id.clone()],
                message: format!(
                    "Interaktiver Name \"{}\" ist ohne Kontext nicht aussagekräftig.",
                    name
                ),
            });
        }
    }
}

fn detect_duplicate_link_texts(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
    for link in views.links.iter().filter(|link| link.count > 1) {
        let quality = match link.quality {
            LinkQuality::Empty => "leeren Linktext",
            LinkQuality::NonDescriptive | LinkQuality::ContextDependent => {
                "kontextabhängigen Linktext"
            }
            LinkQuality::Good => continue,
        };
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.4".into()),
            severity: "low".into(),
            affected_node_ids: link.node_ids.clone(),
            message: format!(
                "{} kommt {} mal vor und erschwert die Linkliste im Screenreader.",
                quality, link.count
            ),
        });
    }
}

fn detect_announcement_deserts(items: &[ReadingItem], issues: &mut Vec<SrAuditIssue>) {
    let mut segment_start = 0usize;
    let mut count = 0usize;
    let mut node_ids = Vec::new();

    for item in items {
        if is_orientation_item(item) {
            if count > ANNOUNCEMENT_DESERT_THRESHOLD {
                push_desert_issue(segment_start, count, &node_ids, issues);
            }
            segment_start = item.seq + 1;
            count = 0;
            node_ids.clear();
        } else {
            count += 1;
            node_ids.push(item.node_id.clone());
        }
    }

    if count > ANNOUNCEMENT_DESERT_THRESHOLD {
        push_desert_issue(segment_start, count, &node_ids, issues);
    }
}

fn push_desert_issue(
    segment_start: usize,
    count: usize,
    node_ids: &[String],
    issues: &mut Vec<SrAuditIssue>,
) {
    issues.push(SrAuditIssue {
        wcag_criterion: None,
        severity: "low".into(),
        affected_node_ids: node_ids.to_vec(),
        message: format!(
            "Langer Abschnitt ab Sequenzposition {} ohne Landmark, Überschrift oder Fokusziel ({} Einträge).",
            segment_start, count
        ),
    });
}

fn detect_skipped_heading_levels(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
    for heading in views
        .headings
        .iter()
        .filter(|heading| heading.quality == HeadingQuality::SkippedLevel)
    {
        issues.push(SrAuditIssue {
            wcag_criterion: Some("2.4.6".into()),
            severity: "medium".into(),
            affected_node_ids: vec![heading.node_id.clone()],
            message: format!(
                "Überschriftenebene wird übersprungen: {:?} an Sequenzposition {}.",
                heading.level, heading.seq
            ),
        });
    }
}

fn detect_unlabeled_duplicate_landmarks(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
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
            message:
                "Mehrere gleichartige Landmarken ohne Namen sind in der Screenreader-Landmarkliste nicht unterscheidbar."
                    .into(),
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
            message: "Kein Main-Landmark in der Screenreader-Landmarkliste erkennbar.".into(),
        });
    }
}

fn detect_missing_required_landmarks(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
    const REQUIRED: &[(&str, &str)] = &[
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

    for (role, message) in REQUIRED {
        let present = views
            .landmarks
            .iter()
            .any(|l| l.role == *role && l.quality != LandmarkQuality::MissingMain);
        if !present {
            issues.push(SrAuditIssue {
                wcag_criterion: Some("1.3.6".into()),
                severity: "medium".into(),
                affected_node_ids: vec![],
                message: message.to_string(),
            });
        }
    }
}

fn detect_icon_font_contamination(items: &[ReadingItem], issues: &mut Vec<SrAuditIssue>) {
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
            message:
                "Link- oder Button-Name enthält Icon-Font-Zeichen (Unicode Private Use Area). Screen Reader lesen diese als kryptische Zeichencodes vor."
                    .into(),
        });
    }
}

fn contains_pua(text: &str) -> bool {
    text.chars().any(|c| ('\u{E000}'..='\u{F8FF}').contains(&c))
}

fn detect_heading_order_issues(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
    let headings = &views.headings;

    // First non-empty heading must be H1.
    if let Some(first) = headings.iter().find(|h| h.quality != HeadingQuality::Empty) {
        if first.level != Some(1) {
            issues.push(SrAuditIssue {
                wcag_criterion: Some("1.3.1".into()),
                severity: "medium".into(),
                affected_node_ids: vec![first.node_id.clone()],
                message: format!(
                    "Erste Überschrift im Dokument ist H{} statt H1 (Sequenzposition {}).",
                    first.level.unwrap_or(0),
                    first.seq
                ),
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
                message: format!(
                    "H1 erscheint erst an Sequenzposition {} — nach einer H2/H3-Überschrift.",
                    h1.seq
                ),
            });
        }
    }
}

fn detect_tab_stop_count(items: &[ReadingItem], issues: &mut Vec<SrAuditIssue>) {
    let tab_stop_count = items.iter().filter(|item| item.tab_stop).count();
    let has_skip_link = items.iter().any(|item| {
        item.role.as_deref() == Some("link")
            && item.name.as_deref().is_some_and(|name| {
                normalize_text(name).contains("skip") || normalize_text(name).contains("überspring")
            })
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
            message: format!(
                "{} Tab-Stops ohne erkennbaren Skip-Link erschweren Tastatur- und Screenreader-Navigation.",
                tab_stop_count
            ),
        });
    }
}

fn detect_empty_interactive_elements(items: &[ReadingItem], issues: &mut Vec<SrAuditIssue>) {
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
            message:
                "Interaktive Elemente ohne zugänglichen Namen werden im Screenreader nicht verständlich angekündigt."
                    .into(),
        });
    }
}

fn detect_empty_form_labels(views: &NavigationViews, issues: &mut Vec<SrAuditIssue>) {
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
            message: "Formularfelder ohne Label sind in der Screenreader-Formularliste nicht verständlich.".into(),
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
            )
        )
}

fn is_empty_name(name: &Option<String>) -> bool {
    name.as_deref().is_none_or(|name| name.trim().is_empty())
}

fn normalize_text(text: &str) -> String {
    text.trim().to_lowercase()
}

pub fn name_quality_score(items: &[ReadingItem], locale: &str) -> u32 {
    let stopwords = localized_stopwords(locale);
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
    fn detects_non_descriptive_and_empty_interactive_names() {
        let items = vec![
            item(0, "link", Some("Hier"), true, vec![]),
            item(1, "button", None, true, vec![]),
            item(2, "button", Some("Menü öffnen"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de");

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
        let issues = analyze_reading_sequence(&items, &views, "de");

        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("Landmarken")));
        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("Überschriftenebene")));
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
        let issues = analyze_reading_sequence(&items, &views, "de");

        assert!(issues.is_empty());
    }

    #[test]
    fn detects_missing_required_landmarks() {
        let items = vec![item(0, "main", Some("Inhalt"), false, vec![])];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de");

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
    fn no_landmark_issues_when_all_required_present() {
        let items = vec![
            item(0, "banner", Some("Header"), false, vec![]),
            item(1, "navigation", Some("Main nav"), false, vec![]),
            item(2, "main", Some("Inhalt"), false, vec![]),
            item(3, "contentinfo", Some("Footer"), false, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de");

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
        let issues = analyze_reading_sequence(&items, &views, "de");

        assert!(issues.iter().any(|i| i.message.contains("Icon-Font")));
    }

    #[test]
    fn does_not_flag_link_with_normal_text() {
        let items = vec![
            item(0, "main", Some("Inhalt"), false, vec![]),
            item(1, "link", Some("Kontakt"), true, vec![]),
        ];
        let views = navigation_views(&items);
        let issues = analyze_reading_sequence(&items, &views, "de");

        assert!(!issues.iter().any(|i| i.message.contains("Icon-Font")));
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
        let issues = analyze_reading_sequence(&items, &views, "de");

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
        let issues = analyze_reading_sequence(&items, &views, "de");

        assert!(!issues.iter().any(|i| {
            i.wcag_criterion.as_deref() == Some("1.3.1")
                && (i.message.contains("erste Überschrift") || i.message.contains("H1 erscheint"))
        }));
    }
}
