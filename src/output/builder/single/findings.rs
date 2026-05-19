use crate::audit::normalized::OccurrenceDetail;
use crate::output::explanations::get_explanation;
use crate::output::report_model::{
    Effort, FindingGroup, FindingPatternCluster, RepresentativeOccurrence, Role,
};

use super::super::actions::{
    derive_business_impact, derive_execution_priority, severity_to_priority,
};

pub(super) fn finding_group_from_normalized(
    locale: &str,
    f: &crate::audit::normalized::NormalizedFinding,
) -> FindingGroup {
    let explanation = get_explanation(&f.wcag_criterion);

    let (
        title,
        customer_desc,
        user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        role,
        effort,
        execution_priority,
    ) = if let Some(expl) = explanation {
        (
            expl.customer_title_for(locale).to_string(),
            expl.customer_description_for(locale).to_string(),
            expl.user_impact_for(locale).to_string(),
            derive_business_impact(
                locale,
                expl.user_impact_for(locale),
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
                f.occurrence_count,
            ),
            expl.typical_cause_for(locale).to_string(),
            expl.recommendation_for(locale).to_string(),
            expl.technical_note_for(locale).to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(f.severity, expl.effort_estimate, f.dimension.as_str()),
        )
    } else {
        (
            f.title.clone(),
            f.description.clone(),
            f.user_impact.clone(),
            derive_business_impact(
                locale,
                &f.user_impact,
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory.as_str()),
                f.occurrence_count,
            ),
            String::new(),
            f.occurrences
                .first()
                .and_then(|o| o.fix_suggestion.clone())
                .unwrap_or_default(),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(f.severity, Effort::Medium, f.dimension.as_str()),
        )
    };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = build_location_hints(&f.occurrences);
    let representative_occurrences = build_representative_occurrences(&f.occurrences);
    let pattern_clusters = build_pattern_clusters(&f.occurrences);
    let additional_occurrences = f
        .occurrence_count
        .saturating_sub(representative_occurrences.len());

    FindingGroup {
        title,
        rule_id: f.rule_id.clone(),
        wcag_criterion: f.wcag_criterion.clone(),
        wcag_level: f.wcag_level.clone(),
        dimension: Some(f.dimension.clone()),
        subcategory: Some(f.subcategory.clone()),
        issue_class: Some(f.issue_class.clone()),
        severity: f.severity,
        priority: severity_to_priority(f.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        occurrence_count: f.occurrence_count,
        affected_urls: Vec::new(),
        affected_elements: f.occurrence_count,
        additional_occurrences,
        pattern_clusters,
        location_hints,
        representative_occurrences,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
    }
}

fn build_location_hints(occurrences: &[OccurrenceDetail]) -> Vec<String> {
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

fn build_representative_occurrences(
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
        });

        if items.len() >= 3 {
            break;
        }
    }

    items
}

fn build_pattern_clusters(occurrences: &[OccurrenceDetail]) -> Vec<FindingPatternCluster> {
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

fn normalize_selector_cluster(selector: &str) -> String {
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
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Contrast ratio 1.13:1 for hero headline.".into(),
                selector: Some("main .hero-title".into()),
                fix_suggestion: Some("Increase foreground/background contrast.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Insights</h1>".into()),
                suggested_code: None,
                tags: vec![],
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
            },
            OccurrenceDetail {
                node_id: "node-4".into(),
                message: "Link landmark is outside a region.".into(),
                selector: Some("a.skip-link".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
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
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Second duplicate with richer text".into(),
                selector: Some("MAIN .HERO-TITLE".into()),
                fix_suggestion: Some("Adjust markup.".into()),
                html_snippet: Some("<h1 class=\"hero-title\">Two</h1>".into()),
                suggested_code: Some("<h1 lang=\"de\">Two</h1>".into()),
                tags: vec![],
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Independent selector".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
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
            },
            OccurrenceDetail {
                node_id: "node-2".into(),
                message: "Two".into(),
                selector: Some("main .card-2 .cta".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
            },
            OccurrenceDetail {
                node_id: "node-3".into(),
                message: "Three".into(),
                selector: Some("footer .meta a".into()),
                fix_suggestion: None,
                html_snippet: None,
                suggested_code: None,
                tags: vec![],
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
