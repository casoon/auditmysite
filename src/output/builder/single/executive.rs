use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::localized::is_english;
use crate::output::report_model::{
    ExecutiveNarrativeBlock, FindingGroup, PositiveSignal, SeverityBlock,
};

/// Translates pattern names into localized titles; the message stays as the
/// description (already human-readable from the detector).
pub(super) fn build_positive_signals(
    i18n: &I18n,
    normalized: &AuditContext<'_>,
) -> Vec<PositiveSignal> {
    let en = is_english(i18n);
    let mut items: Vec<PositiveSignal> = normalized
        .raw_patterns
        .map(|p| {
            p.recognized
                .iter()
                .map(|r| {
                    let title = match r.pattern.as_str() {
                        "MainNavigation" => i18n.t("report-builder-pattern-main-navigation"),
                        "DisclosureMenu" => i18n.t("report-builder-pattern-disclosure-menu"),
                        "ModalDialog" => i18n.t("report-builder-pattern-modal-dialog"),
                        "TabList" => i18n.t("report-builder-pattern-tab-list"),
                        "SkipLink" => i18n.t("report-builder-pattern-skip-link"),
                        "Accordion" => i18n.t("report-builder-pattern-accordion"),
                        "EasyLanguage" => i18n.t("report-builder-pattern-easy-language"),
                        "Form" => i18n.t("report-builder-pattern-form"),
                        "AddToCart" => i18n.t("report-builder-pattern-add-to-cart"),
                        "QuantityStepper" => i18n.t("report-builder-pattern-quantity-stepper"),
                        other => other.to_string(),
                    };
                    // #406: der Text wird in der Lauf-Sprache neu abgeleitet;
                    // ohne Rohwerte (Muster von außen gebaut) bleibt `message`.
                    let description = match &r.detail {
                        Some(detail) => crate::patterns::pattern_message_text(detail, en),
                        None => r.message.clone(),
                    };
                    PositiveSignal {
                        title,
                        description,
                        strong: matches!(r.confidence, crate::patterns::PatternConfidence::Strong),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    for pos in &normalized.raw_wcag.positives {
        items.push(PositiveSignal {
            title: i18n.t("report-builder-wcag-signal"),
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
    let key_points =
        build_single_key_points_text(i18n, severity, top_findings, normalized, dominant_share);

    ExecutiveNarrativeBlock {
        cover_eyebrow: i18n.t("narrative-cover-eyebrow"),
        cover_kicker: i18n.t("narrative-cover-kicker"),
        key_points,
        next_steps_callout_body: i18n.t("narrative-next-steps-callout-body"),
    }
}

fn build_single_key_points_text(
    i18n: &I18n,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    normalized: &NormalizedReport,
    dominant_share: Option<usize>,
) -> Vec<String> {
    let mut points = Vec::with_capacity(3);
    if severity.critical > 0 {
        points.push(i18n.t("report-builder-key-critical"));
    } else if severity.high > 0 {
        points.push(i18n.t("report-builder-key-high"));
    }

    if let Some(top) = top_findings.first() {
        let total_ch = (severity.critical + severity.high) as usize;
        // Prefer the canonical dominant-issue share so this line and the
        // leverage text never disagree on the same percentage (#360).
        let share = dominant_share.unwrap_or_else(|| {
            (top.finding.occurrence_count * 100)
                .checked_div(total_ch)
                .unwrap_or(0)
        });
        if share >= 30 {
            points.push(i18n.t_args(
                "report-builder-key-main-issue",
                &[
                    ("title", top.title.clone()),
                    ("share", share.min(99).to_string()),
                ],
            ));
        } else {
            points.push(i18n.t_args(
                "report-builder-key-most-frequent",
                &[("title", top.title.clone())],
            ));
        }
    }

    let score_driver_note = build_score_driver_note(i18n, normalized);
    if !score_driver_note.is_empty() {
        points.push(score_driver_note);
    }

    if normalized.risk.legal_flags > 0 {
        points.push(i18n.t("report-builder-key-legal"));
    } else if severity.high > 0 {
        points.push(i18n.t("report-builder-key-no-level-violations"));
    } else if !normalized.audit_flags.is_empty() {
        points.push(i18n.t("report-builder-key-audit-notes"));
    } else {
        points.push(i18n.t("report-builder-key-no-barriers"));
    }

    let (automated, total) = crate::wcag::coverage::coverage_stats();
    points.push(i18n.t_args(
        "report-builder-key-scope",
        &[
            ("automated", automated.to_string()),
            ("total", total.to_string()),
        ],
    ));

    if let Some(flag) = normalized
        .audit_flags
        .iter()
        .find(|f| f.kind == "viewport_gap")
    {
        points.push(flag.message.clone());
    }

    points
}

fn build_score_driver_note(i18n: &I18n, normalized: &NormalizedReport) -> String {
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
        i18n.t("report-builder-driver-none")
    } else {
        i18n.t_args(
            "report-builder-driver-areas",
            &[("areas", labels.join(", "))],
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
/// time, or silently was not.
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
            known: false,
            contract: Default::default(),
            origin: Default::default(),
            providers: Default::default(),
            id: String::new(),
            patterns: Vec::new(),
            pattern_count: 0,
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
            understanding_url: None,
            occurrence_count: 1,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            assurance: Default::default(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count: 1,
                score_effect: ScoreEffect::Low,
                wcag_level: "AA".into(),
                score_gain: None,
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences: vec![],
        }
    }

    /// The executive driver note and the JSON breakdown must agree.
    /// They used to be two copies of the same keyword matching, so a
    /// misclassification fixed in one could silently persist in the other.
    #[test]
    fn score_area_for_key_point_matches_the_json_breakdown() {
        for rule_id in [
            "orientation/content-hidden",
            "forms/label-missing",
            "targets/size",
            "landmarks/content-outside",
            "contrast/text-insufficient",
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
        let finding = make_finding("forms/label-missing");
        assert_eq!(score_area_for_key_point(&finding), Some("Forms"));
    }

    /// "Querformat" in the German rule text used to match `form`.
    #[test]
    fn score_area_for_key_point_does_not_classify_orientation_lock_as_forms() {
        let finding = make_finding("orientation/content-hidden");
        assert_ne!(score_area_for_key_point(&finding), Some("Forms"));
    }
}
