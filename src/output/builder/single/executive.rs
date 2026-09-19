use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    ExecutiveNarrativeBlock, FindingGroup, PositiveSignal, SeverityBlock,
};

/// Translates pattern names into localized titles; the message stays as the
/// description (already human-readable from the detector).
pub(super) fn build_positive_signals(
    locale: &str,
    normalized: &AuditContext<'_>,
) -> Vec<PositiveSignal> {
    let en = locale == "en";
    let mut items: Vec<PositiveSignal> = normalized
        .raw_patterns
        .map(|p| {
            p.recognized
                .iter()
                .map(|r| {
                    let title = match (en, r.pattern.as_str()) {
                        (true, "MainNavigation") => "Semantic main navigation",
                        (false, "MainNavigation") => "Semantische Hauptnavigation",
                        (true, "DisclosureMenu") => "Disclosure menu",
                        (false, "DisclosureMenu") => "Disclosure-Menü",
                        (true, "ModalDialog") => "Modal dialog",
                        (false, "ModalDialog") => "Modaler Dialog",
                        (true, "TabList") => "Tab list",
                        (false, "TabList") => "Tab-Liste",
                        (true, "SkipLink") => "Skip link",
                        (false, "SkipLink") => "Skip-Link",
                        (true, "Accordion") => "Accordion",
                        (false, "Accordion") => "Accordion",
                        (true, "EasyLanguage") => "Easy-language version",
                        (false, "EasyLanguage") => "Leichte-Sprache-Version",
                        (_, other) => other,
                    };
                    // #406: every other pattern's `message` is a raw English
                    // passthrough (pre-existing, out of scope here) — only
                    // EasyLanguage's description is re-derived per locale,
                    // since it's the one new message this task adds.
                    let description = if r.pattern == "EasyLanguage" {
                        crate::patterns::easy_language::message_text(en).to_string()
                    } else {
                        r.message.clone()
                    };
                    PositiveSignal {
                        title: title.to_string(),
                        description,
                        strong: matches!(r.confidence, crate::patterns::PatternConfidence::Strong),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    for pos in &normalized.raw_wcag.positives {
        let title = if en { "WCAG signal" } else { "WCAG-Signal" };
        items.push(PositiveSignal {
            title: title.to_string(),
            description: pos.message.clone(),
            strong: false,
        });
    }

    items
}

pub(super) fn build_executive_narrative(
    i18n: &I18n,
    normalized: &NormalizedReport,
    audit_summary: &crate::audit::summary::AuditSummary,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
) -> ExecutiveNarrativeBlock {
    let dominant_share = audit_summary
        .dominant_issue
        .as_ref()
        .map(|d| d.share_pct.round() as usize);
    let key_points = build_single_key_points_text(
        i18n.locale(),
        severity,
        top_findings,
        normalized,
        dominant_share,
    );

    ExecutiveNarrativeBlock {
        cover_eyebrow: i18n.t("narrative-cover-eyebrow"),
        cover_kicker: i18n.t("narrative-cover-kicker"),
        key_points,
        next_steps_callout_body: i18n.t("narrative-next-steps-callout-body"),
    }
}

fn build_single_key_points_text(
    locale: &str,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    normalized: &NormalizedReport,
    dominant_share: Option<usize>,
) -> Vec<String> {
    let en = locale == "en";
    let mut points = Vec::with_capacity(3);
    if severity.critical > 0 {
        if en {
            points.push("Multiple critical WCAG barriers block assistive technology users — screen readers and keyboard navigation are affected.".to_string());
        } else {
            points.push("Mehrere kritische WCAG-Barrieren blockieren Nutzer von Hilfstechnologien — Screenreader und Tastaturnavigation sind betroffen.".to_string());
        }
    } else if severity.high > 0 {
        if en {
            points.push("Significant accessibility gaps reduce usability for users with disabilities — no complete access barriers, but relevant friction points.".to_string());
        } else {
            points.push("Deutliche Barrierefreiheitslücken erschweren die Nutzung für Menschen mit Behinderungen — keine vollständigen Zugangssperren, aber relevante Reibungspunkte.".to_string());
        }
    }

    if let Some(top) = top_findings.first() {
        let total_ch = (severity.critical + severity.high) as usize;
        // Prefer the canonical dominant-issue share so this line and the
        // leverage text never disagree on the same percentage (#360).
        let share = dominant_share.unwrap_or_else(|| {
            (top.occurrence_count * 100)
                .checked_div(total_ch)
                .unwrap_or(0)
        });
        if share >= 30 {
            if en {
                points.push(format!(
                    "Main issue: {} ({}% of all critical/high findings)",
                    top.title,
                    share.min(99)
                ));
            } else {
                points.push(format!(
                    "Hauptproblem: {} ({}% aller kritischen/hohen Befunde)",
                    top.title,
                    share.min(99)
                ));
            }
        } else if en {
            points.push(format!("Most frequent issue: {}", top.title));
        } else {
            points.push(format!("Häufigstes Problem: {}", top.title));
        }
    }

    let score_driver_note = build_score_driver_note(locale, normalized);
    if !score_driver_note.is_empty() {
        points.push(score_driver_note);
    }

    if normalized.risk.legal_flags > 0 {
        if en {
            points.push(
                "High or critical WCAG Level A findings detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "Hohe oder kritische WCAG-Level-A-Befunde automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if severity.high > 0 {
        if en {
            points.push("No Level A violations, but structural weaknesses".to_string());
        } else {
            points.push(
                "Keine Level-A-Verstöße, aber strukturelle Optimierungspotenziale".to_string(),
            );
        }
    } else if !normalized.audit_flags.is_empty() {
        if en {
            points.push(
                "Audit notes present — individual signals should be verified manually.".to_string(),
            );
        } else {
            points.push(
                "Audit-Hinweise vorhanden — einzelne Signale sollten fachlich gegengeprüft werden."
                    .to_string(),
            );
        }
    } else if en {
        points.push(
            "No automatically detectable critical barriers — manual review recommended."
                .to_string(),
        );
    } else {
        points.push(
            "Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen."
                .to_string(),
        );
    }

    let (automated, total) = crate::wcag::coverage::coverage_stats();
    if en {
        points.push(format!(
            "WCAG scope: {automated} of about {total} WCAG 2.1 AA criteria are checked automatically; manual review remains required for context-dependent criteria."
        ));
    } else {
        points.push(format!(
            "WCAG-Prüfumfang: {automated} von ca. {total} WCAG-2.1-AA-Kriterien werden automatisch geprüft; kontextabhängige Kriterien bleiben manuell zu prüfen."
        ));
    }

    if let Some(flag) = normalized
        .audit_flags
        .iter()
        .find(|f| f.kind == "viewport_gap")
    {
        points.push(flag.message.clone());
    }

    points
}

fn build_score_driver_note(locale: &str, normalized: &NormalizedReport) -> String {
    let en = locale == "en";
    let mut drivers: std::collections::BTreeMap<&'static str, u32> =
        std::collections::BTreeMap::new();
    for finding in &normalized.findings {
        let severity_weight = match finding.severity {
            crate::wcag::Severity::Critical => 20,
            crate::wcag::Severity::High => 14,
            crate::wcag::Severity::Medium => 8,
            crate::wcag::Severity::Low => 4,
        };
        let Some(area) = score_area_for_key_point(finding) else {
            continue;
        };
        *drivers.entry(area).or_default() += severity_weight * finding.occurrence_count as u32;
    }
    let mut drivers: Vec<_> = drivers.into_iter().filter(|(_, loss)| *loss > 0).collect();
    drivers.sort_by_key(|(_, loss)| std::cmp::Reverse(*loss));
    let labels: Vec<_> = drivers.into_iter().take(3).map(|(area, _)| area).collect();
    if labels.is_empty() {
        if en {
            "Score matrix: no negative driver detected in the weighted accessibility topics."
                .to_string()
        } else {
            "Score-Matrix: kein negativer Haupttreiber in den gewichteten Accessibility-Themen erkannt."
                .to_string()
        }
    } else if en {
        format!(
            "Score matrix: strongest negative drivers are {}; weighting includes semantics, forms, keyboard, focus, images, ARIA, headings and landmarks.",
            labels.join(", ")
        )
    } else {
        format!(
            "Score-Matrix: stärkste negative Treiber sind {}; gewichtet werden Semantik, Formulare, Tastatur, Fokus, Bilder, ARIA, Überschriften und Landmarks.",
            labels.join(", ")
        )
    }
}

/// Which score-breakdown area a finding belongs to, for the executive
/// summary's driver note.
///
/// Shares `score_area_for_finding`'s taxonomy mapping. This used to be a
/// near-verbatim copy of the same keyword matching — its own comment recorded
/// that it "had drifted into an unfixed, duplicated copy" — so every
/// misclassification fixed in the JSON breakdown had to be fixed here a second
/// time, or silently was not (plan 38).
fn score_area_for_key_point(
    finding: &crate::audit::normalized::NormalizedFinding,
) -> Option<&'static str> {
    crate::output::json::helpers::score_area_for_finding(finding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, NormalizedFinding, ReportVisibilityData, ScoreEffect,
        ScoreImpactData,
    };
    use crate::taxonomy::Severity;

    fn make_finding(rule_id: &str) -> NormalizedFinding {
        NormalizedFinding {
            category: "wcag".into(),
            rule_id: rule_id.into(),
            wcag_criterion: "1.3.4".into(),
            axe_id: None,
            wcag_level: "AA".into(),
            dimension: "Accessibility".into(),
            subcategory: "Structure & Semantics".into(),
            issue_class: "Weak".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::StructureSemantics,
            issue_class_kind: crate::taxonomy::IssueClass::Weak,
            severity: Severity::High,
            user_impact: String::new(),
            technical_impact: String::new(),
            score_impact: ScoreImpactData {
                base_penalty: 3.0,
                max_penalty: 6.0,
                scaling: "Fixed".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: rule_id.into(),
            title: String::new(),
            description:
                "Die Seite erzwingt eine bestimmte Bildschirmausrichtung (Hoch- oder Querformat)."
                    .into(),
            help_url: None,
            occurrence_count: 1,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count: 1,
                score_effect: ScoreEffect::Low,
                wcag_level: "AA".into(),
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences: vec![],
        }
    }

    /// Plan 38: the executive driver note and the JSON breakdown must agree.
    /// They used to be two copies of the same keyword matching, so a
    /// misclassification fixed in one could silently persist in the other.
    #[test]
    fn score_area_for_key_point_matches_the_json_breakdown() {
        for rule_id in [
            "a11y.orientation.restricted",
            "a11y.form_labels.missing",
            "a11y.target_size_minimum.small",
            "a11y.landmark_region.missing",
            "a11y.contrast.weak",
        ] {
            let finding = make_finding(rule_id);
            assert_eq!(
                score_area_for_key_point(&finding),
                crate::output::json::helpers::score_area_for_finding(&finding),
                "{rule_id}",
            );
        }
    }

    #[test]
    fn score_area_for_key_point_still_classifies_real_forms_findings() {
        let finding = make_finding("a11y.form_labels.missing");
        assert_eq!(score_area_for_key_point(&finding), Some("Forms"));
    }

    /// "Querformat" in the German rule text used to match `form`.
    #[test]
    fn score_area_for_key_point_does_not_classify_orientation_lock_as_forms() {
        let finding = make_finding("a11y.orientation.restricted");
        assert_ne!(score_area_for_key_point(&finding), Some("Forms"));
    }
}
