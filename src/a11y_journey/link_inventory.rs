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
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues,
};
use crate::i18n::I18n;
use crate::taxonomy::Severity;

/// Load generic-linktext stopwords from FTL for the given locale.
/// Falls back to an empty list if the key is missing.
fn stopwords_for_locale(locale: &str) -> Vec<String> {
    let Ok(i18n) = I18n::new(locale) else {
        return Vec::new();
    };
    let raw = i18n.t("linktext-generic-stopwords");
    if raw == "linktext-generic-stopwords" {
        return Vec::new(); // key missing — I18n returns the key itself as fallback
    }
    raw.split(',').map(|s| s.trim().to_string()).collect()
}

/// All known locales whose stopwords are always checked, regardless of report
/// language. Sites frequently mix languages; merging keeps current behaviour.
const SUPPORTED_LOCALES: &[&str] = &["de", "en"];

/// Landmark roles that should appear exactly once without a label when there
/// is only one instance, or with distinct labels when there are multiple.
const UNIQUE_LANDMARKS: &[&str] = &["main", "banner", "contentinfo"];

/// Returns `true` if the name matches any word in the stopword list.
fn is_generic(name: &str, stopwords: &[String]) -> bool {
    let lower = name.trim().to_lowercase();
    stopwords
        .iter()
        .any(|g| lower == g.as_str() || lower.contains(g.as_str()))
}

/// Build merged stopword list from all supported locales.
fn load_stopwords() -> Vec<String> {
    let mut words: Vec<String> = SUPPORTED_LOCALES
        .iter()
        .flat_map(|loc| stopwords_for_locale(loc))
        .collect();
    words.sort_unstable();
    words.dedup();
    words
}

/// Analyse link texts, heading outline, and landmark inventory from an AXTree.
///
/// `_locale` is reserved for future per-locale filtering; currently all
/// supported locales are always merged so bilingual sites are covered.
pub fn analyse(tree: &AXTree, _locale: &str) -> Vec<InteractiveFinding> {
    let stopwords = load_stopwords();
    let mut findings = Vec::new();

    findings.extend(check_link_texts(tree, &stopwords));
    findings.extend(check_heading_outline(tree));
    findings.extend(check_landmarks(tree));

    findings
}

fn check_link_texts(tree: &AXTree, stopwords: &[String]) -> Vec<InteractiveFinding> {
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
        if is_generic(&name, stopwords) {
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
        findings.push(InteractiveFinding::new(
            "LinkText",
            InteractiveFindingKind::LinkTextGeneric,
            Some("a11y.link_purpose.weak".to_string()),
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues {
                count: Some(count as u32),
                examples: Some(example_str),
                ..Default::default()
            },
        ));
    }

    // Duplicate link names — aggregate once.
    let duplicates: Vec<_> = name_counts
        .iter()
        .filter(|(name, &count)| count >= 3 && !is_generic(name, stopwords))
        .map(|(name, count)| (name.clone(), *count))
        .collect();

    if !duplicates.is_empty() {
        let count = duplicates.len();
        let examples: Vec<_> = duplicates
            .iter()
            .take(3)
            .map(|(n, c)| format!("'{n}' (x{c})"))
            .collect();
        findings.push(InteractiveFinding::new(
            "LinkText",
            InteractiveFindingKind::LinkTextDuplicate,
            Some("a11y.link_purpose.weak".to_string()),
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues {
                count: Some(count as u32),
                examples: Some(examples.join(", ")),
                ..Default::default()
            },
        ));
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
    if !levels.contains(&1) {
        findings.push(InteractiveFinding::new(
            "HeadingOutline",
            InteractiveFindingKind::HeadingMissingH1,
            None,
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues::default(),
        ));
    }

    // Check: multiple h1.
    let h1_count = levels.iter().filter(|&&l| l == 1).count();
    if h1_count > 1 {
        findings.push(InteractiveFinding::new(
            "HeadingOutline",
            InteractiveFindingKind::HeadingMultipleH1,
            None,
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues {
                count: Some(h1_count as u32),
                ..Default::default()
            },
        ));
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
        findings.push(InteractiveFinding::new(
            "HeadingOutline",
            InteractiveFindingKind::HeadingLevelSkip,
            None,
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues {
                examples: Some(examples.join(", ")),
                ..Default::default()
            },
        ));
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
        findings.push(InteractiveFinding::new(
            "Landmark",
            InteractiveFindingKind::LandmarkMissingMain,
            None,
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues::default(),
        ));
    }

    // Multiple navigation landmarks should each have a distinct accessible name.
    let nav_count = role_counts.get("navigation").copied().unwrap_or(0);
    let nav_named = role_named_counts.get("navigation").copied().unwrap_or(0);
    if nav_count > 1 && nav_named < nav_count {
        findings.push(InteractiveFinding::new(
            "Landmark",
            InteractiveFindingKind::LandmarkNavWithoutLabels,
            None,
            Severity::Medium,
            "link_inventory".to_string(),
            None,
            None,
            InteractiveFindingValues {
                count: Some(nav_count as u32),
                ..Default::default()
            },
        ));
    }

    // Unique landmarks must appear at most once.
    for role in UNIQUE_LANDMARKS {
        let count = role_counts.get(role).copied().unwrap_or(0);
        if count > 1 {
            findings.push(InteractiveFinding::new(
                "Landmark",
                InteractiveFindingKind::LandmarkDuplicateUnique,
                None,
                Severity::Medium,
                "link_inventory".to_string(),
                None,
                None,
                InteractiveFindingValues {
                    role: Some(role.to_string()),
                    count: Some(count as u32),
                    ..Default::default()
                },
            ));
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
        let stopwords = load_stopwords();
        let tree = tree_from(vec![
            make_link("mehr erfahren"),
            make_link("mehr erfahren"),
            make_link("Produkt A"),
        ]);
        let findings = check_link_texts(&tree, &stopwords);
        assert!(findings.iter().any(|f| f.category == "LinkText"));
    }

    #[test]
    fn clean_link_texts_produce_no_findings() {
        let stopwords = load_stopwords();
        let tree = tree_from(vec![
            make_link("Produkt A kaufen"),
            make_link("Barrierefreiheit verbessern"),
        ]);
        let findings = check_link_texts(&tree, &stopwords);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn missing_h1_is_flagged() {
        let tree = tree_from(vec![make_heading(2, "Unterabschnitt")]);
        let findings = check_heading_outline(&tree);
        assert!(findings
            .iter()
            .any(|f| f.category == "HeadingOutline" && f.message.contains("no H1")));
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
