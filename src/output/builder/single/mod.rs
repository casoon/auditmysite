//! Single-report ViewModel builder.

mod actions_block;
mod diagnosis;
mod executive;
mod findings;
mod history;
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
use self::findings::finding_group_from_normalized;
use self::history::build_history_trend_block;
use self::methodology::{build_appendix_block_from_normalized, build_methodology};
use self::module_details::build_module_details_from_normalized;
use self::modules_block::build_modules_block_from_normalized;
use self::positive::derive_positive_aspects_from_normalized;
use super::helpers::localized_module_name;

use crate::audit::normalized::NormalizedReport;
use crate::audit::summary::analyze_with_locale;
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::wcag::Severity;

use super::actions::{derive_action_plan, humanize_action_text, impact_score};
use super::helpers::{
    build_benchmark_context, build_business_consequence, build_consequence_text,
    build_overall_impact, build_score_note, build_technical_overview, build_verdict_text,
    extract_domain, localized_report_subtitle, localized_report_title,
};

/// Build a complete ViewModel from a normalized report (single source of truth for score/grade/certificate)
pub fn build_view_model(normalized: &NormalizedReport, config: &ReportConfig) -> ReportViewModel {
    let i18n = I18n::new(&config.locale)
        .or_else(|_| I18n::new("de"))
        .expect("default locale must always load");
    let priority_by_rule: std::collections::HashMap<&str, f32> = normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.priority_score))
        .collect();

    let mut sorted_groups: Vec<FindingGroup> = normalized
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
            } else {
                seen.insert(key, deduped.len());
                deduped.push(group);
            }
        }
        sorted_groups = deduped;
    }

    let score = normalized.score;
    let grade = normalized.grade.clone();
    let certificate = normalized.certificate.clone();
    let audit_summary = analyze_with_locale(normalized, &config.locale);
    let maturity_label = audit_summary.site_state.label_localized(&i18n);
    let problem_type = audit_summary.problem_type_label.clone();
    let mut technical_overview = build_technical_overview(&config.locale, normalized);
    for cross in &audit_summary.cross_impacts {
        technical_overview.push(format!(
            "Cross-Impact {}: {}",
            cross.dimensions, cross.description
        ));
    }
    let overall_impact = build_overall_impact(&config.locale, normalized);
    let date_fmt = i18n.t("date-format-str");
    let date = normalized.timestamp.format(&date_fmt).to_string();
    let report_title = localized_report_title(&config.locale);
    let report_subtitle = localized_report_subtitle(&config.locale);
    let report_author = extract_domain(&normalized.url);
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
    if normalized.raw_performance.is_some() {
        module_names.push(localized_module_name("Performance", &i18n));
    }
    if normalized.raw_seo.is_some() {
        module_names.push(localized_module_name("SEO", &i18n));
    }
    if normalized.raw_security.is_some() {
        module_names.push(localized_module_name("Security", &i18n));
    }
    if normalized.raw_mobile.is_some() {
        module_names.push(localized_module_name("Mobile", &i18n));
    }
    if normalized.raw_ux.is_some() {
        module_names.push(localized_module_name("UX", &i18n));
    }
    if normalized.raw_journey.is_some() {
        module_names.push(localized_module_name("Journey", &i18n));
    }

    let (component_issues, component_occurrences) = sorted_groups
        .iter()
        .filter(|f| f.is_component_issue)
        .fold((0u32, 0u32), |(ci, co), f| {
            (ci + 1, co + f.occurrence_count as u32)
        });
    let severity = SeverityBlock {
        critical: normalized.occurrence_counts.critical as u32,
        high: normalized.occurrence_counts.high as u32,
        medium: normalized.occurrence_counts.medium as u32,
        low: normalized.occurrence_counts.low as u32,
        total: normalized.occurrence_counts.total as u32,
        has_issues: normalized.occurrence_counts.total > 0,
        component_issues,
        component_occurrences,
    };

    let modules = build_modules_block_from_normalized(&i18n, normalized);

    let quick_win_count = action_plan.quick_wins.len();
    let critical_count =
        (normalized.occurrence_counts.critical + normalized.occurrence_counts.high) as u32;
    let total_violations = normalized.occurrence_counts.total as u32;
    let nodes_analyzed = normalized.nodes_analyzed;
    let warning_count = normalized.raw_wcag.warnings.len() as u32;
    let not_testable_count = normalized.raw_wcag.not_testables.len() as u32;

    let actions = build_actions_block(&i18n, &action_plan, score as f32, &audit_summary.site_state);

    let module_details = build_module_details_from_normalized(&i18n, normalized);
    let history = config
        .history_preview
        .as_ref()
        .map(|preview| build_history_trend_block(&i18n, preview));
    let executive = build_executive_narrative(
        &i18n,
        normalized,
        &audit_summary,
        score,
        &severity,
        &top_findings,
        &action_plan,
    );

    ReportViewModel {
        meta: MetaBlock {
            title: report_title.clone(),
            subtitle: normalized.url.clone(),
            date: date.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: report_author.clone(),
            report_level: config.level,
            score_label: format!("{}/100", score),
        },
        cover: CoverBlock {
            brand: report_author,
            title: report_title,
            domain: normalized.url.clone(),
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
                .viewport_scores
                .as_ref()
                .map(|vs| vs.desktop.accessibility),
            mobile_score: normalized
                .viewport_scores
                .as_ref()
                .map(|vs| vs.mobile.accessibility),
        },
        summary: SummaryBlock {
            score,
            overall_score: normalized.overall_score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            maturity_label: maturity_label.clone(),
            problem_type: problem_type.clone(),
            domain: normalized.url.clone(),
            date: date.clone(),
            executive_lead: audit_summary.verdict_intro.clone(),
            dominant_issue_note: audit_summary.dominant_issue_note.clone(),
            verdict: build_verdict_text(&i18n, &normalized.url, score as f32),
            score_note: build_score_note(&i18n, normalized),
            metrics: {
                let label_violations_total = i18n.t("metric-violations-total");
                let label_critical = i18n.t("metric-critical-high");
                let label_checked_nodes = i18n.t("metric-checked-nodes");
                let label_quick_wins: String = "Quick Wins".into();
                let label_wcag_level = i18n.t("metric-wcag-level");
                let label_warnings = i18n.t("metric-warnings");
                let label_not_testable = i18n.t("metric-not-testable");
                let label_overall_score = i18n.t("metric-overall-score");
                vec![
                    MetricItem {
                        title: label_overall_score,
                        value: normalized.overall_score.to_string(),
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
                        value: normalized.wcag_level.to_string(),
                        accent_color: Some("#22c55e".into()),
                    },
                ]
                .into_iter()
                .chain(if warning_count > 0 {
                    Some(MetricItem {
                        title: label_warnings,
                        value: warning_count.to_string(),
                        accent_color: Some("#f97316".into()),
                    })
                } else {
                    None
                })
                .chain(if not_testable_count > 0 {
                    Some(MetricItem {
                        title: label_not_testable,
                        value: not_testable_count.to_string(),
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
            benchmark_context: build_benchmark_context(&config.locale, score as f32),
            business_consequence: build_business_consequence(&i18n, normalized),
            consequence: build_consequence_text(&i18n, normalized),
            risk_level: normalized.risk.level.label_localized(&i18n),
            risk_summary: normalized.risk.summary_for(&config.locale),
        },
        executive,
        history,
        methodology: build_methodology(&i18n, normalized),
        modules,
        severity,
        findings: {
            let clusters = build_thematic_clusters(&config.locale, &sorted_groups);
            let finding_summary = build_finding_summary(
                &config.locale,
                &normalized.occurrence_counts,
                &audit_summary,
            );
            let by_severity = build_severity_tiers(&config.locale, &sorted_groups);
            let by_tier = build_criticality_groups(&config.locale, &sorted_groups);
            FindingsBlock {
                summary: finding_summary,
                clusters,
                top_findings,
                by_severity,
                by_tier,
                all_findings: sorted_groups,
            }
        },
        diagnosis: build_diagnosis_block(&config.locale, normalized, &audit_summary),
        module_details,
        actions,
        appendix: build_appendix_block_from_normalized(normalized),
        positive_signals: build_positive_signals(&config.locale, normalized),
    }
}

#[cfg(test)]
mod tests {
    use super::build_view_model;
    use crate::audit::{normalize, AuditReport};
    use crate::cli::WcagLevel;
    use crate::output::report_model::{ReportConfig, ReportHistoryPreview};
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
        report.seo = Some(crate::seo::SeoAnalysis {
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
        assert_eq!(normalized.findings.len(), 2);
        assert_eq!(normalized.occurrence_counts.total, 1);

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
    fn view_model_names_high_level_a_findings_as_high_not_critical_only() {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::High,
            "Missing accessible name",
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
        let risk_text = vm
            .executive
            .impact_rows
            .iter()
            .find(|(label, _)| label == "Risiko")
            .map(|(_, text)| text.as_str())
            .expect("risk row");

        assert!(risk_text.contains("hohe oder kritische WCAG-Level-A-Befunde"));
        assert!(!risk_text.contains("keine kritischen Level-A-Verstöße"));
    }

    #[test]
    fn view_model_exposes_history_delta_when_preview_exists() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1500,
        );
        let normalized = normalize(&report);
        let vm = build_view_model(
            &normalized,
            &ReportConfig {
                history_preview: Some(ReportHistoryPreview {
                    previous_date: "01.04.2026".to_string(),
                    timeline_entries: 3,
                    previous_accessibility_score: 74,
                    previous_overall_score: 78,
                    delta_accessibility: 8,
                    delta_overall: 5,
                    delta_total_issues: -6,
                    delta_critical_issues: -2,
                    recent_entries: vec![
                        ("01.04.2026".to_string(), 74, 78, "C".to_string(), 12),
                        (
                            "06.04.2026".to_string(),
                            normalized.score,
                            normalized.overall_score,
                            normalized.grade.clone(),
                            normalized.severity_counts.total as u32,
                        ),
                    ],
                    new_findings: vec!["Link-Purpose".to_string()],
                    resolved_findings: vec!["Alt-Text".to_string()],
                }),
                ..ReportConfig::default()
            },
        );

        let history = vm.history.expect("history block should exist");
        assert_eq!(history.trend_label, "Deutlich verbessert");
        assert!(history.summary.contains("01.04.2026"));
        assert_eq!(history.timeline_rows.len(), 2);
        assert_eq!(history.resolved_findings.len(), 1);
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
            exec.status_title.as_str(),
            exec.metrics_title.as_str(),
            exec.key_points_title.as_str(),
            exec.impact_title.as_str(),
            exec.quick_actions_title.as_str(),
            exec.spotlight_eyebrow.as_str(),
            exec.leverage_title.as_str(),
            exec.findings_title.as_str(),
            exec.findings_intro.as_str(),
            exec.action_plan_title.as_str(),
            exec.action_plan_intro.as_str(),
            exec.action_plan_callout_title.as_str(),
            exec.action_plan_callout_body.as_str(),
            exec.technical_title.as_str(),
            exec.technical_intro.as_str(),
            exec.next_steps_title.as_str(),
            exec.next_steps_intro.as_str(),
            exec.next_steps_callout_title.as_str(),
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
            ["Strong", "Solid foundation", "Unstable", "Critical"]
                .contains(&vm.summary.maturity_label.as_str()),
            "Maturity label should be English, got {}",
            vm.summary.maturity_label
        );
    }

    /// Builds an AuditReport with all 11 modules registered in `active_modules()`.
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
            protection: Default::default(),
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
                has_vulnerabilities: false,
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
        });
        let sq = crate::source_quality::analyze_source_quality(&report);
        let av = crate::ai_visibility::analyze_ai_visibility(&report);
        report.source_quality = Some(sq);
        report.ai_visibility = Some(av);
        report.content_visibility =
            Some(crate::content_visibility::ContentVisibilityAnalysis::default());
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
    ///
    /// Currently ignored: `tech_stack` is present in `ModuleDetailsBlock` but absent
    /// from `active_modules()` and therefore absent from the JSON output via that
    /// path. Un-ignore once the gap is resolved (add tech_stack to active_modules
    /// or remove it from ModuleDetailsBlock).
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

    /// Documents that `patterns` is rendered in both JSON (via ModuleBlob) and PDF
    /// but is absent from `active_modules()` and `ModuleDetailsBlock`.
    ///
    /// Un-ignore once patterns is added to both `active_modules()` (with a
    /// `ReportModule` impl) and `ModuleDetailsBlock` / `pdf_rendered_modules()`.
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
}
