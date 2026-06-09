//! Finding renderers for PDF reports.

use renderreport::components::advanced::WrongRightBlock;
use renderreport::components::advanced::{KeyValueList, List};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::SummaryBox;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};

pub(super) fn render_key_finding_block(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    include_technical_context: bool,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let sev_label = match (group.severity, en) {
        (crate::wcag::Severity::Critical, true) => "CRITICAL",
        (crate::wcag::Severity::Critical, false) => "KRITISCH",
        (crate::wcag::Severity::High, true) => "HIGH",
        (crate::wcag::Severity::High, false) => "HOCH",
        (crate::wcag::Severity::Medium, true) => "MEDIUM",
        (crate::wcag::Severity::Medium, false) => "MITTEL",
        (crate::wcag::Severity::Low, true) => "LOW",
        (crate::wcag::Severity::Low, false) => "GERING",
    };
    let title = format!("{} — {}", sev_label, group.title);

    let arc = &group.narrative;
    let en = i18n.locale() == "en";
    let recommendation_label = if en { "Recommendation" } else { "Empfehlung" };

    builder = builder
        .add_component(Label::new(&title).bold().with_size("11pt"))
        .add_component(
            Callout::info(customer_perspective_body(group, en))
                .with_title(customer_perspective_title(group, en)),
        );

    if let Some(ref cause) = group.structural_cause {
        if group.is_component_issue {
            builder = builder.add_component(Callout::warning(cause).with_title(if en {
                "Recurring pattern"
            } else {
                "Wiederkehrendes Muster"
            }));
        }
    }

    builder = builder
        .add_component(
            TextBlock::new(first_sentence(&arc.wirkung))
                .with_size("10.5pt")
                .with_line_height("1.4em"),
        )
        .add_component(
            Callout::success(first_sentence(&arc.umsetzung)).with_title(recommendation_label),
        );

    if include_technical_context {
        let tech_context_title = if !group.wcag_criterion.is_empty() {
            format!(
                "{} — WCAG {}",
                i18n.t("finding-tech-context"),
                group.wcag_criterion
            )
        } else {
            i18n.t("finding-tech-context")
        };
        builder = builder.add_component(
            SummaryBox::new(tech_context_title)
                .add_item(i18n.t("finding-tech-rule"), &group.rule_id)
                .add_item("WCAG", &group.wcag_criterion)
                .add_item(
                    i18n.t("finding-tech-instances"),
                    group.occurrence_count.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-affected-elements"),
                    group.affected_elements.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-other-occurrences"),
                    group.additional_occurrences.to_string(),
                )
                .add_item(
                    i18n.t("finding-tech-affected-urls"),
                    group.affected_urls.len().to_string(),
                ),
        );
    }
    builder
}

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

fn customer_perspective_body(group: &FindingGroup, en: bool) -> String {
    let impact = first_sentence(&group.user_impact);
    let base = match customer_perspective_title(group, en) {
        "Operability" => "This finding affects whether visitors can reliably use controls, links, forms or keyboard focus.",
        "Bedienbarkeit" => "Dieser Befund betrifft, ob Besucher Bedienelemente, Links, Formulare oder Tastaturfokus zuverlässig nutzen können.",
        "Orientation and structure" => "This finding affects whether people, assistive technologies, search engines and AI systems can understand the page structure.",
        "Orientierung und Struktur" => "Dieser Befund betrifft, ob Menschen, assistive Technologien, Suchmaschinen und KI-Systeme die Seitenstruktur verstehen.",
        "Perception and readability" => "This finding affects whether content is visible, distinguishable or readable for all visitors.",
        "Wahrnehmbarkeit und Lesbarkeit" => "Dieser Befund betrifft, ob Inhalte für alle Besucher sichtbar, unterscheidbar oder lesbar sind.",
        "Discoverability and AI understanding" => "This finding affects whether the page can be found, summarized and cited reliably by search engines or AI systems.",
        "Auffindbarkeit und KI-Verständnis" => "Dieser Befund betrifft, ob die Seite von Suchmaschinen oder KI-Systemen zuverlässig gefunden, zusammengefasst und zitiert werden kann.",
        "Trust and technical quality" => "This finding affects visible trust signals and the technical reliability of the checked page.",
        "Vertrauen und technische Qualität" => "Dieser Befund betrifft sichtbare Vertrauenssignale und die technische Verlässlichkeit der geprüften Seite.",
        "Loading and mobile experience" => "This finding affects how the page feels while loading, especially on constrained devices or networks.",
        "Lade- und Mobile-Erlebnis" => "Dieser Befund betrifft, wie sich die Seite beim Laden anfühlt, besonders auf eingeschränkten Geräten oder Netzwerken.",
        _ if en => "This finding affects the customer-facing quality of the checked page.",
        _ => "Dieser Befund betrifft die kundennahe Qualität der geprüften Seite.",
    };

    if impact.is_empty() {
        base.to_string()
    } else if en {
        format!("{base} Audit impact: {impact}")
    } else {
        format!("{base} Auswirkung im Audit: {impact}")
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
    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!("{} — {}", group.title, group.rule_id)
    };
    builder = builder.add_component(Label::new(&header).bold().with_size("14pt"));

    let mut meta_kv = KeyValueList::new()
        .add(
            if i18n.locale() == "en" {
                "Customer view"
            } else {
                "Kundensicht"
            },
            customer_perspective_title(group, i18n.locale() == "en"),
        )
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
            .add_item("Node", &occ.node_id)
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
