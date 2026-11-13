use super::super::helpers::localized_module_name;
use crate::audit::normalized::NormalizedReport;
use crate::output::report_model::{
    AffectedElement, AppendixBlock, AppendixViolation, CapabilitySignal, MethodologyBlock,
};
use fluent_bundle::FluentValue;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

pub(super) fn build_appendix_block_from_normalized(
    locale: &str,
    normalized: &NormalizedReport,
) -> AppendixBlock {
    let violations: Vec<AppendixViolation> = normalized
        .findings
        .iter()
        .map(|f| {
            let affected_elements: Vec<AffectedElement> = f
                .occurrences
                .iter()
                .map(|occ| AffectedElement {
                    selector: occ.selector.clone().unwrap_or_else(|| occ.node_id.clone()),
                    node_id: occ.node_id.clone(),
                })
                .collect();

            // Stored title is canonical English (#406); re-derive the localized
            // title (taxonomy + SEO headings) for non-English reports.
            let rule_name = crate::output::builder::actions::localized_finding_text(locale, f).0;

            AppendixViolation {
                rule: f.wcag_criterion.clone(),
                rule_name,
                severity: f.severity,
                message: f.description.clone(),
                fix_suggestion: f.occurrences.first().and_then(|o| o.fix_suggestion.clone()),
                affected_elements,
            }
        })
        .collect();

    let has_violations = !violations.is_empty();

    AppendixBlock {
        violations,
        score_methodology: crate::i18n::text("methodology-score-methodology", locale == "en"),
        has_violations,
    }
}

pub(super) fn build_methodology(
    i18n: &crate::i18n::I18n,
    normalized: &NormalizedReport,
) -> MethodologyBlock {
    let locale = i18n.locale();
    let en = locale == "en";
    let active_modules = normalized
        .execution
        .module_runs
        .iter()
        .filter(|run| {
            matches!(
                run.status,
                crate::audit::ExecutionStatus::Completed | crate::audit::ExecutionStatus::Partial
            )
        })
        .map(|run| run.module.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let requested_modules = normalized.execution.scope.requested_modules.join(", ");
    let failed_modules = normalized
        .execution
        .module_runs
        .iter()
        .filter(|run| run.status == crate::audit::ExecutionStatus::Failed)
        .map(|run| run.module.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let scope = i18n.t_args(
        "methodology-scope",
        &[
            ("url", FluentValue::from(normalized.url.clone())),
            (
                "level",
                FluentValue::from(normalized.wcag_level.to_string()),
            ),
            ("modules", FluentValue::from(requested_modules)),
        ],
    );
    let executed_rules = normalized
        .rule_outcomes
        .iter()
        .filter(|outcome| !crate::wcag::rule_run_skipped(outcome))
        .count();
    let method = i18n.t_args(
        "methodology-method",
        &[("rules", FluentValue::from(executed_rules as i64))],
    );
    let mut limitations = i18n.t("methodology-limitations");
    if normalized.execution.quality.qualified_results {
        limitations.push(' ');
        limitations.push_str(&i18n.t("methodology-limitations-incomplete"));
    }
    let disclaimer = i18n.t("methodology-disclaimer");

    let preview_value = if normalized.has_screenshots {
        i18n.t("methodology-preview-captured")
    } else {
        i18n.t("methodology-preview-not-captured")
    };

    let weight = |module: String, pct: u32| -> String {
        i18n.t_args(
            "methodology-weight",
            &[
                ("module", FluentValue::from(module)),
                ("pct", FluentValue::from(i64::from(pct))),
            ],
        )
    };
    let total_score_value = {
        let contributing: Vec<_> = normalized
            .module_scores
            .iter()
            .filter(|m| m.contributes_to_overall)
            .collect();
        let total_raw: u32 = contributing.iter().map(|m| m.weight_pct).sum();
        let weights: Vec<String> = contributing
            .iter()
            .map(|m| {
                let pct = (m.weight_pct * 100 + total_raw / 2)
                    .checked_div(total_raw)
                    .unwrap_or(0);
                weight(localized_module_name(&m.name, i18n), pct)
            })
            .collect();
        let weights_label = if weights.is_empty() {
            weight(i18n.t("module-accessibility"), 100)
        } else {
            weights.join(", ")
        };
        let indicator_names: Vec<String> = normalized
            .module_scores
            .iter()
            .filter(|m| !m.contributes_to_overall || m.measurement_type == "heuristic")
            .map(|m| localized_module_name(&m.name, i18n))
            .collect();
        let mut value = i18n.t_args(
            "methodology-combined-score",
            &[
                (
                    "score",
                    FluentValue::from(i64::from(normalized.overall_score)),
                ),
                ("weights", FluentValue::from(weights_label)),
            ],
        );
        if !indicator_names.is_empty() {
            value.push(' ');
            value.push_str(&i18n.t_args(
                "methodology-indicator-modules",
                &[("modules", FluentValue::from(indicator_names.join(", ")))],
            ));
        }
        value
    };

    let runtime_unit = "s";

    let accessibility_score_value = normalized
        .viewport_scores
        .as_ref()
        .map(|scores| {
            i18n.t_args(
                "methodology-accessibility-score-viewports",
                &[
                    ("score", FluentValue::from(i64::from(normalized.score))),
                    (
                        "mobile",
                        FluentValue::from(i64::from(scores.mobile.accessibility)),
                    ),
                    (
                        "desktop",
                        FluentValue::from(i64::from(scores.desktop.accessibility)),
                    ),
                ],
            )
        })
        .unwrap_or_else(|| format!("{} / 100", normalized.score));
    let all_occurrences: usize = normalized
        .findings
        .iter()
        .map(|finding| finding.occurrence_count)
        .sum();
    let counting_value = i18n.t_args(
        "methodology-counting",
        &[
            (
                "groups",
                FluentValue::from(normalized.severity_counts.total as i64),
            ),
            (
                "occurrences",
                FluentValue::from(normalized.occurrence_counts.total as i64),
            ),
            ("rows", FluentValue::from(normalized.findings.len() as i64)),
            ("all", FluentValue::from(all_occurrences as i64)),
        ],
    );

    let mut audit_facts = vec![
        (
            i18n.t("methodology-fact-accessibility-score"),
            accessibility_score_value,
        ),
        (i18n.t("methodology-fact-combined-score"), total_score_value),
        (i18n.t("methodology-fact-counting"), counting_value),
        (
            i18n.t("scope-box-wcag-level"),
            normalized.wcag_level.to_string(),
        ),
        // #653: the mode the scores belong to, and what the page offers.
        (
            i18n.t("batch-cover-frame-display"),
            crate::display::audited_mode_text(normalized.execution.scope.display_mode, en),
        ),
        (
            i18n.t("methodology-fact-page-display-modes"),
            crate::display::offered_modes_text(normalized.execution.display_modes.as_ref(), en),
        ),
        (
            i18n.t("scope-box-checked-nodes"),
            normalized.nodes_analyzed.to_string(),
        ),
        (
            i18n.t("scope-box-runtime"),
            format!(
                "{} {}",
                crate::i18n::number::decimal(normalized.duration_ms as f64 / 1000.0, 1, en),
                runtime_unit
            ),
        ),
        (i18n.t("batch-cover-frame-modules"), active_modules),
        (
            i18n.t("methodology-fact-failed-modules"),
            if failed_modules.is_empty() {
                i18n.t("methodology-failed-modules-none")
            } else {
                failed_modules
            },
        ),
        (
            i18n.t("methodology-fact-audit-quality"),
            // Regression: `{:?}` leaked the raw English enum variant name
            // ("Complete"/"Partial"/"Insufficient") unlocalized into
            // German reports.
            match normalized.execution.quality.status {
                crate::audit::AuditQualityStatus::Complete => {
                    i18n.t("methodology-quality-complete")
                }
                crate::audit::AuditQualityStatus::Partial => i18n.t("methodology-quality-partial"),
                crate::audit::AuditQualityStatus::Insufficient => {
                    i18n.t("methodology-quality-insufficient")
                }
            },
        ),
        (
            i18n.t("scope-box-audit-notes"),
            normalized.audit_flags.len().to_string(),
        ),
        (i18n.t("methodology-fact-preview"), preview_value),
    ];
    // Audit exclusions (#645): never silent — selectors, matches (also
    // zero) and what was dropped.
    if let Some(note) = normalized
        .execution
        .exclusions
        .as_ref()
        .and_then(|report| crate::audit::exclusion::exclusion_note(report, i18n))
    {
        audit_facts.push((i18n.t("exclusion-fact-label"), note));
    }

    MethodologyBlock {
        scope,
        method,
        limitations,
        disclaimer,
        audit_facts,
        confidence_summary: build_confidence_summary(locale, normalized),
        capabilities: build_capability_matrix(locale, normalized),
    }
}

fn build_confidence_summary(locale: &str, normalized: &NormalizedReport) -> Vec<(String, String)> {
    let t = |key: &str| crate::i18n::text(key, locale == "en");
    let base_confidence = if normalized.nodes_analyzed >= 2_000 {
        "methodology-confidence-high"
    } else if normalized.nodes_analyzed >= 500 {
        "methodology-confidence-solid"
    } else {
        "methodology-confidence-limited"
    };
    let caveat_level = if normalized.audit_flags.is_empty() {
        "methodology-signals-none"
    } else if normalized.audit_flags.len() == 1 {
        "methodology-signals-one"
    } else {
        "methodology-signals-multiple"
    };
    let module_coverage = if normalized.module_scores.len() >= 5 {
        "methodology-coverage-broad"
    } else if normalized.module_scores.len() >= 3 {
        "methodology-coverage-extended"
    } else {
        "methodology-coverage-core"
    };

    vec![
        (t("methodology-label-confidence"), t(base_confidence)),
        (t("methodology-label-coverage"), t(module_coverage)),
        (t("methodology-label-signals"), t(caveat_level)),
        (
            t("methodology-label-manual"),
            t("methodology-manual-review"),
        ),
    ]
}

fn build_capability_matrix(locale: &str, normalized: &NormalizedReport) -> Vec<CapabilitySignal> {
    let en = locale == "en";
    let t = |key: &str| crate::i18n::text(key, en);
    let has_module = |name: &str| normalized.module_scores.iter().any(|m| m.name == name);
    let confidence_high = t("methodology-confidence-high");
    let confidence_solid = t("methodology-confidence-solid");
    let confidence_off = t("methodology-confidence-off");

    let mut capabilities = vec![
        CapabilitySignal {
            signal: t("methodology-capability-wcag"),
            source: t("methodology-capability-wcag-source"),
            confidence: if normalized.nodes_analyzed >= 500 {
                confidence_high.clone()
            } else {
                confidence_solid.clone()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: t("methodology-capability-wcag-note"),
        },
        CapabilitySignal {
            signal: t("methodology-capability-performance"),
            source: t("methodology-capability-performance-source"),
            confidence: if has_module("Performance") {
                confidence_high.clone()
            } else {
                confidence_off.clone()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: t("methodology-capability-performance-note"),
        },
        CapabilitySignal {
            signal: t("methodology-capability-seo"),
            source: t("methodology-capability-seo-source"),
            confidence: if has_module("SEO") {
                confidence_solid.clone()
            } else {
                confidence_off.clone()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: t("methodology-capability-seo-note"),
        },
        CapabilitySignal {
            signal: t("methodology-capability-security"),
            source: t("methodology-capability-security-source"),
            confidence: if has_module("Security") {
                confidence_high
            } else {
                confidence_off
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: t("methodology-capability-security-note"),
        },
        CapabilitySignal {
            signal: t("methodology-capability-heuristic"),
            source: t("methodology-capability-heuristic-source"),
            confidence: t("methodology-capability-heuristic-confidence"),
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: t("methodology-capability-heuristic-note"),
        },
    ];

    if !normalized.audit_flags.is_empty() {
        capabilities.push(CapabilitySignal {
            signal: t("methodology-capability-flags"),
            source: t("methodology-capability-flags-source"),
            confidence: t("methodology-capability-flags-confidence"),
            surfaces: vec!["JSON".into(), "PDF".into()],
            note: crate::i18n::count_text(
                "methodology-capability-flags-note",
                normalized.audit_flags.len(),
                en,
            ),
        });
    }

    capabilities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::normalize;
    use crate::audit::AuditQualityStatus;
    use crate::audit::AuditReport;
    use crate::i18n::I18n;
    use crate::model::WcagLevel;
    use crate::wcag::WcagResults;

    fn report_with_quality_status(status: AuditQualityStatus) -> NormalizedReport {
        let raw = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let mut normalized = normalize(&raw).normalized;
        normalized.execution.quality.status = status;
        normalized
    }

    #[test]
    fn audit_quality_status_is_localized_not_debug_formatted() {
        // Regression: `format!("{:?}", ...status)` leaked the raw English
        // enum variant name ("Partial") unlocalized into German reports.
        let normalized = report_with_quality_status(AuditQualityStatus::Partial);
        let i18n_de = I18n::new("de").expect("de locale loads");
        let block_de = build_methodology(&i18n_de, &normalized);
        let (_, value_de) = block_de
            .audit_facts
            .iter()
            .find(|(k, _)| k == "Audit-Qualität")
            .expect("Audit-Qualität fact present");
        assert_eq!(value_de, "Teilweise");

        let i18n_en = I18n::new("en").expect("en locale loads");
        let block_en = build_methodology(&i18n_en, &normalized);
        let (_, value_en) = block_en
            .audit_facts
            .iter()
            .find(|(k, _)| k == "Audit quality")
            .expect("Audit quality fact present");
        assert_eq!(value_en, "Partial");
    }
}
