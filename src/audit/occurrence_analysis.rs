//! Occurrence analysis — domain logic that selects representative examples and
//! groups recurring selector patterns for a finding's occurrences.
//!
//! These heuristics (which example is most actionable? which occurrences share a
//! structural pattern?) are locale-neutral rules over the domain type
//! [`OccurrenceDetail`]. They used to live in `output/builder/single/findings.rs`,
//! which made the presentation layer the home of the ranking/grouping rules; the
//! builder now only maps the results into the view model. `report_model`
//! re-exports the result structs for backward compatibility.

use crate::audit::normalized::OccurrenceDetail;
use crate::wcag::ViolationEvidence;

/// One representative occurrence chosen to illustrate a finding.
pub struct RepresentativeOccurrence {
    pub selector: String,
    pub node_id: String,
    pub message: String,
    pub html_snippet: Option<String>,
    pub suggested_code: Option<String>,
    /// Machine-readable provenance (DOM path, computed measurements) carried
    /// through from `OccurrenceDetail::evidence` (evidence-grade findings).
    pub evidence: Vec<ViolationEvidence>,
    /// Cropped element screenshot, if one was captured for this occurrence's
    /// rule. In-memory only — never serialized.
    pub evidence_screenshot: Option<Vec<u8>>,
    /// Which viewport pass produced `evidence_screenshot`.
    pub evidence_viewport: Option<&'static str>,
}

/// A group of occurrences sharing a normalized selector pattern.
pub struct FindingPatternCluster {
    pub label: String,
    pub occurrences: usize,
}

/// Up to five distinct location hints (selector, or `AX-Node <id>` fallback).
pub fn build_location_hints(occurrences: &[OccurrenceDetail]) -> Vec<String> {
    let mut hints = Vec::new();
    for occ in occurrences {
        let hint = if let Some(selector) = &occ.selector {
            selector.trim().to_string()
        } else {
            format!("AX-Node {}", occ.node_id)
        };
        if !hint.is_empty() && !hints.contains(&hint) {
            hints.push(hint);
        }
        if hints.len() >= 5 {
            break;
        }
    }
    hints
}

/// The top three most actionable, selector-distinct occurrences for a finding.
pub fn build_representative_occurrences(
    occurrences: &[OccurrenceDetail],
) -> Vec<RepresentativeOccurrence> {
    let mut ranked: Vec<(usize, i32, &OccurrenceDetail)> = occurrences
        .iter()
        .enumerate()
        .map(|(index, occ)| (index, representative_occurrence_score(occ), occ))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| b.2.message.len().cmp(&a.2.message.len()))
    });

    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for (_, _, occ) in ranked {
        let selector = representative_selector(occ);

        if !seen.insert(selector.to_ascii_lowercase()) {
            continue;
        }

        items.push(RepresentativeOccurrence {
            selector,
            node_id: occ.node_id.clone(),
            message: occ.message.clone(),
            html_snippet: occ.html_snippet.clone(),
            suggested_code: occ.suggested_code.clone(),
            evidence: occ.evidence.clone(),
            evidence_screenshot: occ.evidence_screenshot.clone(),
            evidence_viewport: occ.evidence_viewport,
        });

        if items.len() >= 3 {
            break;
        }
    }

    items
}

/// The top three structural selector patterns across a finding's occurrences.
pub fn build_pattern_clusters(occurrences: &[OccurrenceDetail]) -> Vec<FindingPatternCluster> {
    let mut clusters: std::collections::BTreeMap<String, (String, usize)> =
        std::collections::BTreeMap::new();

    for occ in occurrences {
        let selector = representative_selector(occ);
        let normalized = normalize_selector_cluster(&selector);
        let entry = clusters.entry(normalized).or_insert((selector.clone(), 0));
        entry.1 += 1;

        if selector.len() < entry.0.len() {
            entry.0 = selector;
        }
    }

    let mut items: Vec<FindingPatternCluster> = clusters
        .into_values()
        .map(|(label, occurrences)| FindingPatternCluster { label, occurrences })
        .collect();
    items.sort_by(|a, b| {
        b.occurrences
            .cmp(&a.occurrences)
            .then_with(|| a.label.len().cmp(&b.label.len()))
    });
    items.truncate(3);
    items
}

fn representative_selector(occ: &OccurrenceDetail) -> String {
    occ.selector
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&occ.node_id)
        .to_string()
}

pub(crate) fn normalize_selector_cluster(selector: &str) -> String {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return "unspecified".to_string();
    }

    let normalized: String = trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_digit() {
                '#'
            } else if ch.is_whitespace() {
                ' '
            } else {
                ch.to_ascii_lowercase()
            }
        })
        .collect();

    let mut compact = String::new();
    let mut last_was_hash = false;
    let mut last_was_space = false;
    for ch in normalized.chars() {
        match ch {
            '#' => {
                if !last_was_hash {
                    compact.push(ch);
                }
                last_was_hash = true;
                last_was_space = false;
            }
            ' ' => {
                if !last_was_space {
                    compact.push(ch);
                }
                last_was_space = true;
                last_was_hash = false;
            }
            _ => {
                compact.push(ch);
                last_was_hash = false;
                last_was_space = false;
            }
        }
    }

    compact
}

fn representative_occurrence_score(occ: &OccurrenceDetail) -> i32 {
    let mut score = 0;

    let selector = occ
        .selector
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(selector) = selector {
        score += 40;
        score += selector_quality_score(selector);
    } else {
        score -= 12;
    }

    if has_content(&occ.html_snippet) {
        score += 28;
    }
    if has_content(&occ.suggested_code) {
        score += 22;
    }
    if has_content(&occ.fix_suggestion) {
        score += 8;
    }

    let message = occ.message.trim();
    if !message.is_empty() {
        score += 6;
        score += (message.len().min(120) / 20) as i32;
        if message.contains(':') || message.contains('(') {
            score += 3;
        }
        if message.chars().any(|ch| ch.is_ascii_digit()) {
            score += 4;
        }
    }

    if !occ.node_id.trim().is_empty() {
        score += 2;
    }

    score
}

fn selector_quality_score(selector: &str) -> i32 {
    let mut score = 0;

    if selector.contains('#') {
        score += 16;
    }
    if selector.contains('[') {
        score += 10;
    }
    if selector.contains('.') {
        score += 8;
    }
    if selector.contains('>') {
        score += 6;
    }
    if selector.contains(' ') {
        score += 4;
    }
    if selector.starts_with("main")
        || selector.starts_with("header")
        || selector.starts_with("nav")
        || selector.starts_with("footer")
    {
        score += 4;
    }

    score
}

fn has_content(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|text| !text.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{
        build_pattern_clusters, build_representative_occurrences, normalize_selector_cluster,
    };
    use crate::audit::normalized::OccurrenceDetail;

    #[test]
    fn representative_occurrences_prefer_rich_and_actionable_examples() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "Short".into(),
                selector: None,
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Contrast ratio 1.13:1 for hero headline.".into(),
                selector: Some("main .hero-title".into()),
                fix_suggestion: Some("Increase foreground/background contrast.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Insights</h1>".into()),
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Contrast ratio 1.00:1 for CTA button text.".into(),
                selector: Some("#cta-primary".into()),
                fix_suggestion: Some("Use darker text color.".into()),
                html_snippet: Some("<a id=\"cta-primary\">Kontakt</a>".into()),
                suggested_code: Some(
                    "<a id=\"cta-primary\" class=\"text-stone-900\">Kontakt</a>".into(),
                ),
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-4".into(),
                message: "Link landmark is outside a region.".into(),
                selector: Some("a.skip-link".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
        ];

        let selected = build_representative_occurrences(&occurrences);

        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].selector, "#cta-primary");
        assert_eq!(selected[1].selector, "main .hero-title");
        assert_eq!(selected[2].selector, "a.skip-link");
    }

    #[test]
    fn representative_occurrences_deduplicate_selector_variants() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "First duplicate".into(),
                selector: Some("main .hero-title".into()),
                fix_suggestion: None,
                html_snippet: Some("<h1 class=\"hero-title\">One</h1>".into()),
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Second duplicate with richer text".into(),
                selector: Some("MAIN .HERO-TITLE".into()),
                fix_suggestion: Some("Adjust markup.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Two</h1>".into()),
                suggested_code: Some("<h1 lang=\"de\">Two</h1>".into()),
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Independent selector".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
        ];

        let selected = build_representative_occurrences(&occurrences);

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].selector, "MAIN .HERO-TITLE");
        assert_eq!(selected[1].selector, "footer .meta a");
    }

    #[test]
    fn pattern_clusters_group_similar_selector_variants() {
        let occurrences = vec![
            OccurrenceDetail {
                node_id: "node-1".into(),
                message: "One".into(),
                selector: Some("main .card-1 .cta".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Two".into(),
                selector: Some("main .card-2 .cta".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Three".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
                ..Default::default()
            },
        ];

        let clusters = build_pattern_clusters(&occurrences);

        assert_eq!(
            normalize_selector_cluster("main .card-1 .cta"),
            "main .card-# .cta"
        );
        assert_eq!(clusters[0].occurrences, 2);
        assert_eq!(clusters[0].label, "main .card-1 .cta");
    }
}
