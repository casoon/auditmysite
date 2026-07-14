//! Finding renderers for PDF reports.

use renderreport::components::advanced::{Divider, KeyValueList, List, WrongRightBlock};
use renderreport::components::text::Label;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;
use crate::wcag::ViolationEvidence;

use super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};

/// Extract the first sentence from a text (up to first period + space, or full text).
/// Skips common German abbreviations like "z. B.", "d. h.", "u. a.".
/// Compact an HTML/selector evidence string for table cells: collapse
/// whitespace and replace `data:` URI bodies (huge unbreakable tokens that blow
/// up row heights) with `data:…`, then length-truncate. Keeps the meaningful
/// markup (tag, classes) while dropping the inline-asset noise.
pub(super) fn compact_html(s: &str, max: usize) -> String {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::with_capacity(collapsed.len());
    let mut rest = collapsed.as_str();
    while let Some(pos) = rest.find("data:") {
        result.push_str(&rest[..pos]);
        result.push_str("data:…");
        rest = &rest[pos + "data:".len()..];
        // Skip the URI body up to the closing attribute quote.
        if let Some(q) = rest.find(['"', '\'']) {
            rest = &rest[q..];
        } else {
            rest = "";
        }
    }
    result.push_str(rest);
    truncate_url(&result, max)
}

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

/// Find a specific `computed`/`ax_tree` evidence value by field name.
fn evidence_value<'a>(
    evidence: &'a [ViolationEvidence],
    source: &str,
    field: &str,
) -> Option<&'a str> {
    evidence
        .iter()
        .find(|e| e.source == source && e.field.as_deref() == Some(field))
        .and_then(|e| e.value.as_deref())
        .filter(|v| !v.is_empty())
}

/// "Contrast 2.70:1 (required 4.5:1)" — combines the contrast rule's
/// `computed` evidence into one localized measured-value line. `None` when
/// the occurrence carries no computed contrast evidence (i.e. every rule
/// other than 1.4.3).
fn contrast_measured_text(evidence: &[ViolationEvidence], en: bool) -> Option<String> {
    let ratio = evidence_value(evidence, "computed", "contrast_ratio")?;
    let required = evidence_value(evidence, "computed", "required_ratio");
    Some(match (required, en) {
        (Some(req), true) => format!("Contrast {} (required {})", ratio, req),
        (Some(req), false) => format!("Kontrast {} (erforderlich {})", ratio, req),
        (None, true) => format!("Contrast {}", ratio),
        (None, false) => format!("Kontrast {}", ratio),
    })
}

pub(super) fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    report_ts: i64,
    evidence_seq: &mut usize,
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
    } else if group.title.is_empty() {
        // Raw internal rule ids (e.g. "seo.headings.multiple_h1") are never
        // customer-facing prose — only fall back to one when there's truly no
        // title to show at all, and even then, don't also glue it onto a
        // perfectly good title as an unexplained suffix (#QA-039 report review).
        format!("{}{}", severity_prefix, group.rule_id)
    } else {
        format!("{}{}", severity_prefix, group.title)
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

    // `expected_impact`/`complexity_reason` are re-derived here from the
    // stable `..._kind` in the run language rather than printed from the
    // `FindingGroup` string fields verbatim — those stay canonical English
    // (mirroring the JSON), so printing them directly leaked raw English
    // prose into otherwise fully German reports (#406).
    let expected_impact =
        crate::audit::normalized::expected_impact_text(&group.expected_impact_kind, en);
    let complexity_reason = crate::audit::normalized::complexity_text(group.complexity_kind, en);
    if !expected_impact.is_empty() || !complexity_reason.is_empty() {
        let mut assessment = KeyValueList::new().with_title(if i18n.locale() == "en" {
            "Action assessment"
        } else {
            "Maßnahmenbewertung"
        });
        if !expected_impact.is_empty() {
            assessment = assessment.add(
                if i18n.locale() == "en" {
                    "Expected effect"
                } else {
                    "Erwartete Wirkung"
                },
                &expected_impact,
            );
        }
        if !complexity_reason.is_empty() {
            assessment = assessment.add(
                if i18n.locale() == "en" {
                    "Complexity reason"
                } else {
                    "Komplexitätsgrund"
                },
                &complexity_reason,
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

    // Findings with no real DOM evidence (e.g. SEO heading findings, whose
    // `OccurrenceDetail` has no selector at all — see `aggregate_seo_findings`)
    // fall back to using the internal issue-type token as both `node_id` and
    // `selector`, which is how `build_location_hints`'s fallback resolves it.
    // Applying DOM-evidence components (element-type extraction, a "location"
    // table/snapshot, selector-pattern clustering) to that token produces
    // nonsense like "Element-Typen: 1× multiple" or "Fundstelle: multiple_h1" —
    // skip all of them when no occurrence actually has real location data.
    let has_real_location = group
        .representative_occurrences
        .iter()
        .any(|occ| occ.selector != occ.node_id);

    // AffectedElements: element-type summary + deduplicated selector list
    if has_real_location {
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

    if has_real_location {
        let mut table = renderreport::components::AuditTable::new(vec![
            renderreport::components::TableColumn::new(i18n.t("finding-location"))
                .with_width("26%"),
            renderreport::components::TableColumn::new(i18n.t("finding-note")).with_width("74%"),
        ])
        .with_title(i18n.t("finding-representative-occurrences"));

        for occ in &group.representative_occurrences {
            table = table.add_row(vec![
                compact_html(&occ.selector, 48),
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
                compact_html(&occ.selector, 60)
            ))
            .add_item("Node", truncate_url(&occ.node_id, 75))
            .add_item(i18n.t("finding-note"), first_sentence(&occ.message));

            if let Some(html) = occ
                .html_snippet
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                snapshot = snapshot.add_item("HTML", compact_html(html, 110));
            }

            if let Some(path) = evidence_value(&occ.evidence, "ax_tree", "dom_path") {
                snapshot = snapshot.add_item(
                    if en { "DOM path" } else { "DOM-Pfad" },
                    compact_html(path, 90),
                );
            }

            if let Some(measured) = contrast_measured_text(&occ.evidence, en) {
                snapshot =
                    snapshot.add_item(if en { "Measured value" } else { "Messwert" }, measured);
            }

            builder = builder.add_component(snapshot);

            // Evidence-grade findings: a cropped, highlighted screenshot of
            // the element behind this occurrence, if one was captured during
            // the audit (single-URL PDF only, capped at MAX_ELEMENT_CROPS
            // report-wide — see `PipelineConfig.capture_element_evidence`).
            if let Some(bytes) = &occ.evidence_screenshot {
                let temp_path = std::env::temp_dir()
                    .join(format!("ams-evidence-{}-{}.png", report_ts, evidence_seq));
                if std::fs::write(&temp_path, bytes).is_ok() {
                    let asset_name = format!("/auditmysite-evidence-{}.png", evidence_seq);
                    builder = builder.asset(asset_name.clone(), temp_path);
                    let caption = match (occ.evidence_viewport, en) {
                        (Some("mobile"), true) => "Element on page (mobile viewport)",
                        (Some("mobile"), false) => "Element auf der Seite (Mobile-Ansicht)",
                        (_, true) => "Element on page (desktop viewport)",
                        (_, false) => "Element auf der Seite (Desktop-Ansicht)",
                    };
                    builder = builder.add_component(
                        Image::new(asset_name)
                            .with_width("55%")
                            .with_caption(caption),
                    );
                }
                *evidence_seq += 1;
            }

            if let Some(code) = occ
                .suggested_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                builder = builder.add_component(
                    Callout::info(compact_html(code, 180))
                        .with_title(i18n.t("finding-suggested-fix")),
                );
            }
        }
        if hidden > 0 {
            let msg = if en {
                format!(
                    "{hidden} more {} documented in the technical appendix.",
                    if hidden == 1 {
                        "occurrence"
                    } else {
                        "occurrences"
                    }
                )
            } else {
                format!(
                    "{hidden} {} im technischen Anhang dokumentiert.",
                    if hidden == 1 {
                        "weiteres Vorkommen"
                    } else {
                        "weitere Vorkommen"
                    }
                )
            };
            builder = builder.add_component(Callout::info(&msg));
        }
    }

    if has_real_location && !group.pattern_clusters.is_empty() {
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
        builder = builder.add_component(Callout::info(i18n.t("finding-frequent-patterns-note")));
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
            complexity_kind: crate::audit::normalized::ComplexityKind::LowScope,
            expected_impact: String::new(),
            expected_impact_kind: crate::audit::normalized::ExpectedImpactKind::Other {
                occurrence_count: 1,
                score_effect: crate::audit::normalized::ScoreEffect::Low,
            },
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
