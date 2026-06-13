//! Finding renderers for PDF reports.

use renderreport::components::advanced::{Divider, KeyValueList, List, WrongRightBlock};
use renderreport::components::text::Label;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};

/// Extract the first sentence from a text (up to first period + space, or full text).
/// Skips common German abbreviations like "z. B.", "d. h.", "u. a.".
pub(super) fn first_sentence(text: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(". ") {
        let pos = search_from + rel;
        // Check for single-letter abbreviation pattern: " X. " (e.g. "z. B.", "d. h.")
        if pos >= 2 {
            let bytes = text.as_bytes();
            let b0 = bytes[pos - 2];
            let b1 = bytes[pos - 1];
            if b0 == b' ' && b1.is_ascii_alphabetic() {
                search_from = pos + 2;
                continue;
            }
        }
        return &text[..pos + 1];
    }
    text
}

#[cfg(test)]
fn customer_perspective_title(group: &FindingGroup, en: bool) -> &'static str {
    let rule = group.rule_id.as_str();
    let dimension = group.dimension.as_deref().unwrap_or_default();
    let subcategory = group.subcategory.as_deref().unwrap_or_default();
    let haystack = format!(
        "{} {} {} {}",
        rule,
        dimension,
        subcategory,
        group.title.to_lowercase()
    );

    if haystack.contains("name_role")
        || haystack.contains("focus")
        || haystack.contains("keyboard")
        || haystack.contains("button")
        || haystack.contains("link_purpose")
        || haystack.contains("target")
        || haystack.contains("form")
    {
        if en {
            "Operability"
        } else {
            "Bedienbarkeit"
        }
    } else if haystack.contains("structure")
        || haystack.contains("heading")
        || haystack.contains("landmark")
        || haystack.contains("region")
        || haystack.contains("bypass")
        || haystack.contains("parsing")
        || haystack.contains("semant")
    {
        if en {
            "Orientation and structure"
        } else {
            "Orientierung und Struktur"
        }
    } else if haystack.contains("contrast")
        || haystack.contains("color")
        || haystack.contains("image")
        || haystack.contains("alt")
        || haystack.contains("visibility")
    {
        if en {
            "Perception and readability"
        } else {
            "Wahrnehmbarkeit und Lesbarkeit"
        }
    } else if dimension == "SEO"
        || haystack.contains("seo")
        || haystack.contains("schema")
        || haystack.contains("meta")
        || haystack.contains("ai")
    {
        if en {
            "Discoverability and AI understanding"
        } else {
            "Auffindbarkeit und KI-Verständnis"
        }
    } else if dimension == "Security" || haystack.contains("security") {
        if en {
            "Trust and technical quality"
        } else {
            "Vertrauen und technische Qualität"
        }
    } else if dimension == "Performance" || dimension == "Mobile" {
        if en {
            "Loading and mobile experience"
        } else {
            "Lade- und Mobile-Erlebnis"
        }
    } else if en {
        "Customer impact"
    } else {
        "Kundenauswirkung"
    }
}

fn report_code_label(value: &str, i18n: &I18n) -> &'static str {
    let en = i18n.locale() == "en";
    match (value, en) {
        ("very_high", true) => "Very high",
        ("very_high", false) => "Sehr hoch",
        ("high", true) => "High",
        ("high", false) => "Hoch",
        ("medium", true) => "Medium",
        ("medium", false) => "Mittel",
        ("low", true) => "Low",
        ("low", false) => "Niedrig",
        ("very_low", true) => "Very low",
        ("very_low", false) => "Sehr niedrig",
        ("immediate", true) => "Immediate",
        ("immediate", false) => "Sofort",
        ("quick_win", true) => "Quick win",
        ("quick_win", false) => "Quick Win",
        ("normal", true) => "Normal",
        ("normal", false) => "Normal",
        ("manual_review_recommended", true) => "Manual review recommended",
        ("manual_review_recommended", false) => "Manuelle Prüfung empfohlen",
        ("automatically_confirmed", true) => "Automatically confirmed",
        ("automatically_confirmed", false) => "Automatisch bestätigt",
        _ => "n/a",
    }
}

pub(super) fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let (severity_prefix, color) = match group.severity {
        crate::wcag::Severity::Critical => (
            if en { "[Critical] " } else { "[Kritisch] " },
            super::design::tokens::DANGER,
        ),
        crate::wcag::Severity::High => (
            if en { "[High] " } else { "[Hoch] " },
            super::design::tokens::DANGER,
        ),
        crate::wcag::Severity::Medium => (
            if en { "[Medium] " } else { "[Mittel] " },
            super::design::tokens::WARN_DEEP,
        ),
        crate::wcag::Severity::Low => (
            if en { "[Low] " } else { "[Gering] " },
            super::design::tokens::NEUTRAL,
        ),
    };

    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{}{} — WCAG {} ({})",
            severity_prefix, group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!(
            "{}{}{} — {}",
            severity_prefix,
            group.title,
            if group.title.is_empty() { "" } else { " — " },
            group.rule_id
        )
    };
    builder = builder.add_component(
        Label::new(&header)
            .bold()
            .with_size("14pt")
            .with_color(color),
    );

    let mut meta_kv = KeyValueList::new()
        .add(
            i18n.t("label-priority"),
            priority_label_i18n(group.priority, i18n),
        )
        .add(
            i18n.t("label-owner"),
            role_label_i18n(group.responsible_role, i18n),
        )
        .add(
            i18n.t("label-effort"),
            effort_label_i18n(group.effort, i18n),
        )
        .add(
            if i18n.locale() == "en" {
                "Implementation priority"
            } else {
                "Umsetzungspriorität"
            },
            report_code_label(&group.remediation_priority, i18n),
        )
        .add(
            if i18n.locale() == "en" {
                "Detection confidence"
            } else {
                "Erkennungssicherheit"
            },
            report_code_label(&group.confidence, i18n),
        )
        .add(
            if i18n.locale() == "en" {
                "False-positive risk"
            } else {
                "Falschpositiv-Risiko"
            },
            report_code_label(&group.false_positive_risk, i18n),
        )
        .add(
            if i18n.locale() == "en" {
                "BFSG/EAA relevance"
            } else {
                "BFSG-/EAA-Relevanz"
            },
            report_code_label(&group.bfsg_relevance, i18n),
        )
        .add(
            i18n.t("finding-elements"),
            group.affected_elements.to_string(),
        )
        .add(
            i18n.t("finding-occurrences"),
            group.occurrence_count.to_string(),
        );
    if let Some(url) = group
        .help_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        meta_kv = meta_kv.add(i18n.t("finding-reference"), url);
    }
    builder = builder.add_component(meta_kv);

    if !group.expected_impact.is_empty() || !group.complexity_reason.is_empty() {
        let mut assessment = KeyValueList::new().with_title(if i18n.locale() == "en" {
            "Action assessment"
        } else {
            "Maßnahmenbewertung"
        });
        if !group.expected_impact.is_empty() {
            assessment = assessment.add(
                if i18n.locale() == "en" {
                    "Expected effect"
                } else {
                    "Erwartete Wirkung"
                },
                &group.expected_impact,
            );
        }
        if !group.complexity_reason.is_empty() {
            assessment = assessment.add(
                if i18n.locale() == "en" {
                    "Complexity reason"
                } else {
                    "Komplexitätsgrund"
                },
                &group.complexity_reason,
            );
        }
        if group.verification == "manual_review_recommended" {
            assessment = assessment.add(
                if i18n.locale() == "en" {
                    "Verification"
                } else {
                    "Prüfung"
                },
                if i18n.locale() == "en" {
                    "Manual review recommended"
                } else {
                    "Manuelle Prüfung empfohlen"
                },
            );
        }
        builder = builder.add_component(assessment);
    }

    // AffectedElements: element-type summary + deduplicated selector list
    if !group.representative_occurrences.is_empty() {
        // Count occurrences per element type
        let mut type_counts: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        for occ in &group.representative_occurrences {
            let tag = extract_element_type(&occ.selector);
            if !tag.is_empty() {
                *type_counts.entry(tag).or_insert(0) += 1;
            }
        }
        if !type_counts.is_empty() {
            let summary = type_counts
                .iter()
                .map(|(tag, count)| format!("{}× {}", count, tag))
                .collect::<Vec<_>>()
                .join("  ·  ");
            builder = builder
                .add_component(KeyValueList::new().add(i18n.t("finding-element-types"), summary));
        }
    }

    if !group.recommendation.is_empty() {
        builder = builder.add_component(
            Callout::success(&group.recommendation).with_title(i18n.t("finding-recommendation")),
        );
    }

    for example in &group.examples {
        builder = builder.add_component(
            WrongRightBlock::new(&example.bad, &example.good)
                .code()
                .with_labels(i18n.t("finding-wrong"), i18n.t("finding-right")),
        );
        if let Some(ref dec) = example.decorative {
            builder =
                builder.add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    }

    if !group.representative_occurrences.is_empty() {
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(i18n.t("finding-location"))
                .with_width("26%"),
            renderreport::components::TableColumn::new(i18n.t("finding-note")).with_width("74%"),
        ])
        .with_title(i18n.t("finding-representative-occurrences"));

        for occ in &group.representative_occurrences {
            table = table.add_row(vec![
                truncate_url(&occ.selector, 48),
                first_sentence(&occ.message).to_string(),
            ]);
        }
        builder = builder.add_component(table);

        let en = i18n.locale() == "en";
        let hidden = group.representative_occurrences.len().saturating_sub(3);
        for occ in group.representative_occurrences.iter().take(3) {
            let mut snapshot = SummaryBox::new(format!(
                "{}: {}",
                i18n.t("finding-occurrence"),
                truncate_url(&occ.selector, 60)
            ))
            .add_item("Node", truncate_url(&occ.node_id, 75))
            .add_item(i18n.t("finding-note"), first_sentence(&occ.message));

            if let Some(html) = occ
                .html_snippet
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                snapshot = snapshot.add_item("HTML", truncate_url(html, 110));
            }

            builder = builder.add_component(snapshot);

            if let Some(code) = occ
                .suggested_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                builder = builder.add_component(
                    Callout::info(truncate_url(code, 180))
                        .with_title(i18n.t("finding-suggested-fix")),
                );
            }
        }
        if hidden > 0 {
            let msg = if en {
                format!("{hidden} more occurrence(s) documented in the technical appendix.")
            } else {
                format!("{hidden} weitere(s) Vorkommen im technischen Anhang dokumentiert.")
            };
            builder = builder.add_component(Callout::info(&msg));
        }
    }

    if !group.pattern_clusters.is_empty() {
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(i18n.t("finding-pattern")).with_width("70%"),
            renderreport::components::TableColumn::new(i18n.t("finding-occurrences"))
                .with_width("30%"),
        ])
        .with_title(i18n.t("finding-frequent-patterns"));

        for cluster in &group.pattern_clusters {
            table = table.add_row(vec![
                truncate_url(&cluster.label, 72),
                cluster.occurrences.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if let Some(ref cause) = group.structural_cause {
        let label = if group.is_component_issue {
            "Root Cause"
        } else if i18n.locale() == "en" {
            "Structural cause"
        } else {
            "Strukturelle Ursache"
        };
        let callout = if group.is_component_issue {
            Callout::warning(cause).with_title(label)
        } else {
            Callout::info(cause).with_title(label)
        };
        builder = builder.add_component(callout);
    }

    builder = builder.add_component(Divider {
        style: "solid".to_string(),
        thickness: "0pt".to_string(),
        color: Some("#ffffff".to_string()),
        spacing_above: "15pt".to_string(),
        spacing_below: "0pt".to_string(),
    });

    builder
}

/// Extract the HTML element type from a CSS selector path.
/// Examples: "div.main > img" → "img", "a#skip-link" → "a", "button" → "button"
fn extract_element_type(selector: &str) -> &str {
    let last_segment = selector.split('>').next_back().unwrap_or(selector).trim();
    let last_token = last_segment
        .split_whitespace()
        .last()
        .unwrap_or(last_segment);
    let end = last_token
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .unwrap_or(last_token.len());
    &last_token[..end]
}

#[cfg(test)]
mod tests {
    use super::customer_perspective_title;
    use crate::output::report_model::{
        CriticalityTier, Effort, FindingGroup, FindingPatternCluster, NarrativeArc, Priority, Role,
    };
    use crate::wcag::Severity;

    fn group(rule_id: &str, title: &str, dimension: &str) -> FindingGroup {
        FindingGroup {
            title: title.to_string(),
            rule_id: rule_id.to_string(),
            wcag_criterion: String::new(),
            wcag_level: String::new(),
            help_url: None,
            dimension: Some(dimension.to_string()),
            subcategory: None,
            issue_class: None,
            severity: Severity::High,
            priority: Priority::High,
            customer_description: String::new(),
            user_impact: "Nutzer können die betroffene Funktion nicht zuverlässig verwenden."
                .to_string(),
            business_impact: String::new(),
            typical_cause: String::new(),
            recommendation: String::new(),
            technical_note: String::new(),
            confidence: String::new(),
            false_positive_risk: String::new(),
            verification: String::new(),
            complexity: String::new(),
            complexity_reason: String::new(),
            expected_impact: String::new(),
            bfsg_relevance: String::new(),
            remediation_priority: String::new(),
            occurrence_count: 1,
            affected_urls: vec![],
            affected_elements: 1,
            additional_occurrences: 0,
            pattern_clusters: Vec::<FindingPatternCluster>::new(),
            location_hints: vec![],
            representative_occurrences: vec![],
            responsible_role: Role::Development,
            effort: Effort::Medium,
            execution_priority: crate::output::report_model::ExecutionPriority::Important,
            examples: vec![],
            structural_cause: None,
            is_component_issue: false,
            criticality_tier: CriticalityTier::Mandatory,
            narrative: NarrativeArc {
                diagnose: String::new(),
                ursache: String::new(),
                wirkung: String::new(),
                umsetzung: String::new(),
            },
        }
    }

    #[test]
    fn finding_customer_perspective_maps_common_audit_topics() {
        assert_eq!(
            customer_perspective_title(
                &group(
                    "a11y.name_role.missing",
                    "Fehlende Name/Rolle",
                    "Accessibility"
                ),
                false,
            ),
            "Bedienbarkeit"
        );
        assert_eq!(
            customer_perspective_title(
                &group(
                    "a11y.structure.missing",
                    "Fehlende semantische Struktur",
                    "Accessibility"
                ),
                false,
            ),
            "Orientierung und Struktur"
        );
        assert_eq!(
            customer_perspective_title(
                &group(
                    "a11y.contrast.weak",
                    "Unzureichender Farbkontrast",
                    "Accessibility"
                ),
                false,
            ),
            "Wahrnehmbarkeit und Lesbarkeit"
        );
        assert_eq!(
            customer_perspective_title(
                &group("seo.headings.empty_heading", "Leere Überschrift", "SEO"),
                false,
            ),
            "Orientierung und Struktur"
        );
    }
}
