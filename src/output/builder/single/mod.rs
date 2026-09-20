//! Single-report ViewModel builder.

mod actions_block;
mod diagnosis;
mod executive;
mod findings;
mod methodology;
mod module_details;
mod modules_block;
mod positive;
mod serp;
use self::actions_block::build_actions_block;
use self::diagnosis::{
    build_criticality_groups, build_diagnosis_block, build_finding_summary, build_severity_tiers,
    build_thematic_clusters,
};
use self::executive::{build_executive_narrative, build_positive_signals};
use self::findings::{finding_group_from_normalized, recompute_occurrence_derived_fields};
use self::methodology::{build_appendix_block_from_normalized, build_methodology};
use self::module_details::build_module_details_from_normalized;
use self::modules_block::build_modules_block_from_normalized;
use self::positive::derive_positive_aspects_from_normalized;
use super::helpers::localized_module_name;

use crate::audit::normalized::AuditContext;
use crate::audit::summary::analyze_with_locale;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::wcag::Severity;

use super::actions::{derive_action_plan, humanize_action_text, impact_score};
use super::helpers::{
    build_benchmark_context, build_business_consequence, build_consequence_text,
    build_overall_impact, build_score_note, build_verdict_text, extract_domain,
    localized_report_subtitle, localized_report_title,
};

/// Build a complete ViewModel from a live audit context (single source of truth for score/grade/certificate).
/// For the cached/deserialized path, use `build_view_model_from_normalized` instead.
/// Names the measurements that hit the stability budget, with what they were
/// still seeing when it ran out.
///
/// `audit_quality.reasons` carries only a count
/// (`page_stability_budget_exhausted:2`), which tells a reader that something
/// was dropped but not what (plan 36 §5).
fn exhausted_stability_measurements(
    stability: &[crate::interaction::stability::StabilityProvenance],
    en: bool,
) -> String {
    use crate::interaction::stability::StabilityStatus;
    let parts: Vec<String> = stability
        .iter()
        .filter(|entry| entry.status == StabilityStatus::BudgetExhausted)
        .map(|entry| {
            let mut view = entry.viewport.clone();
            if let Some(first) = view.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            let mut text = if en {
                format!("{view} view after {} ms", entry.waited_ms)
            } else {
                format!("{view}-Ansicht nach {} ms", entry.waited_ms)
            };
            // Only when it was actually counted: `mutation_count` falls back
            // to 0 when the in-page script does not report one, and "still 0
            // DOM changes" is a default printed as a measurement.
            if entry.mutation_count > 0 {
                text.push_str(&if en {
                    format!(", still {} DOM changes", entry.mutation_count)
                } else {
                    format!(", noch {} DOM-Änderungen", entry.mutation_count)
                });
            }
            if let Some(reason) = entry.reason.as_deref().filter(|r| !r.is_empty()) {
                text.push_str(&format!(" ({reason})"));
            }
            text
        })
        .collect();
    if parts.is_empty() {
        return if en {
            "individual measurements".to_string()
        } else {
            "einzelne Messungen".to_string()
        };
    }
    parts.join(", ")
}

pub fn build_view_model(normalized: &AuditContext<'_>, config: &ReportConfig) -> ReportViewModel {
    let i18n = I18n::new(&config.locale)
        .or_else(|_| I18n::new("de"))
        .expect("default locale must always load");
    let priority_by_rule: std::collections::HashMap<&str, f32> = normalized
        .normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.priority_score))
        .collect();

    let mut sorted_groups: Vec<FindingGroup> = normalized
        .normalized
        .findings
        .iter()
        .filter(|f| match config.level {
            ReportLevel::Executive => f.report_visibility.executive,
            ReportLevel::Standard => f.report_visibility.standard,
            ReportLevel::Technical => f.report_visibility.technical,
        })
        .map(|f| finding_group_from_normalized(&i18n, f))
        .collect();
    sorted_groups.sort_by(|a, b| {
        let pa = priority_by_rule
            .get(a.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        let pb = priority_by_rule
            .get(b.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        pb.partial_cmp(&pa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| impact_score(b).cmp(&impact_score(a)))
    });

    // Deduplicate findings with the same title (e.g. WCAG + SEO rules detecting
    // the same issue). Prefer the non-"unknown." rule_id; merge occurrence counts.
    {
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut deduped: Vec<FindingGroup> = Vec::with_capacity(sorted_groups.len());
        for group in sorted_groups {
            let key = group.title.trim().to_lowercase();
            if let Some(&idx) = seen.get(&key) {
                let existing = &mut deduped[idx];
                if existing.rule_id.starts_with("unknown.")
                    && !group.rule_id.starts_with("unknown.")
                {
                    let merged = existing.occurrence_count + group.occurrence_count;
                    *existing = group;
                    existing.occurrence_count = merged;
                } else {
                    existing.occurrence_count += group.occurrence_count;
                }
                recompute_occurrence_derived_fields(existing, &i18n);
            } else {
                seen.insert(key, deduped.len());
                deduped.push(group);
            }
        }
        sorted_groups = deduped;
    }

    let score = normalized.normalized.score;
    let grade = normalized.normalized.grade.clone();
    let certificate = normalized.normalized.certificate.clone();
    let audit_summary = analyze_with_locale(&normalized.normalized, &config.locale);
    let maturity_label = audit_summary.site_state.label_localized(&i18n);
    let problem_type = audit_summary.problem_type_label.clone();
    let mut technical_overview: Vec<String> = normalized
        .normalized
        .interpretation
        .as_ref()
        .map(|interp| {
            interp
                .technical_overview
                .iter()
                .map(|t| t.for_locale(&config.locale).to_string())
                .collect()
        })
        .unwrap_or_default();
    for cross in &audit_summary.cross_impacts {
        technical_overview.push(format!(
            "Cross-Impact {}: {}",
            cross.dimensions, cross.description
        ));
    }
    let overall_impact = build_overall_impact(&config.locale, &normalized.normalized);
    let date_fmt = i18n.t("date-format-str");
    let date = normalized
        .normalized
        .timestamp
        .format(&date_fmt)
        .to_string();
    let report_title = localized_report_title(&config.locale);
    let report_subtitle = localized_report_subtitle(&config.locale);
    let report_author = extract_domain(&normalized.normalized.url);
    let top_findings: Vec<FindingGroup> = {
        use crate::output::report_model::CriticalityTier;
        // Prefer Mandatory (BFSG) tier first within the urgent (Critical/High) bucket — #245.
        let mandatory_urgent: Vec<FindingGroup> = sorted_groups
            .iter()
            .filter(|f| {
                f.criticality_tier == CriticalityTier::Mandatory
                    && matches!(f.severity, Severity::Critical | Severity::High)
            })
            .take(5)
            .cloned()
            .collect();

        let mut urgent = mandatory_urgent;
        if urgent.len() < 5 {
            let seen_ids: std::collections::HashSet<String> =
                urgent.iter().map(|f| f.rule_id.clone()).collect();
            let other_urgent: Vec<FindingGroup> = sorted_groups
                .iter()
                .filter(|f| {
                    !seen_ids.contains(&f.rule_id)
                        && matches!(f.severity, Severity::Critical | Severity::High)
                })
                .take(5 - urgent.len())
                .cloned()
                .collect();
            urgent.extend(other_urgent);
        }
        if urgent.len() < 5 {
            let seen_ids: std::collections::HashSet<String> =
                urgent.iter().map(|f| f.rule_id.clone()).collect();
            let remaining: Vec<FindingGroup> = sorted_groups
                .iter()
                .filter(|f| !seen_ids.contains(&f.rule_id))
                .take(5 - urgent.len())
                .cloned()
                .collect();
            urgent.extend(remaining);
        }
        urgent
    };
    let positive_aspects = derive_positive_aspects_from_normalized(&config.locale, normalized);
    let action_plan = derive_action_plan(&i18n, &sorted_groups);

    let mut module_names: Vec<String> = vec![localized_module_name("Accessibility", &i18n)];
    for module_name in &["Performance", "SEO", "Security", "Mobile", "UX", "Journey"] {
        if normalized
            .normalized
            .module_scores
            .iter()
            .any(|m| m.name == *module_name)
        {
            module_names.push(localized_module_name(module_name, &i18n));
        }
    }

    let (component_issues, component_occurrences) = sorted_groups
        .iter()
        .filter(|f| f.is_component_issue)
        .fold((0u32, 0u32), |(ci, co), f| {
            (ci + 1, co + f.occurrence_count as u32)
        });
    let severity = SeverityBlock {
        critical: normalized.normalized.occurrence_counts.critical as u32,
        high: normalized.normalized.occurrence_counts.high as u32,
        medium: normalized.normalized.occurrence_counts.medium as u32,
        low: normalized.normalized.occurrence_counts.low as u32,
        total: normalized.normalized.occurrence_counts.total as u32,
        has_issues: normalized.normalized.occurrence_counts.total > 0,
        component_issues,
        component_occurrences,
    };

    let modules = build_modules_block_from_normalized(&i18n, normalized);

    let quick_win_count = action_plan.quick_wins.len();
    let critical_count = (normalized.normalized.occurrence_counts.critical
        + normalized.normalized.occurrence_counts.high) as u32;
    // WCAG-only by design (severity/occurrence_counts are the legally-relevant
    // accessibility counters; SEO findings are reported separately). The cover
    // label makes the accessibility scope explicit so this does not read as a
    // contradiction with cross-module measures (#446 area).
    let total_violations = normalized.normalized.occurrence_counts.total as u32;
    let nodes_analyzed = normalized.normalized.nodes_analyzed;
    let warning_count = normalized.raw_wcag.warnings.len() as u32;
    let not_testable_count = normalized.raw_wcag.not_testables.len() as u32;
    // Confirmed / warning / manual-check tally. Scores are derived from
    // `confirmed_violations` alone (unchanged); the other two classes exist so
    // no sentence in the report can claim "no findings" while other modules
    // render findings a few pages later.
    let evidence = crate::output::report_model::EvidenceClasses {
        confirmed_violations: total_violations,
        warnings: warning_count
            + normalized.normalized.interactive_findings.len() as u32
            + normalized
                .normalized
                .screen_reader
                .as_ref()
                .map(|sr| sr.issues.len() as u32)
                .unwrap_or(0),
        manual_checks: not_testable_count,
    };

    let actions = build_actions_block(&i18n, &action_plan, score as f32, &audit_summary.site_state);

    let module_details = build_module_details_from_normalized(&i18n, normalized);
    let executive = build_executive_narrative(
        &i18n,
        &normalized.normalized,
        &audit_summary,
        &severity,
        &top_findings,
    );

    // The same verdict the CLI prints and the JSON publishes — the PDF must
    // not tell a different story than `report-lint` and CI (plan 34).
    let verdict_result = crate::audit::compute_verdict(
        &normalized.normalized,
        &crate::cli::config::VerdictConfig::default(),
    );

    let management_risks = crate::audit::management_risk::build_management_risk_kinds(
        std::slice::from_ref(&normalized.normalized),
    );

    ReportViewModel {
        meta: MetaBlock {
            title: report_title.clone(),
            subtitle: normalized.normalized.url.clone(),
            date: date.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: report_author.clone(),
            report_level: config.level,
            score_label: format!("{}/100", score),
        },
        cover: CoverBlock {
            brand: report_author,
            title: report_title,
            domain: normalized.normalized.url.clone(),
            subtitle: report_subtitle.to_string(),
            date: date.clone(),
            score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            maturity_label: maturity_label.clone(),
            total_issues: total_violations,
            critical_issues: critical_count,
            modules: module_names,
            desktop_score: normalized
                .normalized
                .viewport_scores
                .as_ref()
                .map(|vs| vs.desktop.accessibility),
            mobile_score: normalized
                .normalized
                .viewport_scores
                .as_ref()
                .map(|vs| vs.mobile.accessibility),
        },
        summary: SummaryBlock {
            score,
            overall_score: normalized.normalized.overall_score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            maturity_label: maturity_label.clone(),
            problem_type: problem_type.clone(),
            domain: normalized.normalized.url.clone(),
            date: date.clone(),
            executive_lead: audit_summary.verdict_intro.clone(),
            dominant_issue_note: audit_summary.dominant_issue_note.clone(),
            audit_quality_note: {
                use crate::audit::AuditQualityStatus;
                let en = i18n.locale() == "en";
                match normalized.normalized.execution.quality.status {
                    AuditQualityStatus::Complete => None,
                    // Framed as a normal automated-audit coverage limitation, not
                    // a tool defect: the audit itself completed successfully,
                    // only individual measurements hit a stability/retry budget
                    // and are therefore excluded from the reported scope
                    // (feedback: "Dieser Prüflauf ist unvollständig" next to a
                    // clean run read as if the tool had failed).
                    // The affected measurements are named here rather than
                    // deferred: the note used to end in "Details im
                    // Methodik-Anhang", and the appendix only restated the
                    // same sentence, so which measurements were dropped was
                    // never said anywhere (plan 36 §5). The data sits in
                    // `execution.navigation.stability`.
                    AuditQualityStatus::Partial => {
                        let exhausted = exhausted_stability_measurements(
                            &normalized.normalized.execution.navigation.stability,
                            en,
                        );
                        Some(if en {
                            format!(
                                "The automated audit completed successfully. Individual \
                                 measurements hit a stability/retry budget and are excluded from \
                                 the reported scores, which describe only the successfully \
                                 measured scope. Affected: {exhausted}."
                            )
                        } else {
                            format!(
                                "Der automatisierte Audit wurde erfolgreich durchgeführt. \
                                 Einzelne Messungen haben ein Stabilitäts-/Wiederholungsbudget \
                                 erreicht und sind daher nicht in den ausgewiesenen Scores \
                                 enthalten; diese beschreiben den erfolgreich gemessenen Umfang. \
                                 Betroffen: {exhausted}."
                            )
                        })
                    }
                    AuditQualityStatus::Insufficient => Some(if en {
                        "This audit run has insufficient data quality: several measurements \
                         failed. Scores and findings below may be incomplete or unreliable. \
                         See the methodology appendix for details."
                            .to_string()
                    } else {
                        "Dieser Prüflauf hat unzureichende Datenqualität: mehrere Messungen sind \
                         fehlgeschlagen. Scores und Befunde können unvollständig oder \
                         unzuverlässig sein. Details im Methodik-Anhang."
                            .to_string()
                    }),
                }
            },
            audit_quality_severe: normalized.normalized.execution.quality.status
                == crate::audit::AuditQualityStatus::Insufficient,
            verdict: build_verdict_text(
                &i18n,
                &normalized.normalized.url,
                score as f32,
                &normalized.normalized,
            ),
            ci_verdict: verdict_result.verdict,
            ci_verdict_reasons: verdict_result.reason_kinds,
            run_is_partial: normalized.normalized.execution.quality.status
                != crate::audit::AuditQualityStatus::Complete,
            score_calculation_method: normalized.normalized.score_calculation_method.clone(),
            score_breakdown: normalized.normalized.score_breakdown.clone(),
            score_note: build_score_note(&i18n, &normalized.normalized),
            metrics: {
                let label_violations_total = i18n.t("metric-violations-total");
                let label_critical = i18n.t("metric-critical-high");
                let label_checked_nodes = i18n.t("metric-checked-nodes");
                let label_quick_wins: String = "Quick Wins".into();
                let label_wcag_level = i18n.t("metric-wcag-level");
                let label_warnings = i18n.t("metric-warnings");
                let label_not_testable = i18n.t("metric-not-testable");
                // The headline metric is the accessibility score — what the
                // report assesses and what grade and certificate classify
                // (plan 29, D1). The weighted combined value has its own
                // named card in the score-driver section.
                let label_accessibility_score = i18n.t("metric-accessibility-score");
                vec![
                    MetricItem {
                        title: label_accessibility_score,
                        value: normalized.normalized.score.to_string(),
                        accent_color: Some("#0f766e".into()),
                    },
                    MetricItem {
                        title: label_violations_total,
                        value: total_violations.to_string(),
                        accent_color: Some("#f59e0b".into()),
                    },
                    MetricItem {
                        title: label_critical,
                        value: critical_count.to_string(),
                        accent_color: Some("#ef4444".into()),
                    },
                    MetricItem {
                        title: label_checked_nodes,
                        value: nodes_analyzed.to_string(),
                        accent_color: Some("#2563eb".into()),
                    },
                    MetricItem {
                        title: label_quick_wins.clone(),
                        value: quick_win_count.to_string(),
                        accent_color: Some("#7c3aed".into()),
                    },
                    MetricItem {
                        title: label_wcag_level,
                        value: normalized.normalized.wcag_level.to_string(),
                        accent_color: Some("#22c55e".into()),
                    },
                ]
                .into_iter()
                .chain(if evidence.warnings > 0 {
                    Some(MetricItem {
                        title: label_warnings,
                        value: evidence.warnings.to_string(),
                        accent_color: Some("#f97316".into()),
                    })
                } else {
                    None
                })
                .chain(if evidence.manual_checks > 0 {
                    Some(MetricItem {
                        title: label_not_testable,
                        value: evidence.manual_checks.to_string(),
                        accent_color: Some("#6b7280".into()),
                    })
                } else {
                    None
                })
                .collect()
            },
            top_actions: top_findings
                .iter()
                .take(3)
                .map(|f| humanize_action_text(&i18n, &f.recommendation))
                .collect(),
            positive_aspects: positive_aspects
                .iter()
                .map(|a| format!("{}: {}", a.area, a.description))
                .collect(),
            overall_impact,
            technical_overview,
            benchmark_context: build_benchmark_context(&config.locale, &normalized.normalized),
            business_consequence: build_business_consequence(&i18n, &normalized.normalized),
            consequence: build_consequence_text(&i18n, &normalized.normalized),
            risk_level: normalized.normalized.risk.level.label_localized(&i18n),
            risk_summary: normalized.normalized.risk.summary_for(&config.locale),
        },
        executive,
        methodology: build_methodology(&i18n, &normalized.normalized),
        modules,
        severity,
        findings: {
            let clusters = build_thematic_clusters(&config.locale, &sorted_groups);
            let finding_summary = build_finding_summary(
                &config.locale,
                &normalized.normalized.occurrence_counts,
                &audit_summary,
            );
            let by_severity = build_severity_tiers(&config.locale, &sorted_groups);
            let by_tier = build_criticality_groups(&config.locale, &sorted_groups);
            FindingsBlock {
                summary: finding_summary,
                evidence,
                clusters,
                top_findings,
                by_severity,
                by_tier,
                all_findings: sorted_groups,
            }
        },
        diagnosis: build_diagnosis_block(&config.locale, &normalized.normalized, &audit_summary),
        module_details,
        actions,
        appendix: build_appendix_block_from_normalized(&config.locale, &normalized.normalized),
        positive_signals: build_positive_signals(&config.locale, normalized),
        management_risks,
    }
}

#[cfg(test)]
mod tests {
    use super::build_view_model;
    use crate::audit::{normalize, AuditReport};
    use crate::cli::WcagLevel;
    use crate::output::report_model::ReportConfig;
    use crate::wcag::{Severity, Violation, WcagResults};

    #[test]
    fn view_model_exposes_confidence_and_capabilities() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image missing alt attribute",
            "node-123",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(vm
            .methodology
            .confidence_summary
            .iter()
            .any(|(label, _)| label == "Audit-Vertrauen"));
        assert!(vm
            .methodology
            .capabilities
            .iter()
            .any(|cap| cap.signal == "WCAG-Regeln & Vorkommen"));
    }

    #[test]
    fn view_model_summary_counts_wcag_occurrences_not_seo_findings() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::High,
            "Missing accessible name",
            "node-123",
        ));

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1500,
        );
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            headings: crate::seo::HeadingStructure {
                h1_count: 2,
                issues: vec![crate::seo::HeadingIssue {
                    issue_type: "multiple_h1".to_string(),
                    message: "Multiple H1 headings".to_string(),
                    severity: Severity::Medium,
                }],
                ..Default::default()
            },
            ..Default::default()
        });

        let normalized = normalize(&report);
        assert_eq!(normalized.normalized.findings.len(), 2);
        assert_eq!(normalized.normalized.occurrence_counts.total, 1);

        let vm = build_view_model(&normalized, &ReportConfig::default());

        let total_metric = vm
            .summary
            .metrics
            .iter()
            .find(|m| m.title.contains("Verstöße"))
            .expect("total violations metric");
        assert_eq!(total_metric.value, "1");

        let urgent_metric = vm
            .summary
            .metrics
            .iter()
            .find(|m| m.title == "Kritisch / Hoch")
            .expect("critical/high metric");
        assert_eq!(urgent_metric.value, "1");
    }

    #[test]
    fn english_view_model_excludes_top_level_german_labels() {
        use crate::audit::normalize;
        use crate::audit::AuditReport;
        use crate::wcag::{Severity, Violation, WcagResults};

        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alternative text",
                "node-hero-image",
            )
            .with_selector("img.hero")
            .with_html_snippet("<img class=\"hero\" src=\"hero.jpg\">")
            .with_fix("Add a meaningful alt attribute"),
        );
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(
            &normalized,
            &ReportConfig {
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        );

        // Top-level cover and executive narrative must not contain core German labels.
        let exec = &vm.executive;
        let candidates: Vec<&str> = vec![
            exec.cover_eyebrow.as_str(),
            exec.cover_kicker.as_str(),
            exec.next_steps_callout_body.as_str(),
            vm.summary.verdict.as_str(),
            vm.summary.executive_lead.as_str(),
            vm.summary.maturity_label.as_str(),
            vm.summary.problem_type.as_str(),
            vm.summary.business_consequence.as_str(),
            vm.summary.consequence.as_str(),
            vm.summary.risk_level.as_str(),
        ];

        // Marker words that should never appear in a localized English narrative.
        let forbidden = [
            "Automatisierter",
            "Empfohlen",
            "Maßnahmen",
            "Verbesserungshebel",
            "Solide Basis",
            "Kernaussagen",
            "Zertifikat ",
            "Auswirkungen",
            "Hauptproblem",
            "Stark",
            "Instabil",
            "Nutzbarkeit",
            "Barrierefreiheits",
            "Optimierungshebel",
            "Feinschliff",
            "Empfohlene",
            "Wirkung einer Behebung",
            "Erreicht",
            "Gering",
            "Hoch",
            "Mittel",
        ];

        for text in &candidates {
            for word in &forbidden {
                assert!(
                    !text.contains(word),
                    "English ViewModel still contains German marker '{}': {}",
                    word,
                    text
                );
            }
        }

        // Risk level must be a localized English label.
        assert!(
            ["Critical", "High", "Medium", "Low"].contains(&vm.summary.risk_level.as_str()),
            "Risk level should be an English label, got {}",
            vm.summary.risk_level
        );

        // Site state (maturity_label) must be one of the English variants.
        assert!(
            ["Strong", "Stable foundation", "Unstable", "Critical"]
                .contains(&vm.summary.maturity_label.as_str()),
            "Maturity label should be English, got {}",
            vm.summary.maturity_label
        );
    }

    fn report_with_repeated_rule(
        url: &str,
        rule: &str,
        name: &str,
        severity: Severity,
        count: usize,
    ) -> AuditReport {
        let mut results = WcagResults::new();
        for idx in 0..count {
            results.add_violation(
                Violation::new(
                    rule,
                    name,
                    WcagLevel::AA,
                    severity,
                    format!("{name} occurrence {idx}"),
                    format!("node-{idx}"),
                )
                .with_selector(format!("#item-{idx}"))
                .with_fix(format!("Fix required for {name}")),
            );
        }
        AuditReport::new(url.to_string(), WcagLevel::AA, results, 1_500)
    }

    #[test]
    fn report_archetype_many_repeated_findings_is_component_pattern() {
        let report = report_with_repeated_rule(
            "https://auto-birne.example",
            "4.1.2",
            "Name, Role, Value",
            Severity::High,
            12,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(vm.severity.component_issues > 0);
        assert!(vm.severity.component_occurrences >= 10);
        assert!(vm
            .findings
            .top_findings
            .iter()
            .any(|f| f.is_component_issue && f.occurrence_count >= 10));
    }

    #[test]
    fn report_archetype_mixed_findings_keeps_breadth_visible() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::High,
            "Missing accessible name",
            "node-name",
        ));
        results.add_violation(Violation::new(
            "2.4.4",
            "Link Purpose",
            WcagLevel::A,
            Severity::Medium,
            "Generic link text",
            "node-link",
        ));
        let mut report = AuditReport::new(
            "https://berkeley.example".to_string(),
            WcagLevel::AA,
            results,
            1_500,
        );
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            headings: crate::seo::HeadingStructure {
                issues: vec![crate::seo::HeadingIssue {
                    issue_type: "empty_heading".to_string(),
                    message: "Leere Überschrift".to_string(),
                    severity: Severity::High,
                }],
                ..Default::default()
            },
            ..Default::default()
        });
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(vm.findings.all_findings.len() >= 3);
        assert_eq!(vm.severity.component_issues, 0);
        assert!(vm
            .findings
            .all_findings
            .iter()
            .any(|f| f.dimension.as_deref() == Some("SEO")));
    }

    #[test]
    fn report_archetype_single_design_issue_becomes_dominant_pattern() {
        let report = report_with_repeated_rule(
            "https://casoon.example",
            "1.4.3",
            "Contrast (Minimum)",
            Severity::High,
            25,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(vm.severity.component_issues > 0);
        assert!(vm.diagnosis.dominant_issue.is_some());
        assert!(vm
            .findings
            .top_findings
            .iter()
            .any(|f| f.is_component_issue && f.occurrence_count == 25));
    }

    #[test]
    fn report_archetype_clean_accessibility_still_has_module_context() {
        let report = AuditReport::new(
            "https://mv-lack.example".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1_500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert_eq!(vm.summary.score, 100);
        assert_eq!(vm.severity.total, 0);
        assert!(vm
            .summary
            .score_note
            .as_deref()
            .unwrap_or_default()
            .contains("keine bestätigten Accessibility-Verstöße"));
        assert!(vm.summary.verdict.contains("100/100"));
    }

    /// Builds an AuditReport with all modules registered in `active_modules()`.
    fn all_active_modules_report() -> AuditReport {
        use crate::audit::PerformanceResults;
        use crate::dark_mode::DarkModeAnalysis;
        use crate::mobile::{
            ContentSizing, FontSizeAnalysis, MobileFriendliness, TouchTargetAnalysis,
            ViewportAnalysis,
        };
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        )
        .with_performance(PerformanceResults {
            vitals: WebVitals::default(),
            score: PerformanceScore {
                overall: 80,
                grade: PerformanceGrade::Gold,
                lcp_score: None,
                fcp_score: None,
                cls_score: None,
                interactivity_score: None,
                si_score: None,
                metrics_available: 0,
                size_penalty: None,
                js_penalty: None,
                request_penalty: None,
                dom_penalty: None,
                is_capped: None,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: vec![],
        })
        .with_seo(crate::seo::SeoAnalysis::default())
        .with_security(crate::security::SecurityAnalysis {
            score: 80,
            grade: "B".into(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
            protection: Default::default(),            sourcemap_leaks: Default::default(),
        })
        .with_html_conform(crate::html_conform::HtmlConformAnalysis {
            score: 90,
            checked: true,
            error_count: 0,
            warning_count: 1,
            info_count: 0,
            distinct_defect_count: 1,
            findings: vec![],
            raw_html: None,
        })
        .with_mobile(MobileFriendliness {
            score: 75,
            viewport: ViewportAnalysis::default(),
            touch_targets: TouchTargetAnalysis::default(),
            font_sizes: FontSizeAnalysis::default(),
            content_sizing: ContentSizing::default(),
            issues: vec![],
        })
        .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
        .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()))
        .with_dark_mode(DarkModeAnalysis {
            supported: false,
            class_based_dark_mode: false,
            score: 50,
            detection_methods: vec![],
            color_scheme_css: false,
            meta_color_scheme: None,
            meta_theme_color_dark: false,
            css_custom_properties: 0,
            dark_contrast_violations: 0,
            light_only_violations: 0,
            dark_only_violations: 0,
            contrast_violations: vec![],
            print: Default::default(),
            forced_colors: Default::default(),
            vision_deficiency: Default::default(),
            issues: vec![],
        })
        .with_best_practices(crate::best_practices::BestPracticesAnalysis {
            console_errors: crate::best_practices::ConsoleErrorsAnalysis {
                errors: vec![],
                warnings: vec![],
                error_count: 0,
                warning_count: 0,
            },
            vulnerable_libraries: crate::best_practices::VulnerableLibrariesAnalysis {
                detected: vec![],
                vulnerable: vec![],
                has_vulnerabilities: false,                duplicate_libraries: vec![],
            },
            score: 100,
        })
        .with_tech_stack(crate::tech_stack::TechStackAnalysis {
            detected: vec![],
            findings: vec![],
            score: 100,
            grade: "A".into(),
        })
        .with_patterns(crate::patterns::PatternAnalysis {
            recognized: vec![],
            violations: vec![],
            journey_candidates: vec![],
        })
        .with_design_quality(crate::design_quality::DesignQualityAnalysis {
            findings: vec![crate::design_quality::DesignQualityFinding {
                rule_id: "design.line_length".to_string(),
                level: crate::design_quality::FindingLevel::Advisory,
                confidence: crate::design_quality::Confidence::Medium,
                measurement_type: "heuristic".to_string(),
                selector: "p.body-text".to_string(),
                evidence: "~110 characters per line across 3 lines.".to_string(),
                message: "This body text wraps at a line length that may reduce readability."
                    .to_string(),
            }],
            rules_run: vec!["design.line_length".to_string()],
        })
        .with_ai_transparency(crate::ai_transparency::AiTransparencyAnalysis {
            findings: vec![crate::ai_transparency::ImageProvenanceFinding {
                rule_id: "ai_transparency.c2pa_provenance".to_string(),
                image_url: "https://example.com/hero.jpg".to_string(),
                provenance: crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicMedia,
                validation: crate::ai_transparency::ManifestValidation::Valid,
                generator: Some("Test Generative Tool 1.0".to_string()),
                measurement_type: "c2pa_manifest".to_string(),
                evidence: "C2PA manifest asserts digitalSourceType 'trained_algorithmic_media' (generator: Test Generative Tool 1.0), manifest validation: valid.".to_string(),
                message: crate::ai_transparency::finding_message_text(
                    "ai_transparency.c2pa_provenance",
                    true,
                ),
            }],
            images_checked: 3,
        })
        .with_network_dns(crate::network::dns::build_analysis(
            "example.com",
            &crate::network::dns::DnsCheckResult {
                caa_present: false,
                dnssec_detected: true,
                mx_present: true,
                spf_present: false,
            },
        ));
        let sq = crate::source_quality::analyze_source_quality(&report);
        let av = crate::ai_visibility::analyze_ai_visibility(&report);
        report.discoverability.source_quality = Some(sq);
        report.discoverability.ai_visibility = Some(av);
        report.discoverability.content_visibility = Some(
            crate::content_visibility::analyze_content_visibility(&report),
        );
        report.commerce = Some(crate::commerce::CommerceAnalysis {
            page_kind: crate::commerce::CommercePageKind::ProductDetail,
            product: Some(crate::commerce::ProductCommerce {
                price: Some(crate::commerce::PriceInfo {
                    value: "19.99".to_string(),
                    currency: Some("EUR".to_string()),
                    valid_until: None,
                }),
                availability: Some("InStock".to_string()),
                delivery_time: None,
                shipping: crate::commerce::ShippingInfo::default(),
                returns: crate::commerce::ReturnsInfo::default(),
                reviews: crate::commerce::ReviewInfo::default(),
                score: 40,
            }),
            trust_pages: crate::commerce::TrustPages {
                impressum: true,
                agb: true,
                widerruf: false,
                versand: true,
                zahlungsarten: true,
                kontakt: true,
            },
            identification_function_detected: false,
            identification_function_bfsg_reference: None,
            findings: vec![
                crate::commerce::CommerceFinding {
                    kind: crate::commerce::CommerceFindingKind::MissingShippingDetails,
                    severity: Severity::Medium,
                    message: crate::commerce::commerce_finding_text(
                        crate::commerce::CommerceFindingKind::MissingShippingDetails,
                        true,
                    ),
                },
                crate::commerce::CommerceFinding {
                    kind: crate::commerce::CommerceFindingKind::MissingWiderrufLink,
                    severity: Severity::High,
                    message: crate::commerce::commerce_finding_text(
                        crate::commerce::CommerceFindingKind::MissingWiderrufLink,
                        true,
                    ),
                },
            ],
        });
        report
    }

    fn report_with_all_report_areas() -> AuditReport {
        let mut report = all_active_modules_report();
        report.accessibility.wcag_results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image is missing alternative text",
                "img",
            )
            .with_rule_id("image-alt"),
        );
        report.accessibility.wcag_results.add_violation(
            Violation::new(
                "2.4.10",
                "Section Headings",
                WcagLevel::AAA,
                Severity::Low,
                "Long sections are not split by headings",
                "section",
            )
            .with_rule_id("heading-order"),
        );
        report.viewport_scores = Some(crate::audit::ViewportScores {
            desktop: crate::audit::ViewportScoreSet {
                accessibility: 95,
                performance: Some(90),
                overall: 92,
            },
            mobile: crate::audit::ViewportScoreSet {
                accessibility: 65,
                performance: Some(75),
                overall: 70,
            },
            weighted_overall: 77,
        });
        report.consent_banner_detected = true;
        report.consent_banner_cmp = Some("FixtureCMP".to_string());
        report
            .interactive_findings
            .push(crate::audit::normalized::InteractiveFinding {
                category: "SkipLink".to_string(),
                kind: crate::audit::normalized::InteractiveFindingKind::SkipLinkFocusNotMoved,
                maps_to_finding: None,
                severity: Severity::High,
                journey: "skip_link".to_string(),
                before_snapshot_label: Some("before".to_string()),
                after_snapshot_label: Some("after".to_string()),
                message: "Skip link does not move focus".to_string(),
                fix_suggestion: Some("Move focus to the main content target".to_string()),
                values: crate::audit::normalized::InteractiveFindingValues::default(),
            });
        report.accessibility_journey = Some(crate::audit::normalized::AccessibilityJourney {
            execution: Default::default(),
            focus_evidence: Vec::new(),
            traces: vec![crate::audit::normalized::JourneyTrace {
                journey: "skip_link".to_string(),
                steps: vec![crate::audit::normalized::JourneyStep {
                    action: "enter".to_string(),
                    target: Some("a[href=\"#main\"]".to_string()),
                    focus: Some("body".to_string()),
                    result: Some("focus_lost_to_body".to_string()),
                    snapshot_label: Some("after".to_string()),
                }],
            }],
        });
        report.screen_reader_audit = Some(crate::screen_reader::build_sr_audit_report(
            &report.url,
            report.timestamp,
            &crate::AXTree::new(),
            "en",
            None,
        ));
        report
    }

    #[test]
    fn test_pdf_viewmodel_covers_all_active_modules() {
        use crate::output::module::active_modules;

        let report = all_active_modules_report();
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());
        let details = &vm.module_details;

        let active_keys: std::collections::BTreeSet<&str> = active_modules(&report)
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        if active_keys.contains("performance") {
            assert!(
                details.performance.is_some(),
                "ModuleDetailsBlock.performance must be Some"
            );
        }
        if active_keys.contains("seo") {
            assert!(details.seo.is_some(), "ModuleDetailsBlock.seo must be Some");
        }
        if active_keys.contains("security") {
            assert!(
                details.security.is_some(),
                "ModuleDetailsBlock.security must be Some"
            );
        }
        if active_keys.contains("html_conform") {
            assert!(
                details.html_conform.is_some(),
                "ModuleDetailsBlock.html_conform must be Some"
            );
        }
        if active_keys.contains("commerce") {
            assert!(
                details.commerce.is_some(),
                "ModuleDetailsBlock.commerce must be Some"
            );
        }
        if active_keys.contains("mobile") {
            assert!(
                details.mobile.is_some(),
                "ModuleDetailsBlock.mobile must be Some"
            );
        }
        if active_keys.contains("ux") {
            assert!(details.ux.is_some(), "ModuleDetailsBlock.ux must be Some");
        }
        if active_keys.contains("journey") {
            assert!(
                details.journey.is_some(),
                "ModuleDetailsBlock.journey must be Some"
            );
        }
        if active_keys.contains("dark_mode") {
            assert!(
                details.dark_mode.is_some(),
                "ModuleDetailsBlock.dark_mode must be Some"
            );
        }
        if active_keys.contains("source_quality") {
            assert!(
                details.source_quality.is_some(),
                "ModuleDetailsBlock.source_quality must be Some"
            );
        }
        if active_keys.contains("ai_visibility") {
            assert!(
                details.ai_visibility.is_some(),
                "ModuleDetailsBlock.ai_visibility must be Some"
            );
        }
        if active_keys.contains("content_visibility") {
            assert!(
                details.content_visibility.is_some(),
                "ModuleDetailsBlock.content_visibility must be Some"
            );
        }
        if active_keys.contains("best_practices") {
            assert!(
                details.best_practices.is_some(),
                "ModuleDetailsBlock.best_practices must be Some"
            );
        }
    }

    /// Parity: active_modules() keys must equal pdf_rendered_modules() keys.
    #[test]
    fn test_module_parity_json_vs_pdf_viewmodel() {
        use crate::output::module::active_modules;
        use std::collections::BTreeSet;

        let report = all_active_modules_report();

        let json_keys: BTreeSet<&str> = active_modules(&report)
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        let pdf_keys = super::module_details::pdf_rendered_modules();

        let only_json: Vec<&&str> = json_keys.difference(&pdf_keys).collect();
        let only_pdf: Vec<&&str> = pdf_keys.difference(&json_keys).collect();

        assert!(
            only_json.is_empty() && only_pdf.is_empty(),
            "Module set mismatch:\n  only in JSON (active_modules): {:?}\n  only in PDF ViewModel (ModuleDetailsBlock): {:?}",
            only_json,
            only_pdf,
        );
    }

    /// The commerce PDF chapter is deliberately built only when there is
    /// substantive content (product decision, 2026-09-01): a shop page with
    /// `page_kind: Other` and every mandatory/trust page already linked has
    /// nothing beyond "all six links exist" to add.
    #[test]
    fn test_commerce_details_absent_when_nothing_meaningful_to_show() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        );
        report.commerce = Some(crate::commerce::CommerceAnalysis {
            page_kind: crate::commerce::CommercePageKind::Other,
            product: None,
            trust_pages: crate::commerce::TrustPages {
                impressum: true,
                agb: true,
                widerruf: true,
                versand: true,
                zahlungsarten: true,
                kontakt: true,
            },
            identification_function_detected: false,
            identification_function_bfsg_reference: None,
            findings: vec![],
        });

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert!(
            vm.module_details.commerce.is_none(),
            "commerce chapter must stay absent when page_kind is Other and every \
             trust page is already linked"
        );
    }

    /// Documents that `patterns` is rendered in both JSON (via ModuleBlob) and PDF
    /// and remains registered in `active_modules()` / `ModuleDetailsBlock`.
    #[test]
    fn test_patterns_parity_with_active_modules() {
        use crate::output::module::active_modules;

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        )
        .with_patterns(crate::patterns::PatternAnalysis {
            recognized: vec![],
            violations: vec![],
            journey_candidates: vec![],
        });

        let active_keys: std::collections::BTreeSet<&str> = active_modules(&report)
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        assert!(
            active_keys.contains("patterns"),
            "patterns is in AuditReport (and emitted in JSON/PDF) but missing from active_modules() — \
             add ReportModule impl for PatternAnalysis and register it in active_modules()"
        );

        let pdf_keys = super::module_details::pdf_rendered_modules();
        assert!(
            pdf_keys.contains("patterns"),
            "patterns is missing from ModuleDetailsBlock — \
             add a patterns field and update pdf_rendered_modules()"
        );
    }

    #[test]
    #[cfg(feature = "pdf")]
    fn test_active_modules_have_json_detail_and_pdf_registry_coverage() {
        use crate::output::json::UnifiedReport;
        use crate::output::module::{active_modules, active_report_modules};
        use std::collections::BTreeSet;

        let report = all_active_modules_report();
        let active_keys: BTreeSet<&str> = active_modules(&report)
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).expect("JSON should render");
        let json_value: serde_json::Value =
            serde_json::from_str(&json_str).expect("JSON should parse");
        let modules = json_value["pages"][0]["detail"]["modules"]
            .as_object()
            .expect("single report detail modules must be an object");
        let json_keys: BTreeSet<&str> = modules
            .iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(key, _)| key.as_str())
            .collect();
        let pdf_keys = super::module_details::pdf_rendered_modules();
        let i18n = crate::i18n::I18n::new("de").expect("test locale should load");
        let missing_pdf_hooks: Vec<&str> = active_report_modules(&report)
            .into_iter()
            .filter(|module| module.render_pdf(&i18n).is_empty())
            .map(|module| module.module_key())
            .collect();

        let missing_json: Vec<&&str> = active_keys.difference(&json_keys).collect();
        let missing_pdf: Vec<&&str> = active_keys.difference(&pdf_keys).collect();

        assert!(
            missing_json.is_empty() && missing_pdf.is_empty() && missing_pdf_hooks.is_empty(),
            "Active module coverage gap:\n  missing JSON detail entries: {:?}\n  missing PDF registry entries: {:?}\n  empty PDF hooks: {:?}",
            missing_json,
            missing_pdf,
            missing_pdf_hooks,
        );
    }

    #[test]
    fn test_report_areas_have_json_and_pdf_viewmodel_coverage() {
        use crate::output::json::UnifiedReport;
        use crate::output::module::REPORT_AREAS;

        let report = report_with_all_report_areas();
        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).expect("JSON should render");
        let json_value: serde_json::Value =
            serde_json::from_str(&json_str).expect("JSON should parse");
        let page = &json_value["pages"][0];
        let detail_modules = &page["detail"]["modules"];

        for area in REPORT_AREAS {
            let json_present = match *area {
                "wcag_findings" => {
                    page["findings"]
                        .as_array()
                        .is_some_and(|items| !items.is_empty())
                        && detail_modules["accessibility"]["findings"]
                            .as_array()
                            .is_some_and(|items| !items.is_empty())
                }
                "a11y_journey" => page["interactive_findings"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty()),
                "audit_flags" => page["audit_flags"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty()),
                "search_experience" => !detail_modules["search_experience"].is_null(),
                "screen_reader" => !page["screen_reader"].is_null(),
                "advisory_findings" => page["findings"].as_array().is_some_and(|items| {
                    items.iter().any(|item| {
                        item["category"] != "wcag"
                            || !matches!(item["wcag_level"].as_str(), Some("A") | Some("AA"))
                    })
                }),
                "score_breakdown" => !page["score_breakdown"].is_null(),
                unknown => panic!("REPORT_AREAS contains unmapped JSON area: {unknown}"),
            };

            let pdf_viewmodel_present = match *area {
                "wcag_findings" => !vm.findings.all_findings.is_empty(),
                "a11y_journey" => !report.interactive_findings.is_empty(),
                "audit_flags" => !normalized.normalized.audit_flags.is_empty(),
                "search_experience" => vm.module_details.search_experience.is_some(),
                "screen_reader" => report.screen_reader_audit.is_some(),
                "advisory_findings" => vm.findings.by_tier.iter().any(|group| {
                    group.tier == crate::output::report_model::CriticalityTier::Optimization
                        && group.total_findings > 0
                }),
                "score_breakdown" => normalized.normalized.score_breakdown.is_some(),
                unknown => panic!("REPORT_AREAS contains unmapped PDF area: {unknown}"),
            };

            assert!(
                json_present,
                "REPORT_AREAS JSON coverage missing for {area}"
            );
            assert!(
                pdf_viewmodel_present,
                "REPORT_AREAS PDF/ViewModel coverage missing for {area}"
            );
        }
    }
}
