//! Linktext-Inventur + Heading-Outline + Landmark-Inventar (Stufe B).
//!
//! Pure AXTree analysis — no browser interaction required. Runs after the
//! initial AXTree extraction. Detects:
//!
//! - Generic / ambiguous link names ("Mehr erfahren", "Read more", "here", …)
//! - Duplicate link names with different destinations (not detectable without
//!   DOM href access — we flag duplicates regardless of destination as a
//!   conservative proxy)
//! - Missing or duplicate landmark regions
//! - Heading-level skips (h1 → h3 without h2)
//!
//! Results are returned as `InteractiveFinding`s (category "LinkText",
//! "Landmark", "HeadingOutline") with severity Warning or Medium so they
//! do not inflate legal_flags.

use std::collections::HashMap;

use crate::accessibility::AXTree;
use crate::audit::normalized::InteractiveFinding;
use crate::taxonomy::Severity;

/// Generic link-text patterns to flag — case-insensitive substring match.
/// Bilingual (DE + EN) to cover common European sites without i18n overhead.
const GENERIC_LINK_TEXTS: &[&str] = &[
    // German
    "mehr erfahren",
    "weiterlesen",
    "hier klicken",
    "hier",
    "mehr",
    "weiter",
    "lesen",
    "anzeigen",
    "öffnen",
    "details",
    "link",
    "klicken",
    // English
    "read more",
    "learn more",
    "click here",
    "here",
    "more",
    "details",
    "link",
    "view",
    "open",
    "see more",
    "find out more",
    "discover",
];

/// Landmark roles that should appear exactly once without a label when there
/// is only one instance, or with distinct labels when there are multiple.
const UNIQUE_LANDMARKS: &[&str] = &["main", "banner", "contentinfo"];

/// Returns `true` if the name matches a generic link text.
fn is_generic(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    // Exact match or the name *is* exactly one of the generic phrases.
    GENERIC_LINK_TEXTS
        .iter()
        .any(|g| lower == *g || lower.contains(*g))
}

/// Analyse link texts, heading outline, and landmark inventory from an AXTree.
///
/// Returns a list of `InteractiveFinding`s. The journey name is set to
/// `"link_inventory"` for all findings from this function.
pub fn analyse(tree: &AXTree) -> Vec<InteractiveFinding> {
    let mut findings = Vec::new();

    findings.extend(check_link_texts(tree));
    findings.extend(check_heading_outline(tree));
    findings.extend(check_landmarks(tree));

    findings
}

fn check_link_texts(tree: &AXTree) -> Vec<InteractiveFinding> {
    let mut findings = Vec::new();
    let links = tree.links();

    if links.is_empty() {
        return findings;
    }

    // Collect generic-text offenders.
    let mut generic: Vec<String> = Vec::new();
    // Count occurrences per name (lowercase) for duplicate detection.
    let mut name_counts: HashMap<String, usize> = HashMap::new();

    for link in &links {
        let name = link.name.as_deref().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue; // Missing name is a WCAG violation, not a link-text issue.
        }
        let lower = name.to_lowercase();
        *name_counts.entry(lower.clone()).or_insert(0) += 1;
        if is_generic(&name) {
            generic.push(name);
        }
    }

    // Aggregate generic findings into one finding per page.
    if !generic.is_empty() {
        let count = generic.len();
        let examples: Vec<_> = {
            let mut seen = std::collections::HashSet::new();
            generic
                .iter()
                .filter(|n| seen.insert(n.to_lowercase()))
                .take(5)
                .map(|s| format!("'{s}'"))
                .collect()
        };
        let example_str = examples.join(", ");
        findings.push(InteractiveFinding {
            category: "LinkText".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: format!(
                "{count} Link(s) tragen einen generischen oder nichtssagenden Text \
                ({example_str}). Ohne sichtbaren Kontext sind sie für Screenreader-Nutzer \
                nicht unterscheidbar und erfüllen WCAG 2.4.4 nicht."
            ),
            fix_suggestion: Some(
                "Linktexte so formulieren, dass sie auch ohne den umgebenden Seitentext \
                verstaendlich sind, z. B. 'Mehr erfahren ueber Barrierefreiheit' statt \
                'Mehr erfahren'."
                    .to_string(),
            ),
        });
    }

    // Duplicate link names — aggregate once.
    let duplicates: Vec<_> = name_counts
        .iter()
        .filter(|(name, &count)| count >= 3 && !is_generic(name))
        .map(|(name, count)| (name.clone(), *count))
        .collect();

    if !duplicates.is_empty() {
        let count = duplicates.len();
        let examples: Vec<_> = duplicates
            .iter()
            .take(3)
            .map(|(n, c)| format!("'{n}' (x{c})"))
            .collect();
        findings.push(InteractiveFinding {
            category: "LinkText".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: format!(
                "{count} Linktext(e) erscheinen 3 oder mehr Mal auf der Seite: {}. \
                Wenn sie zu unterschiedlichen Zielen führen, können Screenreader-Nutzer \
                sie nicht unterscheiden.",
                examples.join(", ")
            ),
            fix_suggestion: Some(
                "Wiederholte Linktexte durch eindeutige Formulierungen ersetzen oder \
                mit aria-label / aria-labelledby den sichtbaren Text ergänzen."
                    .to_string(),
            ),
        });
    }

    findings
}

fn check_heading_outline(tree: &AXTree) -> Vec<InteractiveFinding> {
    let mut findings = Vec::new();
    let mut headings = tree.headings();

    if headings.is_empty() {
        return findings;
    }

    // Sort by DOM order via node_id (lexicographic is approximate but sufficient).
    headings.sort_by_key(|h| h.node_id.as_str());

    let levels: Vec<u8> = headings.iter().filter_map(|h| h.heading_level()).collect();

    // Check: no h1.
    if !levels.iter().any(|&l| l == 1) {
        findings.push(InteractiveFinding {
            category: "HeadingOutline".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Die Seite enthält keine H1-Überschrift. Screenreader-Nutzer können \
                ohne H1 die Hauptstruktur der Seite nicht erfassen."
                .to_string(),
            fix_suggestion: Some(
                "Exakt eine H1-Überschrift pro Seite verwenden, die den Hauptinhalt \
                beschreibt."
                    .to_string(),
            ),
        });
    }

    // Check: multiple h1.
    let h1_count = levels.iter().filter(|&&l| l == 1).count();
    if h1_count > 1 {
        findings.push(InteractiveFinding {
            category: "HeadingOutline".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: format!(
                "{h1_count} H1-Überschriften gefunden. Mehrere H1-Elemente erschweren \
                Screenreader-Nutzern die Orientierung."
            ),
            fix_suggestion: Some(
                "Nur eine H1-Überschrift pro Seite verwenden. Weitere Hauptüberschriften \
                als H2 auszeichnen."
                    .to_string(),
            ),
        });
    }

    // Check: level skips (e.g. h1 -> h3 without h2).
    let mut skips: Vec<(u8, u8)> = Vec::new();
    let mut prev: u8 = 0;
    for &level in &levels {
        if level > prev.saturating_add(1) && prev != 0 {
            skips.push((prev, level));
        }
        prev = level;
    }

    if !skips.is_empty() {
        let examples: Vec<_> = skips
            .iter()
            .take(3)
            .map(|(from, to)| format!("H{from}→H{to}"))
            .collect();
        findings.push(InteractiveFinding {
            category: "HeadingOutline".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: format!(
                "Heading-Hierarchie überspringt Ebenen ({}). Screenreader-Nutzer können \
                dadurch die Seitenstruktur nicht verlässlich erfassen.",
                examples.join(", ")
            ),
            fix_suggestion: Some(
                "Überschriften-Ebenen nie überspringen. Nach einer H1 folgt H2, \
                nach H2 folgt H3 usw."
                    .to_string(),
            ),
        });
    }

    findings
}

fn check_landmarks(tree: &AXTree) -> Vec<InteractiveFinding> {
    let mut findings = Vec::new();

    // Count unique landmark roles.
    let landmark_roles = [
        "main",
        "navigation",
        "banner",
        "contentinfo",
        "complementary",
        "search",
        "form",
        "region",
    ];

    let mut role_counts: HashMap<&str, usize> = HashMap::new();
    let mut role_named_counts: HashMap<&str, usize> = HashMap::new();

    for node in tree.iter() {
        let role = node.role.as_deref().unwrap_or("");
        if !landmark_roles.contains(&role) {
            continue;
        }
        *role_counts.entry(role).or_insert(0) += 1;
        if node.has_name() {
            *role_named_counts.entry(role).or_insert(0) += 1;
        }
    }

    // main must exist.
    if role_counts.get("main").copied().unwrap_or(0) == 0 {
        findings.push(InteractiveFinding {
            category: "Landmark".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Kein <main>-Landmark gefunden. Screenreader-Nutzer können den \
                Hauptinhalt nicht direkt anspringen."
                .to_string(),
            fix_suggestion: Some(
                "Den Hauptinhalt in einem <main>-Element wrappen oder role=\"main\" \
                auf dem entsprechenden Container setzen."
                    .to_string(),
            ),
        });
    }

    // Multiple navigation landmarks should each have a distinct accessible name.
    let nav_count = role_counts.get("navigation").copied().unwrap_or(0);
    let nav_named = role_named_counts.get("navigation").copied().unwrap_or(0);
    if nav_count > 1 && nav_named < nav_count {
        findings.push(InteractiveFinding {
            category: "Landmark".to_string(),
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: format!(
                "{nav_count} Navigation-Landmarks ohne eindeutige Beschriftungen. \
                Screenreader-Nutzer können nicht unterscheiden, welche Navigation \
                welchen Bereich abdeckt."
            ),
            fix_suggestion: Some(
                "Jede <nav>-Region mit einem aria-label benennen, \
                z. B. aria-label=\"Hauptnavigation\" und aria-label=\"Footer-Navigation\"."
                    .to_string(),
            ),
        });
    }

    // Unique landmarks must appear at most once.
    for role in UNIQUE_LANDMARKS {
        let count = role_counts.get(role).copied().unwrap_or(0);
        if count > 1 {
            findings.push(InteractiveFinding {
                category: "Landmark".to_string(),
                severity: Severity::Medium,
                journey: "link_inventory".to_string(),
                before_snapshot_label: None,
                after_snapshot_label: None,
                message: format!(
                    "Landmark-Rolle \"{role}\" erscheint {count}× auf der Seite. \
                    Diese Rolle sollte pro Seite nur einmal vorkommen."
                ),
                fix_suggestion: Some(format!(
                    "Nur ein Element mit role=\"{role}\" (oder dem entsprechenden \
                    HTML-Element) pro Seite verwenden."
                )),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_link(name: &str) -> AXNode {
        AXNode {
            node_id: name.to_string(),
            role: Some("link".to_string()),
            name: Some(name.to_string()),
            ..Default::default()
        }
    }

    fn make_heading(level: u8, name: &str) -> AXNode {
        use crate::accessibility::{AXProperty, AXValue};
        AXNode {
            node_id: name.to_string(),
            role: Some("heading".to_string()),
            name: Some(name.to_string()),
            properties: vec![AXProperty {
                name: "level".to_string(),
                value: AXValue::Int(level as i64),
            }],
            ..Default::default()
        }
    }

    fn make_landmark(role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: role.to_string(),
            role: Some(role.to_string()),
            name: name.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn tree_from(nodes: Vec<AXNode>) -> AXTree {
        AXTree::from_nodes(nodes)
    }

    #[test]
    fn generic_link_text_is_flagged() {
        let tree = tree_from(vec![
            make_link("mehr erfahren"),
            make_link("mehr erfahren"),
            make_link("Produkt A"),
        ]);
        let findings = check_link_texts(&tree);
        assert!(findings.iter().any(|f| f.category == "LinkText"));
    }

    #[test]
    fn clean_link_texts_produce_no_findings() {
        let tree = tree_from(vec![
            make_link("Produkt A kaufen"),
            make_link("Barrierefreiheit verbessern"),
        ]);
        let findings = check_link_texts(&tree);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn missing_h1_is_flagged() {
        let tree = tree_from(vec![make_heading(2, "Unterabschnitt")]);
        let findings = check_heading_outline(&tree);
        assert!(findings
            .iter()
            .any(|f| f.category == "HeadingOutline" && f.message.contains("keine H1")));
    }

    #[test]
    fn heading_level_skip_is_flagged() {
        let tree = tree_from(vec![
            make_heading(1, "Haupttitel"),
            make_heading(3, "Unterunter"),
        ]);
        let findings = check_heading_outline(&tree);
        assert!(findings
            .iter()
            .any(|f| f.category == "HeadingOutline" && f.message.contains("H1")));
    }

    #[test]
    fn missing_main_landmark_is_flagged() {
        let tree = tree_from(vec![make_landmark("navigation", Some("Hauptnavigation"))]);
        let findings = check_landmarks(&tree);
        assert!(findings
            .iter()
            .any(|f| f.category == "Landmark" && f.message.contains("main")));
    }

    #[test]
    fn clean_landmarks_produce_no_findings() {
        let tree = tree_from(vec![
            make_landmark("main", None),
            make_landmark("banner", None),
        ]);
        let findings = check_landmarks(&tree);
        assert!(findings.is_empty(), "{findings:#?}");
    }
}
