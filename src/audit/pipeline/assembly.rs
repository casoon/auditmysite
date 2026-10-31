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

/// `viewport_scores.weighted_overall`: the two per-viewport overalls blended
/// 70 % mobile, 30 % desktop. This is a viewport-pass figure, **not** a
/// published module weight: it is not the report's `overall_score` (which
/// weights modules, and adds Security afterwards) and is labelled so in
/// `docs/OUTPUT_CONTRACT.md` (#771).
pub(super) fn viewport_weighted_overall(mobile_overall: u32, desktop_overall: u32) -> u32 {
    (mobile_overall as f64 * 0.7 + desktop_overall as f64 * 0.3).round() as u32
}

/// Weighted overall of one viewport over the modules measured there, with
/// the weights `module_scores` publishes (`taxonomy::module_weight`), so the
/// shown weights are the formula: before plan 73 this path weighted SEO 20
/// instead of 15 and left HTML conformance out. Security joins after the
/// viewport blend with its own 10 %. `None` modules (not run, or no Web
/// Vitals measured) drop out and the rest is renormalized.
pub(super) fn compute_viewport_overall(acc: f32, modules: &[(&str, Option<u32>)]) -> u32 {
    let accessibility = crate::taxonomy::module_weight("Accessibility") as f64;
    let mut weighted = acc as f64 * accessibility;
    let mut total = accessibility;
    for (name, score) in modules {
        if let Some(score) = score {
            let weight = crate::taxonomy::module_weight(name) as f64;
            weighted += *score as f64 * weight;
            total += weight;
        }
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

/// A role one viewport needs is not redundant (#783): `role="list"` on a
/// list whose markers only one layout hides restores the semantics there. A
/// `lists/role-redundant` finding stands only where both passes report it for
/// the same element; the others leave both passes — before scoring and merge,
/// so score and report agree — and their run records count them out.
pub(super) fn drop_single_viewport_redundant_roles(
    desktop: &mut WcagResults,
    mobile: &mut WcagResults,
) {
    const RULE: &str = "lists/role-redundant";
    let keys = |r: &WcagResults| -> Vec<Option<String>> {
        r.violations
            .iter()
            .filter(|v| v.rule_id.as_deref() == Some(RULE))
            .map(|v| v.selector.clone())
            .collect()
    };
    let (desktop_keys, mobile_keys) = (keys(desktop), keys(mobile));
    let keep_paired = |results: &mut WcagResults, mut other: Vec<Option<String>>| {
        let before = results.violations.len();
        results.violations.retain(|v| {
            if v.rule_id.as_deref() != Some(RULE) {
                return true;
            }
            match other.iter().position(|k| *k == v.selector) {
                Some(i) => {
                    other.swap_remove(i);
                    true
                }
                None => false,
            }
        });
        let dropped = before - results.violations.len();
        for run in &mut results.rule_outcomes {
            if run.rule_id == RULE {
                run.findings = run.findings.saturating_sub(dropped);
            }
        }
    };
    keep_paired(desktop, mobile_keys);
    keep_paired(mobile, desktop_keys);
}

/// Merge violations from both passes.
///
/// Dedup key: (rule, selector), or (rule, role and name) for a finding
/// without selector. AX node ids are not stable across the two passes — the
/// page is reloaded in between — so they cannot pair a finding with itself
/// (#783: a tree-rule review per modal dialog was reported once per pass).
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
            .unwrap_or_else(|| {
                format!(
                    "{}|{}",
                    v.role.as_deref().unwrap_or(""),
                    v.name.as_deref().unwrap_or("")
                )
            });
        (v.rule_id.as_deref().unwrap_or(&v.rule), id)
    }

    fn merge_aux(desktop: &[Violation], mobile: &[Violation]) -> Vec<Violation> {
        let mut merged = Vec::new();
        let mut mobile_matched = vec![false; mobile.len()];
        for finding in mobile {
            let mut finding = finding.clone();
            finding.tags.push("mobile-only".to_string());
            merged.push(finding);
        }
        for finding in desktop {
            let paired = merged
                .iter()
                .take(mobile.len())
                .enumerate()
                .position(|(i, candidate)| {
                    !mobile_matched[i]
                        && dedup_key(candidate) == dedup_key(finding)
                        && candidate.message == finding.message
                        && candidate.kind == finding.kind
                });
            if let Some(index) = paired {
                mobile_matched[index] = true;
                let existing = &mut merged[index];
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
            // Preserve the more severe observation and its explanation.
            let mut shared = if desktop.violations[idx].severity > mv.severity {
                desktop.violations[idx].clone()
            } else {
                mv.clone()
            };
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

    // Review/positive/manual results obey the same one-to-one conservation
    // as violations; identical wording is not an element or rule identity.
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
    //
    // Same element means the same backend node when both findings carry one:
    // the shared rules build short selectors (`button`), so two different
    // buttons share a key, and a fail on one swallowed the review on the
    // other (`label-in-name/mismatch`, #692). Without a backend id on either
    // side the selector decides, as before.
    let mut violation_keys: std::collections::HashMap<
        (String, Option<String>, String),
        Vec<Option<i64>>,
    > = std::collections::HashMap::new();
    for v in &merged {
        let (rule, id) = dedup_key(v);
        violation_keys
            .entry((rule.to_owned(), v.rule_id.clone(), id))
            .or_default()
            .push(v.backend_node_id);
    }
    let warnings: Vec<Violation> = warnings
        .into_iter()
        .filter(|w| {
            let (rule, id) = dedup_key(w);
            let Some(backends) = violation_keys.get(&(rule.to_owned(), w.rule_id.clone(), id))
            else {
                return true;
            };
            !backends.iter().any(|b| match (b, w.backend_node_id) {
                (Some(a), Some(c)) => *a == c,
                _ => true,
            })
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
    report.run_context = crate::audit::RunContext {
        indexability: config.indexability,
        decision_profile: config.decision_profile.clone(),
    };
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

    // Whether the hardening probes ran, answered or did not apply is recorded
    // by the `tech_stack.hardening_probes` collection (#777).
    if let Some(tech_stack) = snapshot.tech_stack.clone() {
        report = report.with_tech_stack(tech_stack);
    }

    // The console and library collections are recorded under the collection
    // contract (#777); their records already sit in the module run.
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

pub(super) fn audit_scope_from_config(config: &PipelineConfig) -> crate::audit::AuditScope {
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

    let mut requested_subchecks = Vec::new();
    if config.check_isolate_third_party_impact {
        requested_subchecks.push(crate::performance::third_party::ISOLATION.id.to_string());
    }
    if config.check_ssr_content {
        requested_subchecks.push(crate::content_visibility::ssr_gap::SSR_GAP.id.to_string());
    }

    crate::audit::AuditScope {
        requested_modules,
        requested_subchecks,
        requested_rules: crate::wcag::expectation::expected_rules(
            config.wcag_level,
            &config.rule_filter,
            config.check_html_conform,
        )
        .into_iter()
        .map(|rule| crate::audit::RequestedRule {
            rule_id: rule.rule_id,
            viewports: rule.viewports.iter().map(|v| v.to_string()).collect(),
        })
        .collect(),
        frame_rules: crate::audit::frames::expected_frame_rules(
            config.wcag_level,
            &config.rule_filter,
        ),
        rule_filter: {
            let record = crate::audit::RuleFilterRecord {
                disabled_rules: config.rule_filter.disabled_rules.clone(),
                enabled_only_rules: config.rule_filter.enabled_only_rules.clone(),
            };
            (!record.is_empty()).then_some(record)
        },
        wcag_level: config.wcag_level.to_string(),
        full_audit: config.full_audit,
        measurement_profile: crate::audit::MeasurementProfile {
            version: crate::audit::MEASUREMENT_PROFILE_VERSION,
            session_policy: config.session.as_str().to_string(),
            read_only: crate::a11y_journey::state_changing_journeys(
                config.interactive,
                config.allow_state_changes,
            )
            .is_empty(),
            state_changing_journeys: crate::a11y_journey::state_changing_journeys(
                config.interactive,
                config.allow_state_changes,
            )
            .iter()
            .map(|j| j.to_string())
            .collect(),
            state_id: config.state_id.clone(),
            extra_checks: crate::audit::support_matrix::ExtraCheckRequest {
                isolate_third_party_impact: config.check_isolate_third_party_impact,
                check_ssr_content: config.check_ssr_content,
                ai_transparency: config.check_ai_transparency,
                dns_check: config.check_dns,
                ..Default::default()
            }
            .requested_flags(),
            storage_state: config.storage_state.as_ref().map(|state| {
                crate::audit::StorageStateRef {
                    sha256: state.digest.clone(),
                    cookies: state.cookie_count(),
                    origins: state.origin_count(),
                }
            }),
        },
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
                subchecks: merge_viewport_subchecks(
                    items.iter().flat_map(|run| run.subchecks.iter()),
                ),
                reason_code: items.iter().find_map(|run| run.reason_code.clone()),
                message: items.iter().find_map(|run| run.message.clone()),
            }
        })
        .collect()
}

/// One entry per subcheck across viewports. When the viewports disagree, the
/// one that lost its measurement wins: a result measured on desktop only is
/// not a complete measurement of the page.
fn merge_viewport_subchecks<'a>(
    subchecks: impl Iterator<Item = &'a crate::audit::SubcheckRun>,
) -> Vec<crate::audit::SubcheckRun> {
    let mut merged: Vec<crate::audit::SubcheckRun> = Vec::new();
    for subcheck in subchecks {
        let lost = |s: &crate::audit::SubcheckRun| {
            s.state.map_or(
                matches!(
                    s.status,
                    crate::audit::ExecutionStatus::Failed | crate::audit::ExecutionStatus::Partial
                ),
                |state| state.is_gap(),
            )
        };
        match merged.iter_mut().find(|m| m.subcheck == subcheck.subcheck) {
            Some(existing) if !lost(existing) && lost(subcheck) => *existing = subcheck.clone(),
            Some(_) => {}
            None => merged.push(subcheck.clone()),
        }
    }
    merged
}

pub(super) fn update_audit_quality(report: &mut AuditReport) {
    let gaps = crate::audit::validation::capture_gaps(&report.accessibility.execution);
    let stability_budget_exhausted = gaps.exhausted_stability_budgets;
    // A page-rule executor and its concrete rules are one check (#778).
    let failed_rule_checks = crate::wcag::expectation::count_checks(
        &report.accessibility.wcag_results.rule_outcomes,
        crate::wcag::rule_run_errored,
    );
    let partial_or_failed_modules = report
        .accessibility
        .execution
        .module_runs
        .iter()
        .filter(|run| {
            // The accessibility run is partial *because of* the errored rule
            // checks counted above — one fact, one counter (#778).
            run.reason_code.as_deref() != Some("one_or_more_rule_checks_failed")
                && matches!(
                    run.status,
                    crate::audit::ExecutionStatus::Partial | crate::audit::ExecutionStatus::Failed
                )
        })
        .count();
    let missing_capabilities = crate::wcag::expectation::count_checks(
        &report.accessibility.wcag_results.rule_outcomes,
        |run| run.not_run == Some(crate::wcag::NotRun::CapabilityMissing),
    );
    let status = if failed_rule_checks > 0 {
        crate::audit::AuditQualityStatus::Insufficient
    } else if partial_or_failed_modules > 0
        || stability_budget_exhausted > 0
        || missing_capabilities > 0
        || gaps.stability_fallbacks > 0
        || gaps.uninspected_frames > 0
    {
        crate::audit::AuditQualityStatus::Partial
    } else {
        crate::audit::AuditQualityStatus::Complete
    };
    let mut reasons = Vec::new();
    if gaps.stability_fallbacks > 0 {
        reasons.push(format!(
            "page_stability_fallbacks:{}",
            gaps.stability_fallbacks
        ));
    }
    if gaps.uninspected_frames > 0 {
        reasons.push(format!("uninspected_frames:{}", gaps.uninspected_frames));
    }
    if missing_capabilities > 0 {
        reasons.push(format!("missing_rule_capabilities:{missing_capabilities}"));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{ExecutionStatus, ModuleRun};
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    #[test]
    fn capture_gaps_qualify_audit_without_penalizing_intentional_exclusions() {
        use crate::audit::frames::{FrameCoverage, FrameEntry, FrameSkipReason};
        for (reason, expected) in [
            (
                FrameSkipReason::CrossOrigin,
                crate::audit::AuditQualityStatus::Partial,
            ),
            (
                FrameSkipReason::AxTreeFailed,
                crate::audit::AuditQualityStatus::Partial,
            ),
            (
                FrameSkipReason::Hidden,
                crate::audit::AuditQualityStatus::Complete,
            ),
            (
                FrameSkipReason::Excluded,
                crate::audit::AuditQualityStatus::Complete,
            ),
        ] {
            let mut report = report_with_run("probe");
            report.accessibility.execution.frames = Some(FrameCoverage {
                skipped: vec![FrameEntry {
                    reason: Some(reason),
                    ..Default::default()
                }],
                ..Default::default()
            });
            update_audit_quality(&mut report);
            assert_eq!(
                report.accessibility.execution.quality.status, expected,
                "{reason:?}"
            );
            crate::audit::normalize_checked(&report).unwrap();
        }
    }

    fn quality_of(report: &mut AuditReport) -> crate::audit::AuditQualityStatus {
        update_audit_quality(report);
        report.accessibility.execution.quality.status
    }

    #[test]
    fn execution_state_matrix_qualifies_the_audit() {
        use crate::audit::AuditQualityStatus::{Complete, Insufficient, Partial};
        use crate::interaction::stability::{StabilityProvenance, StabilityStatus};
        use crate::wcag::{NotRun, RuleRun};

        let with_rule = |run: RuleRun| {
            let mut report = report_with_run("accessibility");
            report.accessibility.wcag_results.rule_outcomes.push(run);
            // Boxed: nine reports by value are ~2 MB of stack, the whole
            // default test-thread stack.
            Box::new(report)
        };
        let with_stability = |status| {
            let mut report = report_with_run("accessibility");
            report
                .accessibility
                .execution
                .navigation
                .stability
                .push(StabilityProvenance {
                    viewport: "desktop".into(),
                    status,
                    ..Default::default()
                });
            Box::new(report)
        };
        let cases = [
            ("ran", with_rule(RuleRun::ran("r", 0)), Complete),
            (
                "not applicable",
                with_rule(RuleRun::not_run("r", NotRun::NotApplicable)),
                Complete,
            ),
            (
                "disabled by configuration",
                with_rule(RuleRun::not_run("r", NotRun::Disabled)),
                Complete,
            ),
            (
                "capability missing",
                with_rule(RuleRun::not_run("r", NotRun::CapabilityMissing)),
                Partial,
            ),
            (
                "errored",
                with_rule(RuleRun::not_run("r", NotRun::Errored)),
                Insufficient,
            ),
            ("stable", with_stability(StabilityStatus::Stable), Complete),
            (
                "ongoing animation",
                with_stability(StabilityStatus::OngoingAnimation),
                Complete,
            ),
            (
                "fallback",
                with_stability(StabilityStatus::Fallback),
                Partial,
            ),
            (
                "budget exhausted",
                with_stability(StabilityStatus::BudgetExhausted),
                Partial,
            ),
        ];
        for (name, mut report, expected) in cases {
            assert_eq!(quality_of(&mut report), expected, "{name}");
            crate::audit::normalize_checked(&report)
                .unwrap_or_else(|e| panic!("{name} must stay valid data: {e}"));
        }
    }

    #[test]
    fn measurement_profile_must_agree_with_the_interactive_depth() {
        use clap::Parser;
        let config_scope = |interactive: crate::cli::InteractiveMode, allow: bool| {
            let mut config = crate::audit::PipelineConfig::from_args_and_config(
                &crate::cli::Args::parse_from(["auditmysite", "https://example.com"]),
                None,
            );
            config.interactive = interactive;
            config.allow_state_changes = allow;
            audit_scope_from_config(&config)
        };
        // `full` alone only observes; the clicks are an explicit opt-in.
        let observing = config_scope(crate::cli::InteractiveMode::Full, false);
        assert!(observing.measurement_profile.read_only);
        let full = config_scope(crate::cli::InteractiveMode::Full, true);
        assert!(!full.measurement_profile.read_only);
        assert!(!full.measurement_profile.state_changing_journeys.is_empty());
        assert_eq!(full.measurement_profile.session_policy, "isolated");
        let basic = config_scope(crate::cli::InteractiveMode::Basic, true);
        assert!(basic.measurement_profile.read_only);

        // A read-only mode claiming a state-changing journey is rejected.
        let mut report = report_with_run("accessibility");
        report.accessibility.execution.scope = basic;
        report
            .accessibility
            .execution
            .scope
            .measurement_profile
            .state_changing_journeys = vec!["add_to_cart".into()];
        update_audit_quality(&mut report);
        let Err(err) = crate::audit::normalize_checked(&report) else {
            panic!("contradiction must be rejected");
        };
        assert!(err
            .violations
            .iter()
            .any(|v| v.code == "measurement_profile_read_only"
                || v.code == "measurement_profile_journeys"));
    }

    #[test]
    fn declared_state_enters_the_profile_by_identity_not_by_value() {
        use clap::Parser;
        let mut config = crate::audit::PipelineConfig::from_args_and_config(
            &crate::cli::Args::parse_from(["auditmysite", "https://example.com"]),
            None,
        );
        config.state_id = Some("logged-in".into());
        config.storage_state = Some(
            crate::browser::StorageState::from_bytes(
                br#"{"cookies":[{"name":"sid","value":"TOPSECRET"}],"origins":[]}"#,
            )
            .unwrap(),
        );
        let scope = audit_scope_from_config(&config);
        let json = serde_json::to_string(&scope).unwrap();
        assert!(json.contains("logged-in") && json.contains("\"cookies\":1"));
        assert!(!json.contains("TOPSECRET"), "{json}");
        let without = audit_scope_from_config(&crate::audit::PipelineConfig {
            storage_state: None,
            state_id: None,
            ..config.clone()
        });
        assert_ne!(
            serde_json::to_string(&without).unwrap(),
            json,
            "another state is another measurement"
        );
        assert_ne!(
            config.audit_signature(),
            crate::audit::PipelineConfig {
                storage_state: None,
                ..config.clone()
            }
            .audit_signature()
        );
    }

    #[test]
    fn an_errored_rule_is_counted_once() {
        let mut report = report_with_run("probe");
        report
            .accessibility
            .wcag_results
            .rule_outcomes
            .push(crate::wcag::RuleRun::not_run(
                "r",
                crate::wcag::NotRun::Errored,
            ));
        ensure_requested_module_runs(&mut report);
        update_audit_quality(&mut report);
        let quality = &report.accessibility.execution.quality;
        assert_eq!(quality.failed_rule_checks, 1);
        assert_eq!(quality.partial_or_failed_modules, 0);
    }

    #[test]
    fn a_complete_claim_over_incomplete_execution_is_rejected() {
        use crate::wcag::{NotRun, RuleRun};
        for run in [
            RuleRun::not_run("r", NotRun::Errored),
            RuleRun::not_run("r", NotRun::CapabilityMissing),
        ] {
            let mut report = report_with_run("accessibility");
            report.accessibility.wcag_results.rule_outcomes.push(run);
            update_audit_quality(&mut report);
            // Tamper: claim completeness the execution data contradicts.
            report.accessibility.execution.quality.status =
                crate::audit::AuditQualityStatus::Complete;
            report.accessibility.execution.quality.qualified_results = false;
            let Err(err) = crate::audit::normalize_checked(&report) else {
                panic!("contradiction must be rejected");
            };
            assert!(err
                .violations
                .iter()
                .any(|v| v.code == "execution_completeness"));
        }
    }

    /// A report whose scope was derived from the registrations (default
    /// config) and whose outcomes are exactly the expected ones, each ran.
    fn report_with_expected_outcomes() -> AuditReport {
        // Building the config takes several hundred KB of stack in a debug
        // build; under coverage instrumentation that overflows a test
        // thread (cargo-tarpaulin segfaulted), so it gets its own.
        std::thread::Builder::new()
            .stack_size(32 << 20)
            .spawn(build_report_with_expected_outcomes)
            .unwrap()
            .join()
            .unwrap()
    }

    fn build_report_with_expected_outcomes() -> AuditReport {
        use clap::Parser;
        let config = crate::audit::PipelineConfig::from_args_and_config(
            &crate::cli::Args::parse_from(["auditmysite", "https://example.com"]),
            None,
        );
        let mut report = report_with_run("accessibility");
        report.accessibility.execution.scope = audit_scope_from_config(&config);
        report
            .accessibility
            .execution
            .scope
            .requested_modules
            .retain(|m| m == "accessibility");
        for rule in &report.accessibility.execution.scope.requested_rules.clone() {
            // Shared rules above the level report as disabled, as in a run.
            let above = crate::wcag::expectation::registered_min_level(&rule.rule_id)
                .is_some_and(|min| min > config.wcag_level);
            for viewport in &rule.viewports {
                let run = if above {
                    crate::wcag::RuleRun::not_run(&rule.rule_id, crate::wcag::NotRun::Disabled)
                } else {
                    crate::wcag::RuleRun::ran(&rule.rule_id, 0)
                };
                report
                    .accessibility
                    .wcag_results
                    .rule_outcomes
                    .push(run.in_viewport(viewport));
            }
        }
        update_audit_quality(&mut report);
        report
    }

    fn rejected_codes(report: &AuditReport) -> Vec<&'static str> {
        match crate::audit::normalize_checked(report) {
            Ok(_) => Vec::new(),
            Err(err) => err.violations.iter().map(|v| v.code).collect(),
        }
    }

    #[test]
    fn the_expected_scope_is_derived_from_the_registrations_and_accepted() {
        let report = report_with_expected_outcomes();
        let scope = &report.accessibility.execution.scope;
        assert_eq!(scope.wcag_level, "AA");
        // Tree, page and inline rules are in the scope, not only shared ones.
        for id in ["image-alt", "focus-visible", "css-overflow-hidden"] {
            assert!(
                scope.requested_rules.iter().any(|r| r.rule_id == id),
                "{id}"
            );
        }
        assert!(rejected_codes(&report).is_empty());
    }

    #[test]
    fn a_requested_rule_without_outcome_in_one_viewport_is_a_gap() {
        let mut report = report_with_expected_outcomes();
        // Every viewport is judged separately: losing the mobile outcome of
        // a rule that has a desktop one is still a gap.
        let outcomes = &mut report.accessibility.wcag_results.rule_outcomes;
        let at = outcomes
            .iter()
            .position(|o| o.rule_id == "image-alt" && o.viewport.as_deref() == Some("mobile"))
            .unwrap();
        outcomes.remove(at);
        assert_eq!(rejected_codes(&report), vec!["requested_rule_outcome"]);
        // Same for an inline rule that only the mobile pass owns.
        let mut report = report_with_expected_outcomes();
        report
            .accessibility
            .wcag_results
            .rule_outcomes
            .retain(|o| o.rule_id != "css-overflow-hidden");
        assert_eq!(rejected_codes(&report), vec!["requested_rule_outcome"]);
        // A page-rule executor without its row is a gap too.
        let mut report = report_with_expected_outcomes();
        let executor = crate::wcag::rules::PAGE_RULES[0].rule_id;
        report
            .accessibility
            .wcag_results
            .rule_outcomes
            .retain(|o| o.rule_id != executor);
        assert_eq!(rejected_codes(&report), vec!["requested_rule_outcome"; 2]);
    }

    #[test]
    fn a_doubled_outcome_is_rejected() {
        let mut report = report_with_expected_outcomes();
        let copy = report.accessibility.wcag_results.rule_outcomes[0].clone();
        report.accessibility.wcag_results.rule_outcomes.push(copy);
        assert_eq!(rejected_codes(&report), vec!["rule_outcome_duplicate"]);
    }

    #[test]
    fn a_rule_that_ran_above_the_level_is_rejected() {
        let mut report = report_with_expected_outcomes();
        report.accessibility.execution.scope.wcag_level = "A".into();
        // `focus-visible` is AA: at level A it must not have run. The
        // outcomes were built for AA, so it is exactly this one (and the
        // other AA rules) that contradict the level.
        let codes = rejected_codes(&report);
        assert!(!codes.is_empty());
        assert!(codes.iter().all(|c| *c == "rule_outcome_above_level"));
        // Not run with a reason is the honest outcome for it.
        for run in &mut report.accessibility.wcag_results.rule_outcomes {
            if crate::wcag::expectation::registered_min_level(&run.rule_id)
                .is_some_and(|min| min > crate::cli::WcagLevel::A)
            {
                *run = crate::wcag::RuleRun::not_run(&run.rule_id, crate::wcag::NotRun::Disabled)
                    .in_viewport(run.viewport.clone().unwrap());
            }
        }
        assert!(rejected_codes(&report).is_empty());
    }

    /// The report with one audited iframe that accounts for every frame rule
    /// in both viewports, as the frame pass leaves it.
    fn report_with_audited_frame() -> AuditReport {
        use crate::audit::frames::{FrameCoverage, FrameEntry};
        let mut report = report_with_expected_outcomes();
        let scope = &report.accessibility.execution.scope;
        assert!(!scope.frame_rules.is_empty(), "the scope names frame rules");
        let level = crate::cli::WcagLevel::AA;
        let outcomes = scope
            .frame_rules
            .iter()
            .flat_map(|rule_id| {
                ["desktop", "mobile"].map(|viewport| {
                    let above = crate::wcag::expectation::registered_min_level(rule_id)
                        .is_some_and(|min| min > level);
                    let run = if above {
                        crate::wcag::RuleRun::not_run(rule_id, crate::wcag::NotRun::Disabled)
                    } else {
                        crate::wcag::RuleRun::ran(rule_id, 0)
                    };
                    run.in_viewport(viewport)
                })
            })
            .collect();
        report.accessibility.execution.frames = Some(FrameCoverage {
            audited: vec![FrameEntry {
                selector: "iframe#w".into(),
                url: Some("https://example.org/w".into()),
                reason: None,
                viewports: vec!["desktop".into(), "mobile".into()],
                outcomes,
            }],
            skipped: vec![],
        });
        report
    }

    fn frame_outcomes(report: &mut AuditReport) -> &mut Vec<crate::wcag::RuleRun> {
        &mut report
            .accessibility
            .execution
            .frames
            .as_mut()
            .unwrap()
            .audited[0]
            .outcomes
    }

    #[test]
    fn an_audited_frame_with_one_outcome_per_rule_and_viewport_is_accepted() {
        assert!(rejected_codes(&report_with_audited_frame()).is_empty());
    }

    #[test]
    fn a_frame_rule_without_outcome_in_one_viewport_is_a_gap() {
        let mut report = report_with_audited_frame();
        let outcomes = frame_outcomes(&mut report);
        let at = outcomes
            .iter()
            .position(|o| o.rule_id == "image-alt" && o.viewport.as_deref() == Some("mobile"))
            .unwrap();
        outcomes.remove(at);
        assert_eq!(rejected_codes(&report), vec!["frame_rule_outcome_missing"]);
    }

    #[test]
    fn a_doubled_frame_outcome_is_rejected() {
        let mut report = report_with_audited_frame();
        let copy = frame_outcomes(&mut report)[0].clone();
        frame_outcomes(&mut report).push(copy);
        assert_eq!(
            rejected_codes(&report),
            vec!["frame_rule_outcome_duplicate"]
        );
    }

    #[test]
    fn a_frame_outcome_for_a_rule_outside_the_frame_scope_is_rejected() {
        let mut report = report_with_audited_frame();
        frame_outcomes(&mut report)
            .push(crate::wcag::RuleRun::ran("landmarks/main-missing", 0).in_viewport("desktop"));
        assert_eq!(
            rejected_codes(&report),
            vec!["frame_rule_outcome_unexpected"]
        );
    }

    #[test]
    fn a_frame_rule_that_ran_above_the_level_is_rejected() {
        let mut report = report_with_audited_frame();
        report.accessibility.execution.scope.wcag_level = "A".into();
        let above: Vec<String> = report
            .accessibility
            .execution
            .scope
            .frame_rules
            .iter()
            .filter(|id| {
                crate::wcag::expectation::registered_min_level(id)
                    .is_some_and(|min| min > crate::cli::WcagLevel::A)
            })
            .cloned()
            .collect();
        assert!(!above.is_empty(), "an AA frame rule exists");
        for run in frame_outcomes(&mut report) {
            if above.contains(&run.rule_id) {
                *run = crate::wcag::RuleRun::ran(&run.rule_id, 0)
                    .in_viewport(run.viewport.clone().unwrap());
            }
        }
        let codes = rejected_codes(&report);
        assert!(
            codes.contains(&"frame_rule_outcome_above_level"),
            "{codes:?}"
        );
        // Reported as not run, the frame is honest again.
        for run in frame_outcomes(&mut report) {
            if above.contains(&run.rule_id) {
                *run = crate::wcag::RuleRun::not_run(&run.rule_id, crate::wcag::NotRun::Disabled)
                    .in_viewport(run.viewport.clone().unwrap());
            }
        }
        report
            .accessibility
            .wcag_results
            .rule_outcomes
            .iter_mut()
            .for_each(|run| {
                if crate::wcag::expectation::registered_min_level(&run.rule_id)
                    .is_some_and(|min| min > crate::cli::WcagLevel::A)
                {
                    *run =
                        crate::wcag::RuleRun::not_run(&run.rule_id, crate::wcag::NotRun::Disabled)
                            .in_viewport(run.viewport.clone().unwrap());
                }
            });
        assert!(rejected_codes(&report).is_empty());
    }

    #[test]
    fn a_skipped_frame_owes_no_outcomes() {
        use crate::audit::frames::FrameSkipReason;
        let mut report = report_with_audited_frame();
        let coverage = report.accessibility.execution.frames.as_mut().unwrap();
        let mut frame = coverage.audited.remove(0);
        frame.outcomes.clear();
        // A hidden frame is out of scope, not a coverage gap.
        frame.reason = Some(FrameSkipReason::Hidden);
        coverage.skipped.push(frame);
        assert!(rejected_codes(&report).is_empty());
    }

    #[test]
    fn a_tree_rule_that_ran_against_the_recorded_filter_is_rejected() {
        let mut report = report_with_expected_outcomes();
        report.accessibility.execution.scope.rule_filter = Some(crate::audit::RuleFilterRecord {
            disabled_rules: vec!["image-alt".into()],
            enabled_only_rules: vec![],
        });
        // The filter says image-alt did not run; the outcomes say it did.
        let codes = rejected_codes(&report);
        assert_eq!(codes, vec!["rule_outcome_filtered"; 2], "both viewports");
        // The honest record: filtered out, so neither requested nor run.
        report
            .accessibility
            .execution
            .scope
            .requested_rules
            .retain(|r| r.rule_id != "image-alt");
        report
            .accessibility
            .wcag_results
            .rule_outcomes
            .retain(|o| o.rule_id != "image-alt");
        assert!(rejected_codes(&report).is_empty());
    }

    #[test]
    fn the_scope_records_the_rule_filter_and_the_frame_rules_it_leaves() {
        use clap::Parser;
        let mut config = crate::audit::PipelineConfig::from_args_and_config(
            &crate::cli::Args::parse_from(["auditmysite", "https://example.com"]),
            None,
        );
        assert!(audit_scope_from_config(&config).rule_filter.is_none());
        config.rule_filter.disabled_rules = vec!["image-alt".into()];
        let scope = audit_scope_from_config(&config);
        assert_eq!(
            scope.rule_filter.unwrap().disabled_rules,
            vec!["image-alt".to_string()]
        );
        assert!(!scope.frame_rules.iter().any(|id| id == "image-alt"));
        assert!(scope.frame_rules.iter().any(|id| id == "keyboard-trap"));
    }

    #[test]
    fn a_failed_page_rule_is_one_failed_check_not_one_per_concrete_rule() {
        let mut report = report_with_expected_outcomes();
        let executor = crate::wcag::rules::PAGE_RULES
            .iter()
            .find(|r| r.outcome_ids.len() > 1)
            .unwrap();
        for run in &mut report.accessibility.wcag_results.rule_outcomes {
            let owned = run.rule_id == executor.rule_id
                || executor.outcome_ids.contains(&run.rule_id.as_str());
            if owned && run.viewport.as_deref() == Some("mobile") {
                run.not_run = Some(crate::wcag::NotRun::Errored);
            }
        }
        update_audit_quality(&mut report);
        assert_eq!(report.accessibility.execution.quality.failed_rule_checks, 1);
        assert!(rejected_codes(&report).is_empty());
    }

    #[test]
    fn foundation_merge_preserves_concrete_rule_identity() {
        let finding = |rule: &str, node: i64| {
            let mut v = Violation::new(
                "4.1.2",
                "Name Role Value",
                WcagLevel::A,
                crate::wcag::Severity::High,
                rule,
                node.to_string(),
            )
            .with_rule_id(rule)
            .with_selector("button");
            v.backend_node_id = Some(node);
            v
        };
        for (desktop_rule, desktop_node, mobile_rule, mobile_node) in [
            ("aria/role-invalid", 1, "aria/attribute-value-invalid", 1),
            ("aria/role-invalid", 1, "aria/attribute-value-invalid", 2),
        ] {
            let mut desktop = WcagResults::new();
            desktop.violations.push(finding(desktop_rule, desktop_node));
            let mut mobile = WcagResults::new();
            mobile.violations.push(finding(mobile_rule, mobile_node));
            let merged = merge_wcag_violations(&desktop, &mobile);
            assert_eq!(merged.violations.len(), 2);
            assert!(!merged
                .violations
                .iter()
                .any(|v| v.tags.iter().any(|t| t == "both-viewports")));
        }
    }

    #[test]
    fn list_role_needed_in_one_viewport_is_not_redundant() {
        let redundant = |selector: &str| {
            Violation::new(
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                crate::wcag::Severity::Low,
                "role=\"list\" repeats the implicit role",
                "x",
            )
            .with_rule_id("lists/role-redundant")
            .with_selector(selector)
        };
        let mut desktop = WcagResults::new();
        desktop.violations.push(redundant("ul#cards"));
        desktop.violations.push(redundant("ul#both"));
        desktop
            .rule_outcomes
            .push(crate::wcag::RuleRun::ran("lists/role-redundant", 2).in_viewport("desktop"));
        let mut mobile = WcagResults::new();
        mobile.violations.push(redundant("ul#both"));
        mobile
            .rule_outcomes
            .push(crate::wcag::RuleRun::ran("lists/role-redundant", 1).in_viewport("mobile"));
        drop_single_viewport_redundant_roles(&mut desktop, &mut mobile);
        assert_eq!(desktop.violations.len(), 1);
        assert_eq!(desktop.violations[0].selector.as_deref(), Some("ul#both"));
        assert_eq!(desktop.rule_outcomes[0].findings, 1);
        assert_eq!(mobile.violations.len(), 1);
        assert_eq!(mobile.rule_outcomes[0].findings, 1);
    }

    #[test]
    fn review_without_selector_pairs_across_passes_despite_new_node_ids() {
        let review = |node: &str| {
            let mut v = Violation::new(
                "2.1.2",
                "No Keyboard Trap",
                WcagLevel::A,
                crate::wcag::Severity::Medium,
                "Verify focus can leave this dialog",
                node,
            )
            .with_rule_id("keyboard-trap")
            .as_warning();
            v.role = Some("dialog".into());
            v.name = Some("Settings".into());
            v
        };
        let mut desktop = WcagResults::new();
        desktop.warnings.push(review("41"));
        let mut mobile = WcagResults::new();
        mobile.warnings.push(review("97"));
        let merged = merge_wcag_violations(&desktop, &mobile);
        assert_eq!(merged.warnings.len(), 1, "one dialog, one review");
        assert!(merged.warnings[0]
            .tags
            .iter()
            .any(|t| t == "both-viewports"));
    }

    #[test]
    fn foundation_aux_merge_preserves_rules_locations_and_multiplicity() {
        let finding = |id: &str, selector: &str| {
            Violation::new(
                "2.4.1",
                "Review",
                WcagLevel::A,
                crate::wcag::Severity::Low,
                "same message",
                "node",
            )
            .with_rule_id(id)
            .with_selector(selector)
        };
        for (desktop, mobile, expected) in [
            (vec![finding("one", "#a"), finding("one", "#b")], vec![], 2),
            (vec![finding("one", "#a")], vec![finding("two", "#a")], 2),
            (
                vec![finding("one", "button"), finding("one", "button")],
                vec![finding("one", "button")],
                2,
            ),
            (vec![finding("one", "#a")], vec![finding("one", "#a")], 1),
        ] {
            for list in ["warnings", "positives", "manual"] {
                let mut d = WcagResults::new();
                let mut m = WcagResults::new();
                match list {
                    "warnings" => {
                        d.warnings = desktop.clone();
                        m.warnings = mobile.clone();
                    }
                    "positives" => {
                        d.positives = desktop.clone();
                        m.positives = mobile.clone();
                    }
                    _ => {
                        d.not_testables = desktop.clone();
                        m.not_testables = mobile.clone();
                    }
                }
                let merged = merge_wcag_violations(&d, &m);
                assert_eq!(
                    merged.warnings.len() + merged.positives.len() + merged.not_testables.len(),
                    expected,
                    "{list}"
                );
            }
        }
    }

    #[test]
    fn foundation_merge_never_downgrades_the_stronger_viewport() {
        use crate::wcag::Severity;
        for desktop_severity in [
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ] {
            for mobile_severity in [
                Severity::Low,
                Severity::Medium,
                Severity::High,
                Severity::Critical,
            ] {
                let finding = |severity| {
                    Violation::new(
                        "4.1.2",
                        "Name",
                        WcagLevel::A,
                        severity,
                        severity.as_str(),
                        "node",
                    )
                    .with_rule_id("aria/role-invalid")
                    .with_selector("#control")
                };
                let mut desktop = WcagResults::new();
                desktop.violations.push(finding(desktop_severity));
                let mut mobile = WcagResults::new();
                mobile.violations.push(finding(mobile_severity));
                let merged = merge_wcag_violations(&desktop, &mobile);
                let expected = desktop_severity.max(mobile_severity);
                assert_eq!(merged.violations[0].severity, expected);
                assert_eq!(merged.violations[0].message, expected.as_str());
            }
        }
    }

    fn report_with_run(module: &str) -> AuditReport {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        );
        report.accessibility.execution.module_runs.push(ModuleRun {
            module: module.to_string(),
            status: ExecutionStatus::Completed,
            ..Default::default()
        });
        report
    }
}
