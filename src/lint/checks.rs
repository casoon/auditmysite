//! Individual deterministic checks, each operating on a parsed report
//! [`serde_json::Value`] and appending [`LintFinding`]s.

use serde_json::Value;

use crate::registry::{MetricKind, CERTIFICATE, LETTER_GRADE};
use crate::taxonomy::Severity;

use super::LintFinding;

pub fn run_all_checks(report: &Value, findings: &mut Vec<LintFinding>) {
    check_score_consistency(report, findings);
    check_grade_certificate_consistency(report, findings);
    check_severity_occurrence_sums(report, findings);
    check_metric_context_matches_registry(report, findings);
    check_registry_docs_urls_well_formed(findings);
}

// ─── Score consistency ────────────────────────────────────────────────────

const CHECK_SCORE_ALIAS: &str = "score_alias_matches_overall";
const CHECK_SINGLE_SUMMARY_MATCHES_PAGE: &str = "single_summary_matches_page";

/// `summary.score` must always equal `summary.overall_score` (registry:
/// "Compatibility alias for summary.overall_score"), and for single-page
/// reports the summary must equal the (sole) page's own scores — this is the
/// shape of the historical "18/100 vs 20/100" class of bug (#511 corpus).
fn check_score_consistency(report: &Value, findings: &mut Vec<LintFinding>) {
    let Some(summary) = report.get("summary") else {
        return;
    };

    if let (Some(overall), Some(alias)) = (
        summary.get("overall_score").and_then(Value::as_i64),
        summary.get("score").and_then(Value::as_i64),
    ) {
        if overall != alias {
            findings.push(LintFinding {
                check_id: CHECK_SCORE_ALIAS,
                evidence_path: "summary.score".to_string(),
                expected: format!("{overall} (equal to summary.overall_score)"),
                actual: alias.to_string(),
                severity: Severity::High,
            });
        }
    }

    let report_type = report
        .get("report_type")
        .and_then(Value::as_str)
        .unwrap_or("");
    if report_type != "single" {
        return;
    }
    let Some(page) = report
        .get("pages")
        .and_then(Value::as_array)
        .and_then(|pages| pages.first())
    else {
        return;
    };

    for field in ["accessibility_score", "overall_score"] {
        if let (Some(summary_val), Some(page_val)) = (
            summary.get(field).and_then(Value::as_i64),
            page.get(field).and_then(Value::as_i64),
        ) {
            if summary_val != page_val {
                findings.push(LintFinding {
                    check_id: CHECK_SINGLE_SUMMARY_MATCHES_PAGE,
                    evidence_path: format!("summary.{field} vs pages[0].{field}"),
                    expected: page_val.to_string(),
                    actual: summary_val.to_string(),
                    severity: Severity::Critical,
                });
            }
        }
    }
}

// ─── Grade/certificate consistency ────────────────────────────────────────

const CHECK_GRADE_MATCHES_SCORE: &str = "grade_matches_overall_score";
const CHECK_CERTIFICATE_MATCHES_SCORE: &str = "certificate_matches_overall_score";

/// Risk-gate inputs for a `summary` or `pages[i]` node. `pages[i]` carries a
/// nested `risk` object with full detail; `summary` only ever carries a flat
/// `risk_level` string (no aggregate `legal_flags`/`blocking_issues`), so
/// those default to 0 there — level-only gating (High/Critical) still works.
fn risk_gate_inputs(node: &Value) -> (Option<&str>, i64, i64) {
    if let Some(risk) = node.get("risk") {
        let level = risk.get("level").and_then(Value::as_str);
        let legal_flags = risk.get("legal_flags").and_then(Value::as_i64).unwrap_or(0);
        let blocking_issues = risk
            .get("blocking_issues")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        (level, legal_flags, blocking_issues)
    } else {
        let level = node.get("risk_level").and_then(Value::as_str);
        (level, 0, 0)
    }
}

/// Certificate values `gate_certificate_by_risk` (`src/audit/normalized.rs`)
/// can produce from a given score-implied certificate, mirroring its exact
/// precedence: Critical risk always wins ("NICHT BESTANDEN"); otherwise a
/// positive certificate gets downgraded to "EINGESCHRÄNKT" when risk is High
/// or there's any legal/blocking flag. Returns every value that's a legitimate
/// outcome for this input, not just the one score-based value.
fn acceptable_certificates(
    score_based: &'static str,
    risk_level: Option<&str>,
    legal_flags: i64,
    blocking_issues: i64,
) -> Vec<&'static str> {
    let mut acceptable = vec![score_based];
    if risk_level == Some("critical") {
        acceptable.push("NICHT BESTANDEN");
    }
    let is_positive = matches!(score_based, "AUSBAUFÄHIG" | "STABIL" | "GUT" | "SEHR GUT");
    let does_not_pass = risk_level == Some("high") || legal_flags > 0 || blocking_issues > 0;
    if is_positive && does_not_pass {
        acceptable.push("EINGESCHRÄNKT");
    }
    acceptable
}

/// `grade` and `certificate` must be derivable from `overall_score` via the
/// same shared `BandSet` the production code uses
/// (`audit::scoring::AccessibilityScorer::calculate_grade`/
/// `calculate_certificate`) — catches a certificate/grade computed from a
/// different (e.g. stale or pre-normalization) score base. `certificate` may
/// also legitimately be a risk-gated downgrade of the score-based value; see
/// [`acceptable_certificates`].
fn check_grade_and_certificate(
    node: &Value,
    evidence_prefix: &str,
    findings: &mut Vec<LintFinding>,
) {
    let Some(overall) = node.get("overall_score").and_then(Value::as_i64) else {
        return;
    };

    if let Some(grade) = node.get("grade").and_then(Value::as_str) {
        let expected = LETTER_GRADE.label(overall as f32, false);
        if grade != expected {
            findings.push(LintFinding {
                check_id: CHECK_GRADE_MATCHES_SCORE,
                evidence_path: format!("{evidence_prefix}.grade"),
                expected: expected.to_string(),
                actual: grade.to_string(),
                severity: Severity::Critical,
            });
        }
    }

    if let Some(certificate) = node.get("certificate").and_then(Value::as_str) {
        let score_based = CERTIFICATE.label(overall as f32, false);
        let (risk_level, legal_flags, blocking_issues) = risk_gate_inputs(node);
        let acceptable =
            acceptable_certificates(score_based, risk_level, legal_flags, blocking_issues);
        if !acceptable.contains(&certificate) {
            findings.push(LintFinding {
                check_id: CHECK_CERTIFICATE_MATCHES_SCORE,
                evidence_path: format!("{evidence_prefix}.certificate"),
                expected: acceptable.join(" or "),
                actual: certificate.to_string(),
                severity: Severity::Critical,
            });
        }
    }
}

fn check_grade_certificate_consistency(report: &Value, findings: &mut Vec<LintFinding>) {
    if let Some(summary) = report.get("summary") {
        check_grade_and_certificate(summary, "summary", findings);
    }
    if let Some(pages) = report.get("pages").and_then(Value::as_array) {
        for (i, page) in pages.iter().enumerate() {
            check_grade_and_certificate(page, &format!("pages[{i}]"), findings);
        }
    }
}

// ─── Severity/occurrence count sums ───────────────────────────────────────

const CHECK_SEVERITY_TOTAL_SUM: &str = "severity_counts_total_matches_sum";
const CHECK_OCCURRENCE_TOTAL_SUM: &str = "occurrence_counts_total_matches_sum";
const CHECK_VIOLATED_RULE_COUNT_SCOPE: &str = "violated_rule_count_matches_severity_total";
const CHECK_VIOLATION_COUNT_SCOPE: &str = "violation_count_matches_occurrence_total";

fn counts_sum(node: &Value) -> Option<i64> {
    Some(
        node.get("critical")?.as_i64()?
            + node.get("high")?.as_i64()?
            + node.get("medium")?.as_i64()?
            + node.get("low")?.as_i64()?,
    )
}

/// How a scoped count field relates to its counts-block total. Exact for a
/// single page (a page's own distinct-rule count trivially equals its own
/// `severity_counts.total`); at most equal for a batch `summary`, where the
/// scoped count is a *globally* distinct count while the counts-block total
/// is the *sum of per-page* distinct-rule rows — the same rule repeating
/// across pages inflates the total but not the global distinct count
/// (confirmed via a real batch report during the #509 report-critic eval).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeRelation {
    Exact,
    AtMost,
}

/// Which counts block and scoped-count field this check pass covers, and the
/// check ids to report under — one spec per (`severity_counts`,
/// `violated_rule_count`) / (`occurrence_counts`, `violation_count`) pair.
struct CountsCheckSpec {
    block_field: &'static str,
    scope_count_field: &'static str,
    check_total_sum: &'static str,
    check_scope_match: &'static str,
}

fn check_counts_block(
    node: &Value,
    spec: &CountsCheckSpec,
    scope_relation: ScopeRelation,
    evidence_prefix: &str,
    findings: &mut Vec<LintFinding>,
) {
    let Some(counts) = node.get(spec.block_field) else {
        return;
    };
    let total = counts.get("total").and_then(Value::as_i64);

    if let (Some(total), Some(sum)) = (total, counts_sum(counts)) {
        if total != sum {
            findings.push(LintFinding {
                check_id: spec.check_total_sum,
                evidence_path: format!("{evidence_prefix}.{}.total", spec.block_field),
                expected: sum.to_string(),
                actual: total.to_string(),
                severity: Severity::High,
            });
        }
    }

    if let (Some(scope_count), Some(total)) = (
        node.get(spec.scope_count_field).and_then(Value::as_i64),
        total,
    ) {
        let violated = match scope_relation {
            ScopeRelation::Exact => scope_count != total,
            ScopeRelation::AtMost => scope_count > total,
        };
        if violated {
            findings.push(LintFinding {
                check_id: spec.check_scope_match,
                evidence_path: format!("{evidence_prefix}.{}", spec.scope_count_field),
                expected: match scope_relation {
                    ScopeRelation::Exact => total.to_string(),
                    ScopeRelation::AtMost => format!("<= {total}"),
                },
                actual: scope_count.to_string(),
                severity: Severity::Medium,
            });
        }
    }
}

/// `severity_counts.total`/`occurrence_counts.total` must equal the sum of
/// their four severity fields, and `violated_rule_count`/`violation_count`
/// must match the corresponding scoped total — catches a counter whose scope
/// silently drifted from its label (#511 corpus: "Zähler ohne Scope"). At the
/// batch `summary` level the scoped counts are only bounded above, not exact
/// — see [`ScopeRelation`].
fn check_severity_occurrence_sums(report: &Value, findings: &mut Vec<LintFinding>) {
    let is_batch = report.get("report_type").and_then(Value::as_str) == Some("batch");

    const SEVERITY_SPEC: CountsCheckSpec = CountsCheckSpec {
        block_field: "severity_counts",
        scope_count_field: "violated_rule_count",
        check_total_sum: CHECK_SEVERITY_TOTAL_SUM,
        check_scope_match: CHECK_VIOLATED_RULE_COUNT_SCOPE,
    };
    const OCCURRENCE_SPEC: CountsCheckSpec = CountsCheckSpec {
        block_field: "occurrence_counts",
        scope_count_field: "violation_count",
        check_total_sum: CHECK_OCCURRENCE_TOTAL_SUM,
        check_scope_match: CHECK_VIOLATION_COUNT_SCOPE,
    };

    let check_node = |node: &Value,
                      evidence_prefix: &str,
                      scope_relation: ScopeRelation,
                      findings: &mut Vec<LintFinding>| {
        check_counts_block(
            node,
            &SEVERITY_SPEC,
            scope_relation,
            evidence_prefix,
            findings,
        );
        check_counts_block(
            node,
            &OCCURRENCE_SPEC,
            scope_relation,
            evidence_prefix,
            findings,
        );
    };

    if let Some(summary) = report.get("summary") {
        let summary_relation = if is_batch {
            ScopeRelation::AtMost
        } else {
            ScopeRelation::Exact
        };
        check_node(summary, "summary", summary_relation, findings);
    }
    if let Some(pages) = report.get("pages").and_then(Value::as_array) {
        for (i, page) in pages.iter().enumerate() {
            check_node(page, &format!("pages[{i}]"), ScopeRelation::Exact, findings);
        }
    }
}

// ─── metric_context drift ──────────────────────────────────────────────────

const CHECK_METRIC_CONTEXT_MATCHES_REGISTRY: &str = "metric_context_matches_registry";

type Definition = (String, String, String);

fn expected_definitions(report_type: &str, count_kind: bool) -> Vec<Definition> {
    crate::registry::REGISTRY
        .iter()
        .filter(|m| (m.kind == MetricKind::Count) == count_kind)
        .map(|m| {
            (
                m.json_path.to_string(),
                m.unit.to_string(),
                m.meaning_for(report_type).to_string(),
            )
        })
        .collect()
}

fn actual_definitions(defs: &Value) -> Vec<Definition> {
    defs.as_array()
        .map(|arr| {
            arr.iter()
                .map(|d| {
                    (
                        field_str(d, "field"),
                        field_str(d, "unit"),
                        field_str(d, "meaning"),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn field_str(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// The report's own `metric_context.score_definitions`/`count_definitions`
/// must match what `REGISTRY` (#506) currently generates — catches a cached
/// or hand-edited report whose `metric_context` drifted from a prior tool
/// version instead of the registry it claims to describe.
fn check_metric_context_matches_registry(report: &Value, findings: &mut Vec<LintFinding>) {
    let Some(metric_context) = report.get("metric_context") else {
        return;
    };
    let report_type = report
        .get("report_type")
        .and_then(Value::as_str)
        .unwrap_or("single");

    for (json_key, count_kind) in [("score_definitions", false), ("count_definitions", true)] {
        let expected = expected_definitions(report_type, count_kind);
        let actual = metric_context
            .get(json_key)
            .map(actual_definitions)
            .unwrap_or_default();
        if actual != expected {
            findings.push(LintFinding {
                check_id: CHECK_METRIC_CONTEXT_MATCHES_REGISTRY,
                evidence_path: format!("metric_context.{json_key}"),
                expected: format!("{} entries matching the current registry", expected.len()),
                actual: format!(
                    "{} entries, differs from the current registry",
                    actual.len()
                ),
                severity: Severity::Medium,
            });
        }
    }
}

// ─── Registry docs_url well-formedness ────────────────────────────────────

const CHECK_DOCS_URL_WELL_FORMED: &str = "registry_docs_url_well_formed";

/// Every `REGISTRY` (#506) entry's `docs_url` must be a non-empty
/// `<path>#<anchor>` reference. This runs independent of the linted report —
/// it's a regression guard on the registry itself. The deeper check (does the
/// anchor actually exist in `docs/OUTPUT_CONTRACT.md`) stays in
/// `tests/registry_contract.rs`, which runs in the dev/CI checkout where that
/// file exists; a released `auditmysite` binary has no such guarantee, so
/// `report-lint` only validates shape, not filesystem reachability.
fn check_registry_docs_urls_well_formed(findings: &mut Vec<LintFinding>) {
    for metric in crate::registry::REGISTRY {
        let well_formed = match metric.docs_url.split_once('#') {
            Some((path, anchor)) => !path.is_empty() && !anchor.is_empty(),
            None => false,
        };
        if !well_formed {
            findings.push(LintFinding {
                check_id: CHECK_DOCS_URL_WELL_FORMED,
                evidence_path: format!("registry[{}].docs_url", metric.id),
                expected: "a non-empty '<path>#<anchor>' reference".to_string(),
                actual: format!("{:?}", metric.docs_url),
                severity: Severity::Low,
            });
        }
    }
}

// ─── PDF traceability (scoped: certificate token only) ────────────────────

const CHECK_PDF_CERTIFICATE_TRACEABLE: &str = "pdf_certificate_traceable_to_json";

/// Checks that the certificate token the JSON's `overall_score` implies (via
/// the same shared `CERTIFICATE` `BandSet` the PDF's `cover::batch_certificate_label`/
/// `audit::scoring::AccessibilityScorer::calculate_certificate` use) actually
/// appears in the report's rendered Typst source.
///
/// Deliberately narrow: this does not scan for arbitrary numbers/claims in
/// the PDF text (a naive "does this number appear anywhere" scan has a real
/// false-positive risk — dates, page counts, unrelated percentages). It only
/// checks a single, distinctive, all-caps token this build's `BandSet`
/// computes from data already in the report, so a mismatch is unambiguous.
/// Presence-only (not "no other certificate word may appear") because a
/// legend/explanation of the band system elsewhere in the PDF may
/// legitimately mention other certificate words.
pub(super) fn check_pdf_certificate_traceability(
    report: &Value,
    typst_source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let Some(overall_score) = report
        .get("summary")
        .and_then(|s| s.get("overall_score"))
        .and_then(Value::as_i64)
    else {
        return;
    };

    let expected = CERTIFICATE.label(overall_score as f32, false);
    if !typst_source.contains(expected) {
        findings.push(LintFinding {
            check_id: CHECK_PDF_CERTIFICATE_TRACEABLE,
            evidence_path: "summary.overall_score vs --typst-source".to_string(),
            expected: format!("Typst source contains {expected:?}"),
            actual: "not found in Typst source".to_string(),
            severity: Severity::Low,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Builds the exact `metric_context` shape production code generates
    /// from `REGISTRY` (mirrors `output::json::metric_context`), so the
    /// "clean" fixture doesn't drift from the registry by construction.
    fn registry_metric_context_json(report_type: &str) -> Value {
        let mut score_definitions = Vec::new();
        let mut count_definitions = Vec::new();
        for metric in crate::registry::REGISTRY {
            let def = json!({
                "field": metric.json_path,
                "unit": metric.unit,
                "meaning": metric.meaning_for(report_type),
            });
            if metric.kind == MetricKind::Count {
                count_definitions.push(def);
            } else {
                score_definitions.push(def);
            }
        }
        json!({
            "score_scale": {"minimum": 0, "maximum": 100, "higher_is_better": true},
            "score_definitions": score_definitions,
            "count_definitions": count_definitions,
        })
    }

    fn clean_single_report() -> Value {
        json!({
            "schema_version": "2.0",
            "report_type": "single",
            "metric_context": registry_metric_context_json("single"),
            "summary": {
                "accessibility_score": 82,
                "overall_score": 82,
                "score": 82,
                "grade": "B",
                "certificate": "GUT",
                "severity_counts": {"critical": 1, "high": 2, "medium": 0, "low": 0, "total": 3},
                "violated_rule_count": 3,
                "occurrence_counts": {"critical": 1, "high": 3, "medium": 0, "low": 0, "total": 4},
                "violation_count": 4
            },
            "pages": [
                {
                    "accessibility_score": 82,
                    "overall_score": 82,
                    "grade": "B",
                    "certificate": "GUT",
                    "severity_counts": {"critical": 1, "high": 2, "medium": 0, "low": 0, "total": 3},
                    "violated_rule_count": 3,
                    "occurrence_counts": {"critical": 1, "high": 3, "medium": 0, "low": 0, "total": 4},
                    "violation_count": 4
                }
            ]
        })
    }

    #[test]
    fn clean_report_yields_no_findings() {
        let mut findings = Vec::new();
        run_all_checks(&clean_single_report(), &mut findings);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn detects_summary_score_alias_mismatch() {
        let mut report = clean_single_report();
        report["summary"]["score"] = json!(80); // was 82
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings.iter().any(|f| f.check_id == CHECK_SCORE_ALIAS));
    }

    #[test]
    fn detects_summary_vs_page_score_mismatch() {
        // The historical "18/100 vs 20/100" shape: summary and the sole page
        // disagree on the same canonical score.
        let mut report = clean_single_report();
        report["pages"][0]["overall_score"] = json!(80);
        report["pages"][0]["grade"] = json!("B");
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_SINGLE_SUMMARY_MATCHES_PAGE));
    }

    #[test]
    fn detects_grade_from_wrong_score_base() {
        let mut report = clean_single_report();
        report["summary"]["grade"] = json!("A"); // score 82 should be "B"
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_GRADE_MATCHES_SCORE));
    }

    #[test]
    fn detects_certificate_from_wrong_score_base() {
        let mut report = clean_single_report();
        report["summary"]["certificate"] = json!("SEHR GUT"); // score 82 should be "GUT"
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_CERTIFICATE_MATCHES_SCORE));
    }

    #[test]
    fn certificate_downgraded_by_risk_gate_is_not_flagged() {
        // overall_score 82 implies "GUT", but production code
        // (gate_certificate_by_risk) downgrades a positive certificate to
        // "EINGESCHRÄNKT" whenever risk is High -- report-lint must accept
        // this as a legitimate outcome, not a score/certificate mismatch.
        let mut report = clean_single_report();
        report["summary"]["certificate"] = json!("EINGESCHRÄNKT");
        report["summary"]["risk_level"] = json!("high");
        report["pages"][0]["certificate"] = json!("EINGESCHRÄNKT");
        report["pages"][0]["risk"] =
            json!({"level": "high", "legal_flags": 0, "blocking_issues": 0});
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(
            !findings
                .iter()
                .any(|f| f.check_id == CHECK_CERTIFICATE_MATCHES_SCORE),
            "unexpected findings: {findings:?}"
        );
    }

    #[test]
    fn certificate_still_flagged_when_risk_does_not_justify_downgrade() {
        // Same downgraded certificate as above, but risk is "low" -- nothing
        // justifies "EINGESCHRÄNKT" here, so this must still be flagged.
        let mut report = clean_single_report();
        report["summary"]["certificate"] = json!("EINGESCHRÄNKT");
        report["summary"]["risk_level"] = json!("low");
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_CERTIFICATE_MATCHES_SCORE));
    }

    #[test]
    fn detects_severity_total_sum_mismatch() {
        let mut report = clean_single_report();
        report["summary"]["severity_counts"]["total"] = json!(99);
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_SEVERITY_TOTAL_SUM));
    }

    #[test]
    fn detects_violated_rule_count_scope_mismatch() {
        let mut report = clean_single_report();
        report["summary"]["violated_rule_count"] = json!(99);
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_VIOLATED_RULE_COUNT_SCOPE));
    }

    fn clean_batch_report() -> Value {
        json!({
            "schema_version": "2.0",
            "report_type": "batch",
            "summary": {
                "accessibility_score": 82,
                "overall_score": 82,
                "score": 82,
                "grade": "B",
                "certificate": "GUT",
                // 3 distinct rules across the whole batch (global count), but
                // severity_counts.total sums per-page distinct-rule rows, so
                // it's legitimately larger when a rule repeats across pages.
                "severity_counts": {"critical": 0, "high": 16, "medium": 19, "low": 0, "total": 35},
                "violated_rule_count": 3,
                "occurrence_counts": {"critical": 0, "high": 190, "medium": 35, "low": 0, "total": 225},
                "violation_count": 225
            },
            "pages": [
                {
                    "accessibility_score": 82,
                    "overall_score": 82,
                    "grade": "B",
                    "certificate": "GUT",
                    "severity_counts": {"critical": 0, "high": 1, "medium": 1, "low": 0, "total": 2},
                    "violated_rule_count": 2,
                    "occurrence_counts": {"critical": 0, "high": 12, "medium": 3, "low": 0, "total": 15},
                    "violation_count": 15
                }
            ]
        })
    }

    #[test]
    fn batch_summary_violated_rule_count_below_severity_total_is_not_flagged() {
        let mut findings = Vec::new();
        run_all_checks(&clean_batch_report(), &mut findings);
        assert!(
            !findings
                .iter()
                .any(|f| f.check_id == CHECK_VIOLATED_RULE_COUNT_SCOPE),
            "unexpected findings: {findings:?}"
        );
    }

    #[test]
    fn batch_summary_violated_rule_count_above_severity_total_is_still_flagged() {
        // A global distinct-rule count can never exceed the sum of per-page
        // distinct-rule rows -- this direction of mismatch is still a real bug.
        let mut report = clean_batch_report();
        report["summary"]["violated_rule_count"] = json!(999);
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_VIOLATED_RULE_COUNT_SCOPE));
    }

    #[test]
    fn batch_page_violated_rule_count_mismatch_is_still_flagged_exactly() {
        // Per-page scope relation stays exact even in a batch report -- only
        // the top-level summary aggregate gets the relaxed AtMost relation.
        let mut report = clean_batch_report();
        report["pages"][0]["violated_rule_count"] = json!(99);
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_VIOLATED_RULE_COUNT_SCOPE));
    }

    #[test]
    fn detects_stale_metric_context() {
        let mut report = clean_single_report();
        report["metric_context"]["score_definitions"] = json!([]);
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_METRIC_CONTEXT_MATCHES_REGISTRY));
    }

    #[test]
    fn missing_optional_fields_do_not_panic_or_false_positive() {
        let report = json!({
            "report_type": "single",
            "summary": {"overall_score": 50}
        });
        let mut findings = Vec::new();
        run_all_checks(&report, &mut findings);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn pdf_certificate_traceability_passes_when_token_present() {
        let report = clean_single_report(); // overall_score 82 -> "GUT"
        let typst_source = "... some rendered PDF text ... GUT ... more text ...";
        let mut findings = Vec::new();
        check_pdf_certificate_traceability(&report, typst_source, &mut findings);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn pdf_certificate_traceability_flags_missing_token() {
        let report = clean_single_report(); // overall_score 82 -> "GUT"
        let typst_source = "... some rendered PDF text without the certificate word ...";
        let mut findings = Vec::new();
        check_pdf_certificate_traceability(&report, typst_source, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.check_id == CHECK_PDF_CERTIFICATE_TRACEABLE));
    }
}
