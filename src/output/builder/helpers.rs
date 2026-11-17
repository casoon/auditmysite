//! Miscellaneous helper utilities used across builder submodules.

use crate::audit::{BatchReport, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    AffectedElement, AppendixViolation, BatchAppendixData, UrlAppendix,
};
use crate::util::truncate_url;

pub(super) fn build_overall_impact(
    locale: &str,
    normalized: &NormalizedReport,
) -> Vec<(String, String)> {
    if let Some(ref interp) = normalized.interpretation {
        interp
            .overall_impact
            .iter()
            .map(|(label, value)| {
                (
                    label.for_locale(locale).to_string(),
                    value.for_locale(locale).to_string(),
                )
            })
            .collect()
    } else {
        vec![]
    }
}

pub(super) fn build_benchmark_context(locale: &str, normalized: &NormalizedReport) -> String {
    normalized
        .interpretation
        .as_ref()
        .map(|i| i.benchmark_context.for_locale(locale).to_string())
        .unwrap_or_default()
}

pub(super) fn build_business_consequence(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let key = normalized
        .interpretation
        .as_ref()
        .map(|i| i.business_consequence_key.as_str())
        .unwrap_or("business-consequence-default");
    i18n.t(key)
}

pub(super) fn build_consequence_text(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let key = normalized
        .interpretation
        .as_ref()
        .map(|i| i.consequence_key.as_str())
        .unwrap_or("");
    if key.is_empty() {
        String::new()
    } else {
        i18n.t(key)
    }
}

pub(super) fn localized_report_title(locale: &str) -> String {
    crate::i18n::text("output-report-title", locale == "en")
}

pub(super) fn localized_report_subtitle(locale: &str) -> String {
    crate::i18n::text("output-report-subtitle", locale == "en")
}

pub(super) fn build_verdict_text(
    i18n: &I18n,
    url: &str,
    score: f32,
    normalized: &NormalizedReport,
) -> String {
    let key = normalized
        .interpretation
        .as_ref()
        .map(|i| i.verdict_key.as_str())
        .unwrap_or("verdict-tier-solid");
    i18n.t_args(
        key,
        &[("url", url.to_string()), ("score", format!("{:.0}", score))],
    )
}

pub(super) fn build_score_note(i18n: &I18n, normalized: &NormalizedReport) -> Option<String> {
    normalized
        .interpretation
        .as_ref()
        .and_then(|i| i.score_note_key.as_deref())
        .map(|key| i18n.t(key))
}

pub(super) fn build_batch_verdict(i18n: &I18n, batch: &crate::audit::BatchReport) -> String {
    // Nothing measured: no band and no headline score, not a verdict on 0.
    let Some(average) = batch.summary.average_score else {
        return String::new();
    };
    let key = &batch.summary.score_band_key;
    // The accessibility average, matching the batch cover's headline.
    let headline_score = average.round() as u32;
    i18n.t_args(
        key,
        &[
            ("total_urls", batch.summary.total_urls.to_string()),
            ("score", headline_score.to_string()),
        ],
    )
}

pub(super) fn build_batch_appendix(
    locale: &str,
    batch: &BatchReport,
    normalized_reports: &[NormalizedReport],
) -> BatchAppendixData {
    debug_assert_eq!(
        batch.reports.len(),
        normalized_reports.len(),
        "batch appendix requires one normalized report per raw report"
    );

    BatchAppendixData {
        per_url: batch
            .reports
            .iter()
            .zip(normalized_reports.iter())
            .map(|(r, normalized)| {
                UrlAppendix {
                    url: r.url.clone(),
                    violations: normalized
                        .findings
                        .iter()
                        .map(|finding| AppendixViolation {
                            rule: finding.rule_id.clone(),
                            // Stored title is canonical English (#406); re-derive
                            // the localized title (taxonomy + SEO headings) for
                            // non-English reports.
                            rule_name: super::actions::localized_finding_text(locale, finding).0,
                            severity: finding.severity,
                            message: finding.description.clone(),
                            fix_suggestion: finding
                                .occurrences
                                .iter()
                                .find_map(|occ| occ.fix_suggestion.clone()),
                            affected_elements: finding
                                .occurrences
                                .iter()
                                .map(|occ| AffectedElement {
                                    selector: occ
                                        .selector
                                        .clone()
                                        .unwrap_or_else(|| occ.node_id.clone()),
                                    node_id: occ.node_id.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}

pub(super) fn yes_no(locale: &str, val: bool) -> String {
    crate::i18n::text(if val { "yes" } else { "no" }, locale == "en")
}

/// Localized label for a security header's classification tier (#578) —
/// lets the security header table show at a glance which missing headers
/// are baseline hygiene vs. situational, instead of a flat present/missing
/// count.
pub(super) fn security_header_tier_label(
    locale: &str,
    tier: crate::security::HeaderTier,
) -> String {
    use crate::security::HeaderTier::*;
    let key = match tier {
        Baseline => "output-header-tier-baseline",
        ArchitectureDependent => "output-header-tier-architecture-dependent",
        ContextDependent => "output-header-tier-context-dependent",
        NotAssessable => "output-header-tier-not-assessable",
    };
    crate::i18n::text(key, locale == "en")
}

/// Localized label distinguishing "header not sent at all" from "header
/// sent but weak/permissive" (#578) — makes a distinction that already
/// exists internally via `SecurityIssue::issue_type` visible in the
/// rendered report instead of leaving it implicit in the finding message.
pub(super) fn security_issue_kind_label(locale: &str, issue_type: &str) -> String {
    let missing = issue_type == "missing_header" || issue_type == "missing_https";
    let key = if missing {
        "output-issue-kind-missing"
    } else {
        "output-issue-kind-misconfigured"
    };
    crate::i18n::text(key, locale == "en")
}

pub(super) fn truncate_list(items: &[String], limit: usize) -> String {
    let mut values: Vec<String> = items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .cloned()
        .collect();
    values.sort();
    values.dedup();

    let shown: Vec<String> = values.iter().take(limit).cloned().collect();
    if values.len() > limit {
        format!("{} +{}", shown.join(", "), values.len() - limit)
    } else {
        shown.join(", ")
    }
}

pub(super) fn truncate_url_list(items: &[String], limit: usize, max_len: usize) -> String {
    let shortened: Vec<String> = items
        .iter()
        .map(|item| truncate_url(item, max_len))
        .collect();
    truncate_list(&shortened, limit)
}

pub(crate) fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
}

/// German display name for a module identifier used in prose (site-quality-score
/// weights, indicator notes). English keeps the canonical identifier.
pub(crate) fn localized_module_name(name: &str, i18n: &I18n) -> String {
    // Fluent keys cannot contain spaces, so multi-word module names like
    // "Best Practices" or "AI Visibility" map to hyphenated keys (#447).
    let key = format!("module-{}", name.to_lowercase().replace(' ', "-"));
    let translated = i18n.t(&key);
    if translated == key {
        // Fallback to name if not found in Fluent
        name.to_string()
    } else {
        translated
    }
}
