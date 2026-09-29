//! Technician-mode aggregates next to per-page JSON reports (plan 67).
//!
//! A batch run with `--per-page-reports -f json` writes one single-page
//! report per URL. For people who fix issues rather than read a report, two
//! machine-friendly files sit next to them:
//!
//! - `index.json` ([`TechnicianIndex`]): every attempted URL in input order,
//!   with its report file (or none), status `ok`/`blocked`/`failed` plus
//!   reason, scores, finding/occurrence counts and audit-quality status.
//! - `findings.jsonl` ([`FindingRow`], one JSON object per line): every
//!   finding occurrence of every page, flat. The per-page report caps its
//!   stored occurrences per finding; this list does not.
//!
//! Both are canonical English (#406): every text is taken from the
//! English-baked analysis results, nothing is localized here.
//! Schemas: `docs/technician-index.schema.json`,
//! `docs/technician-finding.schema.json`.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::audit::normalized::{wcag_criterion_of, wcag_group_key, NormalizedReport};
use crate::audit::{AuditQualityStatus, AuditReport, BatchReport};
use crate::taxonomy::{RuleLookup, Severity, SeverityExt};

/// Version of the `index.json` / `findings.jsonl` shape.
pub const TECHNICIAN_SCHEMA_VERSION: &str = "1.0";

/// `index.json`: one entry per attempted URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicianIndex {
    pub schema_version: String,
    pub tool_version: String,
    pub totals: IndexTotals,
    pub pages: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexTotals {
    pub attempted: usize,
    pub ok: usize,
    pub blocked: usize,
    pub failed: usize,
    /// Number of lines in `findings.jsonl`.
    pub occurrences: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    /// Audited; a report file exists.
    Ok,
    /// The server answered with a bot wall, challenge or access denial
    /// instead of the page; not audited.
    Blocked,
    /// The audit did not complete (timeout, navigation or browser error).
    Failed,
}

/// One attempted URL. Score and count fields are `null` unless `status` is `ok`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub url: String,
    /// Report file name inside the output directory; `null` without a report.
    pub file: Option<String>,
    pub status: PageStatus,
    /// Why the page is blocked or failed; `null` for `ok`.
    pub reason: Option<String>,
    pub overall_score: Option<u32>,
    pub accessibility_score: Option<u32>,
    /// Distinct `(source, rule_id)` pairs among this page's `findings.jsonl` lines.
    pub finding_count: Option<usize>,
    /// Number of this page's `findings.jsonl` lines.
    pub occurrence_count: Option<usize>,
    /// `execution.quality.status` of the page report.
    pub audit_quality: Option<AuditQualityStatus>,
}

/// Which analysis produced a [`FindingRow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSource {
    /// WCAG rule violation (accessibility tree / page rules).
    Wcag,
    /// Keyboard/interaction journey (`interactive_findings`).
    Journey,
    /// SEO heading-structure issue.
    Seo,
    /// HTML5 conformance (html-conform checker).
    HtmlConform,
}

/// One line of `findings.jsonl`: a single occurrence on a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRow {
    pub url: String,
    pub source: FindingSource,
    /// Taxonomy rule id for `wcag` (the per-page report's `findings[].rule_id`),
    /// `journey.<Kind>`, `seo.headings.<issue>` or the html-conform rule id.
    pub rule_id: String,
    /// WCAG success criterion, e.g. `1.1.1`; `null` outside `wcag`.
    pub wcag_criterion: Option<String>,
    /// WCAG level `A`/`AA`/`AAA`; `null` outside `wcag`.
    pub level: Option<String>,
    /// Severity of this occurrence. For `html_conform` the checker's
    /// error/warning/info map to high/medium/low.
    pub severity: Severity,
    /// CSS selector of the affected element, when known.
    pub selector: Option<String>,
    /// `line:column` in the rendered HTML (`html_conform` only).
    pub location: Option<String>,
    pub message: String,
    pub fix_suggestion: Option<String>,
    /// Which viewport pass saw it: `mobile-only`, `desktop-only`,
    /// `both-viewports` (`wcag` only; empty when not recorded).
    pub viewport_tags: Vec<String>,
}

/// Every finding occurrence of one audited page, in a stable order:
/// WCAG violations, journey findings, SEO heading issues, HTML conformance.
pub fn finding_rows(report: &AuditReport, normalized: &NormalizedReport) -> Vec<FindingRow> {
    let url = &report.url;
    let mut rows = Vec::new();

    for v in &report.accessibility.wcag_results.violations {
        let key = wcag_group_key(v);
        let rule_id = RuleLookup::by_legacy_wcag_id(key)
            .map(|rule| rule.id.to_string())
            .unwrap_or_else(|| format!("unknown.{key}"));
        rows.push(FindingRow {
            url: url.clone(),
            source: FindingSource::Wcag,
            rule_id,
            wcag_criterion: Some(wcag_criterion_of(&v.rule)),
            level: Some(v.level.to_string()),
            severity: v.severity,
            selector: v.selector.clone(),
            location: None,
            message: v.message.clone(),
            fix_suggestion: v.fix_suggestion.clone(),
            viewport_tags: v.tags.clone(),
        });
    }

    for f in &normalized.interactive_findings {
        let kind = serde_json::to_value(f.kind)
            .ok()
            .and_then(|k| k.as_str().map(str::to_string))
            .unwrap_or_default();
        rows.push(FindingRow {
            url: url.clone(),
            source: FindingSource::Journey,
            rule_id: format!("journey.{kind}"),
            wcag_criterion: None,
            level: None,
            severity: f.severity,
            selector: f.values.selector.clone(),
            location: None,
            message: f.message.clone(),
            fix_suggestion: f.fix_suggestion.clone(),
            viewport_tags: Vec::new(),
        });
    }

    if let Some(seo) = &report.discoverability.seo {
        for issue in &seo.headings.issues {
            rows.push(FindingRow {
                url: url.clone(),
                source: FindingSource::Seo,
                rule_id: format!("seo.headings.{}", issue.issue_type),
                wcag_criterion: None,
                level: None,
                severity: issue.severity,
                selector: None,
                location: None,
                message: issue.message.clone(),
                fix_suggestion: None,
                viewport_tags: Vec::new(),
            });
        }
    }

    if let Some(html) = report.html_conform.as_ref().filter(|h| h.checked) {
        for f in &html.findings {
            rows.push(FindingRow {
                url: url.clone(),
                source: FindingSource::HtmlConform,
                rule_id: f.rule_id.clone(),
                wcag_criterion: None,
                level: None,
                severity: Severity::from_legacy_module(&f.severity),
                selector: None,
                location: f.location.clone(),
                message: f.message.clone(),
                fix_suggestion: None,
                viewport_tags: Vec::new(),
            });
        }
    }

    rows
}

/// A page report's contribution to the index, computed from its rows.
pub fn ok_entry(
    report: &AuditReport,
    normalized: &NormalizedReport,
    rows: &[FindingRow],
    file: String,
) -> IndexEntry {
    let distinct: BTreeSet<(FindingSource, &str)> = rows
        .iter()
        .map(|r| (r.source, r.rule_id.as_str()))
        .collect();
    IndexEntry {
        url: report.url.clone(),
        file: Some(file),
        status: PageStatus::Ok,
        reason: None,
        overall_score: Some(normalized.overall_score),
        accessibility_score: Some(normalized.score),
        finding_count: Some(distinct.len()),
        occurrence_count: Some(rows.len()),
        audit_quality: Some(normalized.execution.quality.status),
    }
}

/// Assemble `index.json` in the order the URLs were attempted.
///
/// `attempted` is the audited URL list in input order; `ok_entries` holds the
/// entries built with [`ok_entry`], keyed by URL. Every URL without a report
/// is looked up in the batch errors; `blocked_reason` separates bot walls from
/// other failures.
pub fn build_index(
    attempted: &[String],
    batch: &BatchReport,
    mut ok_entries: HashMap<String, IndexEntry>,
) -> TechnicianIndex {
    let errors: HashMap<&str, &crate::audit::BatchError> =
        batch.errors.iter().map(|e| (e.url.as_str(), e)).collect();
    let mut totals = IndexTotals::default();
    let mut pages = Vec::with_capacity(attempted.len());

    for url in attempted {
        let entry = if let Some(entry) = ok_entries.remove(url) {
            entry
        } else {
            let (status, reason) = match errors.get(url.as_str()) {
                Some(e) => match &e.blocked_reason {
                    Some(reason) => (PageStatus::Blocked, reason.clone()),
                    None => (PageStatus::Failed, e.error.clone()),
                },
                None => (PageStatus::Failed, "no audit result recorded".to_string()),
            };
            IndexEntry {
                url: url.clone(),
                file: None,
                status,
                reason: Some(reason),
                overall_score: None,
                accessibility_score: None,
                finding_count: None,
                occurrence_count: None,
                audit_quality: None,
            }
        };
        match entry.status {
            PageStatus::Ok => totals.ok += 1,
            PageStatus::Blocked => totals.blocked += 1,
            PageStatus::Failed => totals.failed += 1,
        }
        totals.occurrences += entry.occurrence_count.unwrap_or(0);
        pages.push(entry);
    }
    totals.attempted = pages.len();

    TechnicianIndex {
        schema_version: TECHNICIAN_SCHEMA_VERSION.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        totals,
        pages,
    }
}

/// Serialize rows as JSON Lines: one compact object per line, `\n`-terminated.
pub fn to_jsonl(rows: &[FindingRow]) -> serde_json::Result<String> {
    let mut out = String::new();
    for row in rows {
        out.push_str(&serde_json::to_string(row)?);
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{normalize, BatchError};
    use crate::cli::WcagLevel;
    use crate::wcag::{Violation, WcagResults};

    fn report(url: &str, violations: Vec<Violation>) -> AuditReport {
        let mut results = WcagResults::new();
        results.violations = violations;
        AuditReport::new(url.to_string(), WcagLevel::AA, results, 100)
    }

    fn violation(rule: &str, selector: &str) -> Violation {
        Violation::new(
            rule,
            "Rule",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Image missing alt attribute",
            selector,
        )
        .with_selector(selector)
        .with_fix("Add an alt attribute")
    }

    fn ok(report: &AuditReport, file: &str) -> (String, IndexEntry, Vec<FindingRow>) {
        let normalized = normalize(report).normalized;
        let rows = finding_rows(report, &normalized);
        let entry = ok_entry(report, &normalized, &rows, file.to_string());
        (report.url.clone(), entry, rows)
    }

    #[test]
    fn rows_list_every_occurrence_beyond_the_report_cap() {
        let violations: Vec<Violation> = (0..8)
            .map(|i| violation("1.1.1", &format!("img:nth-of-type({i})")))
            .collect();
        let report = report("https://example.com/a", violations);
        let normalized = normalize(&report).normalized;
        let rows = finding_rows(&report, &normalized);

        assert_eq!(rows.len(), 8, "one row per occurrence, no cap");
        let finding = normalized
            .findings
            .iter()
            .find(|f| f.category == "wcag")
            .expect("wcag finding");
        assert!(
            finding.occurrences.len() < rows.len(),
            "report caps, rows do not"
        );
        for row in &rows {
            assert_eq!(row.url, "https://example.com/a");
            assert_eq!(row.source, FindingSource::Wcag);
            assert_eq!(row.rule_id, finding.rule_id, "same id as the page report");
            assert_eq!(row.wcag_criterion.as_deref(), Some("1.1.1"));
            assert_eq!(row.level.as_deref(), Some("A"));
            assert_eq!(row.fix_suggestion.as_deref(), Some("Add an alt attribute"));
            assert!(row.selector.is_some());
        }
    }

    #[test]
    fn html_conform_rows_map_checker_severity_and_keep_location() {
        let mut report = report("https://example.com/", vec![]);
        report.html_conform = Some(crate::html_conform::HtmlConformAnalysis {
            score: 90,
            checked: true,
            error_count: 1,
            warning_count: 1,
            info_count: 0,
            distinct_defect_count: 2,
            findings: vec![
                crate::html_conform::HtmlConformFinding {
                    rule_id: "schema.html5".to_string(),
                    severity: "error".to_string(),
                    message: "Element div not allowed here".to_string(),
                    location: Some("12:4".to_string()),
                    byte_offset: None,
                },
                crate::html_conform::HtmlConformFinding {
                    rule_id: "parser.html5".to_string(),
                    severity: "warning".to_string(),
                    message: "Stray end tag".to_string(),
                    location: None,
                    byte_offset: None,
                },
            ],
            raw_html: None,
        });
        let (_, entry, rows) = ok(&report, "x.json");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source, FindingSource::HtmlConform);
        assert_eq!(rows[0].severity, Severity::High);
        assert_eq!(rows[0].location.as_deref(), Some("12:4"));
        assert_eq!(rows[1].severity, Severity::Medium);
        assert_eq!(entry.finding_count, Some(2));
        assert_eq!(entry.occurrence_count, Some(2));
    }

    #[test]
    fn unchecked_html_conform_contributes_no_rows() {
        let mut report = report("https://example.com/", vec![]);
        report.html_conform = Some(crate::html_conform::HtmlConformAnalysis {
            score: 100,
            checked: false,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            distinct_defect_count: 0,
            findings: vec![crate::html_conform::HtmlConformFinding {
                rule_id: "schema.html5".to_string(),
                severity: "error".to_string(),
                message: "stale".to_string(),
                location: None,
                byte_offset: None,
            }],
            raw_html: None,
        });
        let normalized = normalize(&report).normalized;
        assert!(finding_rows(&report, &normalized).is_empty());
    }

    #[test]
    fn index_keeps_input_order_and_classifies_blocked_and_failed() {
        let a = report(
            "https://example.com/a",
            vec![violation("1.1.1", "img.a"), violation("1.1.1", "img.b")],
        );
        let c = report("https://example.com/c", vec![]);
        let batch = BatchReport::from_reports(
            vec![a.clone(), c.clone()],
            vec![
                BatchError {
                    url: "https://example.com/b".to_string(),
                    error: "Access to 'https://example.com/b' was blocked: the server answered HTTP 403. That page is not the site's content and was not audited".to_string(),
                    blocked_reason: Some("the server answered HTTP 403".to_string()),
                },
                BatchError {
                    url: "https://example.com/d".to_string(),
                    error: "Page load timeout".to_string(),
                    blocked_reason: None,
                },
            ],
            100,
        );
        let mut entries = HashMap::new();
        let mut all_rows = Vec::new();
        for r in [&c, &a] {
            let (url, entry, rows) = ok(r, &format!("{}.json", &r.url[20..]));
            entries.insert(url, entry);
            all_rows.extend(rows);
        }
        let attempted: Vec<String> = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|p| format!("https://example.com/{p}"))
            .collect();
        let index = build_index(&attempted, &batch, entries);

        let urls: Vec<&str> = index.pages.iter().map(|p| p.url.as_str()).collect();
        assert_eq!(
            urls,
            attempted.iter().map(String::as_str).collect::<Vec<_>>()
        );

        let a = &index.pages[0];
        assert_eq!(a.status, PageStatus::Ok);
        assert_eq!(a.file.as_deref(), Some("a.json"));
        assert_eq!(a.finding_count, Some(1));
        assert_eq!(a.occurrence_count, Some(2));
        assert!(a.accessibility_score.is_some() && a.overall_score.is_some());
        assert_eq!(a.audit_quality, Some(AuditQualityStatus::Complete));
        assert!(a.reason.is_none());

        let b = &index.pages[1];
        assert_eq!(b.status, PageStatus::Blocked);
        assert_eq!(b.reason.as_deref(), Some("the server answered HTTP 403"));
        assert!(b.file.is_none() && b.overall_score.is_none() && b.occurrence_count.is_none());

        assert_eq!(index.pages[2].occurrence_count, Some(0));

        let d = &index.pages[3];
        assert_eq!(d.status, PageStatus::Failed);
        assert_eq!(d.reason.as_deref(), Some("Page load timeout"));

        let e = &index.pages[4];
        assert_eq!(e.status, PageStatus::Failed);
        assert_eq!(e.reason.as_deref(), Some("no audit result recorded"));

        assert_eq!(index.totals.attempted, 5);
        assert_eq!(index.totals.ok, 2);
        assert_eq!(index.totals.blocked, 1);
        assert_eq!(index.totals.failed, 2);
        assert_eq!(index.totals.occurrences, all_rows.len());
    }

    #[test]
    fn jsonl_is_one_compact_object_per_line() {
        let report = report(
            "https://example.com/a",
            vec![violation("1.1.1", "img.a"), violation("1.1.1", "img.b")],
        );
        let normalized = normalize(&report).normalized;
        let rows = finding_rows(&report, &normalized);
        let text = to_jsonl(&rows).unwrap();
        assert!(text.ends_with('\n'));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(value["source"], "wcag");
            assert_eq!(value["severity"], "high");
            assert!(
                value["location"].is_null(),
                "absent fields are explicit nulls"
            );
        }
    }

    #[test]
    fn rows_are_canonical_english() {
        let report = report("https://example.com/a", vec![violation("1.1.1", "img.a")]);
        let normalized = normalize(&report).normalized;
        let text = to_jsonl(&finding_rows(&report, &normalized)).unwrap();
        assert!(
            !text.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
            "{text}"
        );
    }
}
