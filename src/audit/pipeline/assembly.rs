//! Synchronous post-processing of the two viewport passes: merging the WCAG
//! results, scoring each viewport, aggregating the report and keeping its
//! module-run and audit-quality records consistent.

use tracing::warn;

use super::{PipelineConfig, SnapshotData};
use crate::audit::catalog::AuditCatalog;
use crate::audit::report::AuditReport;
use crate::browser::ThrottleProfile;
use crate::seo::HtmlValidationKind;
use crate::wcag::{self, Violation, WcagResults};

// ── Score helpers ─────────────────────────────────────────────────────────────

/// Compute a normalized module score for one viewport pass.
///
/// Weights: Accessibility 40 %, Performance 20 %, SEO 20 %, Mobile 10 %
/// (normalized to active modules, same as the single-pass formula).
pub(super) fn compute_viewport_overall(
    acc: f32,
    perf: Option<u32>,
    seo: Option<u32>,
    mobile: Option<u32>,
) -> u32 {
    let mut weighted = acc as f64 * 40.0;
    let mut total = 40.0;

    if let Some(p) = perf {
        weighted += p as f64 * 20.0;
        total += 20.0;
    }
    if let Some(s) = seo {
        weighted += s as f64 * 20.0;
        total += 20.0;
    }
    if let Some(m) = mobile {
        weighted += m as f64 * 10.0;
        total += 10.0;
    }

    (weighted / total).round() as u32
}

// ── WCAG deduplication ────────────────────────────────────────────────────────

/// Rules that run only on the mobile pass but judge a viewport-independent
/// property of the page: the HTML content model (raw markup) and reflow
/// (measured at 320 CSS px regardless of the audited viewport).
const SHARED_PAGE_RULE_IDS: [&str; 2] = [
    wcag::rules::HTML_CONTENT_MODEL_RULE.axe_id,
    wcag::rules::REFLOW_RULE.axe_id,
];

/// Desktop violations plus the mobile pass's violations of the shared page
/// rules, so both viewport scores are computed over the same rule set.
pub(super) fn with_shared_page_rule_violations(
    desktop: &[Violation],
    mobile: &[Violation],
) -> Vec<Violation> {
    let shared = mobile.iter().filter(|v| {
        v.rule_id
            .as_deref()
            .is_some_and(|id| SHARED_PAGE_RULE_IDS.contains(&id))
    });
    desktop.iter().chain(shared).cloned().collect()
}

/// Merge violations from both passes.
///
/// Dedup key: (rule, selector-or-node_id).
/// - Violations present on both → tag "both-viewports" (reported once)
/// - Desktop-only → tag "desktop-only"
/// - Mobile-only → tag "mobile-only"
pub(super) fn merge_wcag_violations(desktop: &WcagResults, mobile: &WcagResults) -> WcagResults {
    fn dedup_key(v: &Violation) -> (&str, String) {
        let id = v
            .selector
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| v.node_id.clone());
        (v.rule.as_str(), id)
    }

    fn merge_aux(desktop: &[Violation], mobile: &[Violation]) -> Vec<Violation> {
        let mut merged = Vec::new();
        for finding in mobile {
            let mut finding = finding.clone();
            finding.tags.push("mobile-only".to_string());
            merged.push(finding);
        }
        for finding in desktop {
            if let Some(existing) = merged.iter_mut().find(|candidate| {
                candidate.rule == finding.rule && candidate.message == finding.message
            }) {
                existing
                    .tags
                    .retain(|tag| tag != "mobile-only" && tag != "desktop-only");
                existing.tags.push("both-viewports".to_string());
            } else {
                let mut finding = finding.clone();
                finding.tags.push("desktop-only".to_string());
                merged.push(finding);
            }
        }
        merged
    }

    let mut merged: Vec<Violation> = Vec::new();
    let mut desktop_matched = vec![false; desktop.violations.len()];

    for mv in &mobile.violations {
        let mk = dedup_key(mv);
        // Only an unmatched desktop entry can pair up: several elements can
        // share a selector (18 × `ul.link-list__list` on sachsen-anhalt.de),
        // and reusing the first match left the other 17 desktop entries to be
        // appended as desktop-only — 35 occurrences for 18 elements.
        let match_idx = desktop
            .violations
            .iter()
            .enumerate()
            .position(|(i, dv)| !desktop_matched[i] && dedup_key(dv) == mk);

        if let Some(idx) = match_idx {
            desktop_matched[idx] = true;
            let mut shared = mv.clone();
            shared.tags.push("both-viewports".to_string());
            // Evidence-grade findings: prefer the desktop crop when a
            // violation is confirmed in both viewports (owner decision —
            // desktop crops are larger/more legible; the mobile pass skips
            // capturing a rule already captured on desktop, so this is
            // usually a no-op restoring what the mobile clone already lacks).
            if shared.evidence_screenshot.is_none() {
                shared.evidence_screenshot = desktop.violations[idx].evidence_screenshot.clone();
                shared.evidence_viewport = desktop.violations[idx].evidence_viewport;
            }
            merged.push(shared);
        } else {
            let mut mobile_only = mv.clone();
            mobile_only.tags.push("mobile-only".to_string());
            merged.push(mobile_only);
        }
    }

    for (i, dv) in desktop.violations.iter().enumerate() {
        if !desktop_matched[i] {
            let mut desktop_only = dv.clone();
            desktop_only.tags.push("desktop-only".to_string());
            merged.push(desktop_only);
        }
    }

    // Merge warnings, positives, and not_testables without deduplication
    // (heuristic/structural signals — viewport tagging not meaningful).
    let warnings = merge_aux(&desktop.warnings, &mobile.warnings);
    let positives = merge_aux(&desktop.positives, &mobile.positives);
    let not_testables = merge_aux(&desktop.not_testables, &mobile.not_testables);

    // #527: pixel sampling can classify the same element differently across
    // viewport passes (e.g. rendering/DPR differences on an actual <img>
    // behind text), which would otherwise double-report one real problem as
    // both a confirmed violation (from one viewport) and a manual-review
    // warning (from the other) for the same rule+selector. A confirmed
    // violation supersedes a "needs review" warning, so drop the warning.
    //
    // Same rule means same `rule_id`, not just the same criterion: the
    // shared rules report several page-level findings on `<html>`, and
    // `keyboard/skip-link-missing` (2.4.1, review) was swallowed by
    // `landmarks/main-missing` (2.4.1, fail) on the same element (#690).
    let violation_keys: std::collections::HashSet<(String, Option<String>, String)> = merged
        .iter()
        .map(|v| {
            let (rule, id) = dedup_key(v);
            (rule.to_owned(), v.rule_id.clone(), id)
        })
        .collect();
    let warnings: Vec<Violation> = warnings
        .into_iter()
        .filter(|w| {
            let (rule, id) = dedup_key(w);
            !violation_keys.contains(&(rule.to_owned(), w.rule_id.clone(), id))
        })
        .collect();

    WcagResults {
        violations: merged,
        warnings,
        positives,
        not_testables,
        passes: mobile.passes.max(desktop.passes),
        incomplete: mobile.incomplete.max(desktop.incomplete),
        nodes_checked: mobile.nodes_checked.max(desktop.nodes_checked),
        rule_outcomes: desktop
            .rule_outcomes
            .iter()
            .chain(&mobile.rule_outcomes)
            .cloned()
            .collect(),
        localized_texts: desktop
            .localized_texts
            .iter()
            .chain(&mobile.localized_texts)
            .map(|(en, de)| (en.clone(), de.clone()))
            .collect(),
    }
}

pub(super) fn aggregate_report(
    url: &str,
    config: &PipelineConfig,
    snapshot: &SnapshotData,
    wcag_results: WcagResults,
    pattern_analysis: crate::patterns::PatternAnalysis,
    duration_ms: u64,
) -> AuditReport {
    // Pattern violations were already enriched and merged into wcag_results by
    // the caller (audit_page), which has access to the live page for CDP lookups.
    let mut report = AuditReport::new(
        url.to_string(),
        config.wcag_level,
        wcag_results,
        duration_ms,
    );
    report.accessibility.execution.scope = audit_scope_from_config(config);
    report.accessibility.execution.navigation.requested_url = url.to_string();
    report.accessibility.execution.environment.source = "live".to_string();
    report.accessibility.execution.module_runs = consolidate_module_runs(&snapshot.module_runs);
    let sr_audit = crate::screen_reader::build_sr_audit_report(
        url,
        report.timestamp,
        &snapshot.ax_tree,
        &config.lang,
        Some(&pattern_analysis),
    );
    report = report.with_patterns(pattern_analysis);
    report.screen_reader_audit = Some(sr_audit);

    if let Some(performance) = snapshot.performance.clone() {
        report = report.with_performance(performance);
        attach_performance_subchecks(&mut report);
    }
    if let Some(seo) = snapshot.seo.clone() {
        report = report.with_seo(seo);
        reconcile_image_alt_count(&mut report);
    }
    if let Some(security) = snapshot.security.clone() {
        report = report.with_security(security);
    }
    if let Some(mobile) = snapshot.mobile.clone() {
        report = report.with_mobile(mobile);
    }
    if let Some(ux) = snapshot.ux.clone() {
        report = report.with_ux(ux);
    }
    if let Some(journey) = snapshot.journey.clone() {
        report = report.with_journey(journey);
    }
    if let Some(dark_mode) = snapshot.dark_mode.clone() {
        report = report.with_dark_mode(dark_mode);
    }
    if let Some(design_quality) = snapshot.design_quality.clone() {
        report = report.with_design_quality(design_quality);
    }
    if let Some(html_conform) = snapshot.html_conform.clone() {
        report = report.with_html_conform(html_conform);
    }
    if let Some(ai_transparency) = snapshot.ai_transparency.clone() {
        report = report.with_ai_transparency(ai_transparency);
    }
    if let Some(network_dns) = snapshot.network_dns.clone() {
        report = report.with_network_dns(network_dns);
    }

    if let Some(tech_stack) = snapshot.tech_stack.clone() {
        report = report.with_tech_stack(tech_stack);
    }

    if let Some(bp) = snapshot.best_practices.clone() {
        report = report.with_best_practices(bp);
    }

    // Post-processing modules (source_quality, ai_visibility, content_visibility)
    // run via the catalog's derive phase. Topo order ensures content_visibility
    // sees source_quality + ai_visibility populated.
    if let Err(e) = AuditCatalog::standard().derive_all(&mut report, config) {
        warn!("Post-processing derive_all failed: {}", e);
    }
    report.accessibility.execution.module_runs =
        consolidate_module_runs(&report.accessibility.execution.module_runs);

    report
}

/// Reconciles `page_health`'s "images without alt" HTML-validation
/// entry with the canonical WCAG 1.1.1 violation count (#574).
///
/// `page_health::analyze_url` computes `images_without_alt` from a raw DOM
/// probe (`document.querySelectorAll('img:not([alt])')`) that runs before
/// the AXTree/WCAG pass exists, so it has no way to know which of those
/// `<img>` elements the accessibility tree already excludes as legitimately
/// decorative or AT-ignored (hidden, `aria-hidden`, off-screen carousel
/// slides not yet rendered, etc.) — the WCAG 1.1.1 rule is the authoritative
/// check for "does this image have a text alternative" and already accounts
/// for that. Left alone, the two report sections independently re-derive
/// "the same" fact and can show contradicting numbers for the same page
/// (e.g. "42 images without alt" in the SEO/HTML-validation table next to
/// "all images have alt text" in source_quality). This runs once WCAG
/// results and SEO/page_health are both attached to `report`, overwriting
/// `images_without_alt` and its `html_issues` entry with the canonical
/// `WcagResults::count_by_rule("1.1.1")` value.
pub(super) fn reconcile_image_alt_count(report: &mut AuditReport) {
    let canonical = report.accessibility.wcag_results.count_by_rule("1.1.1") as u32;
    let Some(page_health) = report
        .discoverability
        .seo
        .as_mut()
        .and_then(|seo| seo.page_health.as_mut())
    else {
        return;
    };
    page_health.images_without_alt = canonical;
    page_health
        .html_issues
        .retain(|issue| issue.kind != HtmlValidationKind::ImagesWithoutAlt);
    if canonical > 0 {
        page_health
            .html_issues
            .push(crate::seo::HtmlValidationIssue::new(
                HtmlValidationKind::ImagesWithoutAlt,
                canonical,
                "high",
                Vec::new(),
            ));
    }
}

fn attach_performance_subchecks(report: &mut AuditReport) {
    let Some(performance) = report.performance.as_ref() else {
        return;
    };
    let checks = [
        ("content_weight", performance.content_weight.is_some()),
        ("render_blocking", performance.render_blocking.is_some()),
        ("third_party", performance.third_party.is_some()),
        ("critical_chain", performance.critical_chain.is_some()),
        ("minification", performance.minification.is_some()),
        ("animations", performance.animations.is_some()),
        ("coverage", performance.coverage.is_some()),
    ];
    if let Some(run) = report
        .accessibility
        .execution
        .module_runs
        .iter_mut()
        .find(|run| run.module == "performance")
    {
        run.subchecks = checks
            .iter()
            .map(|(name, available)| crate::audit::SubcheckRun {
                subcheck: (*name).to_string(),
                status: if *available {
                    crate::audit::ExecutionStatus::Completed
                } else {
                    crate::audit::ExecutionStatus::Failed
                },
                reason_code: (!available).then(|| "subanalysis_unavailable".to_string()),
            })
            .collect();
        if checks.iter().any(|(_, available)| !available) {
            run.status = crate::audit::ExecutionStatus::Partial;
            run.reason_code = Some("one_or_more_subanalyses_unavailable".to_string());
        }
    }
}

fn audit_scope_from_config(config: &PipelineConfig) -> crate::audit::AuditScope {
    let catalog = AuditCatalog::standard();
    let mut requested_modules = vec!["accessibility".to_string()];
    if !matches!(config.interactive, crate::cli::InteractiveMode::Off) {
        requested_modules.push("accessibility_journey".to_string());
    }
    requested_modules.extend(
        catalog
            .enabled(config)
            .map(|module| module.id().to_string()),
    );
    requested_modules.sort();
    requested_modules.dedup();

    crate::audit::AuditScope {
        requested_modules,
        full_audit: config.full_audit,
        interactive_mode: format!("{:?}", config.interactive).to_lowercase(),
        journey_budget_ms: config.journey_budget_ms,
        throttling_profiles: if config.check_performance {
            ThrottleProfile::AUTO_PROFILES
                .iter()
                .map(|profile| profile.label().to_string())
                .collect()
        } else {
            Vec::new()
        },
        viewports: vec![
            crate::audit::ViewportDefinition {
                name: "desktop".to_string(),
                width: 1280,
                height: 800,
                device_scale_factor: 1.0,
            },
            crate::audit::ViewportDefinition {
                name: "mobile".to_string(),
                width: 390,
                height: 844,
                device_scale_factor: 2.0,
            },
        ],
        dismiss_consent: config.dismiss_consent,
        capture_screenshots: config.capture_screenshots,
        capture_element_evidence: config.capture_element_evidence,
        display_mode: config.display_mode.into(),
    }
}

pub(super) fn ensure_requested_module_runs(report: &mut AuditReport) {
    let outcomes = &report.accessibility.wcag_results.rule_outcomes;
    let failed = outcomes
        .iter()
        .filter(|outcome| crate::wcag::rule_run_errored(outcome))
        .count();
    let accessibility_status = if outcomes.is_empty() || failed == outcomes.len() {
        crate::audit::ExecutionStatus::Failed
    } else if failed > 0 {
        crate::audit::ExecutionStatus::Partial
    } else {
        crate::audit::ExecutionStatus::Completed
    };
    if !report
        .accessibility
        .execution
        .module_runs
        .iter()
        .any(|run| run.module == "accessibility")
    {
        report
            .accessibility
            .execution
            .module_runs
            .push(crate::audit::ModuleRun {
                module: "accessibility".to_string(),
                status: accessibility_status,
                reason_code: (accessibility_status != crate::audit::ExecutionStatus::Completed)
                    .then(|| "one_or_more_rule_checks_failed".to_string()),
                ..Default::default()
            });
    }

    let requested = report
        .accessibility
        .execution
        .scope
        .requested_modules
        .clone();
    for module in requested {
        if !report
            .accessibility
            .execution
            .module_runs
            .iter()
            .any(|run| run.module == module)
        {
            report
                .accessibility
                .execution
                .module_runs
                .push(crate::audit::ModuleRun {
                    module,
                    status: crate::audit::ExecutionStatus::Failed,
                    reason_code: Some("requested_module_has_no_run_record".to_string()),
                    ..Default::default()
                });
        }
    }
}

pub(super) fn consolidate_module_runs(
    runs: &[crate::audit::ModuleRun],
) -> Vec<crate::audit::ModuleRun> {
    use std::collections::BTreeMap;

    let mut grouped: BTreeMap<String, Vec<&crate::audit::ModuleRun>> = BTreeMap::new();
    for run in runs {
        grouped.entry(run.module.clone()).or_default().push(run);
    }

    grouped
        .into_iter()
        .map(|(module, items)| {
            let has_completed = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Completed);
            let has_failed = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Failed);
            let has_partial = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Partial);
            let status = if has_partial || (has_completed && has_failed) {
                crate::audit::ExecutionStatus::Partial
            } else if has_failed {
                crate::audit::ExecutionStatus::Failed
            } else if has_completed {
                crate::audit::ExecutionStatus::Completed
            } else if items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::NotApplicable)
            {
                crate::audit::ExecutionStatus::NotApplicable
            } else {
                crate::audit::ExecutionStatus::Skipped
            };
            let mut viewports: Vec<String> = items
                .iter()
                .flat_map(|run| run.viewports.iter().cloned())
                .collect();
            viewports.sort();
            viewports.dedup();
            crate::audit::ModuleRun {
                module,
                status,
                viewports,
                subchecks: items
                    .iter()
                    .flat_map(|run| run.subchecks.iter().cloned())
                    .collect(),
                reason_code: items.iter().find_map(|run| run.reason_code.clone()),
                message: items.iter().find_map(|run| run.message.clone()),
            }
        })
        .collect()
}

pub(super) fn update_audit_quality(report: &mut AuditReport) {
    let stability_budget_exhausted = report
        .accessibility
        .execution
        .navigation
        .stability
        .iter()
        .filter(|entry| {
            entry.status == crate::interaction::stability::StabilityStatus::BudgetExhausted
        })
        .count();
    let failed_rule_checks = report
        .accessibility
        .wcag_results
        .rule_outcomes
        .iter()
        .filter(|outcome| crate::wcag::rule_run_errored(outcome))
        .count();
    let partial_or_failed_modules = report
        .accessibility
        .execution
        .module_runs
        .iter()
        .filter(|run| {
            matches!(
                run.status,
                crate::audit::ExecutionStatus::Partial | crate::audit::ExecutionStatus::Failed
            )
        })
        .count();
    let status = if failed_rule_checks > 0 {
        crate::audit::AuditQualityStatus::Insufficient
    } else if partial_or_failed_modules > 0 || stability_budget_exhausted > 0 {
        crate::audit::AuditQualityStatus::Partial
    } else {
        crate::audit::AuditQualityStatus::Complete
    };
    let mut reasons = Vec::new();
    if failed_rule_checks > 0 {
        reasons.push(format!("failed_rule_checks:{failed_rule_checks}"));
    }
    if partial_or_failed_modules > 0 {
        reasons.push(format!(
            "partial_or_failed_modules:{partial_or_failed_modules}"
        ));
    }
    if stability_budget_exhausted > 0 {
        reasons.push(format!(
            "page_stability_budget_exhausted:{stability_budget_exhausted}"
        ));
    }
    report.accessibility.execution.quality = crate::audit::AuditQuality {
        status,
        qualified_results: status != crate::audit::AuditQualityStatus::Complete,
        failed_rule_checks,
        partial_or_failed_modules,
        reasons,
    };
}
