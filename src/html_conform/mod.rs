//! HTML5 spec-conformance checking via the `html-conform` crate.
//!
//! Wraps `html_conform::check` (browser-style HTML5 tree construction, full
//! RelaxNG schema validation, Schematron co-constraints, import-map/
//! speculation-rules JSON validation, CSP enforcement) — a much deeper check
//! than the existing crude `html5ever`-parse-errors-only validator in
//! `seo::page_health` (left untouched, see module structure docs).
//!
//! `rule_id`/`message` are stored as opaque canonical-English payload —
//! `html-conform`'s rule set is open-ended (not a small closed enum) and its
//! messages are third-party English prose, the same shape as
//! `best_practices::console_errors`/`vulnerable_libs`, so this module
//! deliberately does not use the #406 kind-enum localization pattern.

pub mod module;
pub use module::HtmlConformModule;

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::{AuditError, Result};

/// Complete HTML5 conformance analysis for a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlConformAnalysis {
    /// Conformance score (0-100). Only meaningful when `checked` is true.
    pub score: u32,
    /// Whether the check actually ran (HTML extraction + `html_conform::check`
    /// both succeeded). `false` means "not measured" — excluded from the
    /// weighted overall score, not counted as a score of 0.
    pub checked: bool,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    /// How many *distinct* defects the counts above represent (see
    /// `defect_key`). One bad component in a template emits one finding
    /// per render, so `error_count` alone reads as a much larger problem
    /// than it is; this is the number of things actually to fix, and what
    /// the score is charged against.
    pub distinct_defect_count: u32,
    pub findings: Vec<HtmlConformFinding>,
    /// The raw document HTML the check ran against, kept for the
    /// `html_content_model` WCAG rule (#579) to re-scan for enclosing tag
    /// context — `html-conform`'s content-model findings carry no parent
    /// element info, only a source location. Not serialized: it's derived
    /// at analysis time and would bloat the JSON report for no reader
    /// benefit, the same rationale as skipping raw screenshot bytes
    /// elsewhere in this codebase.
    #[serde(skip)]
    pub raw_html: Option<String>,
}

/// A single HTML5 conformance finding, passthrough from `html_conform::Finding`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlConformFinding {
    /// Stable rule identifier from the checker (e.g. "schema.html5", "parser.html5").
    pub rule_id: String,
    /// Lowercased severity: "error" / "warning" / "info".
    pub severity: String,
    /// Human-readable, canonical-English explanation authored by the checker.
    pub message: String,
    /// "line:column" source location, when the checker could establish one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Zero-based byte offset into the checked HTML, when the checker could
    /// establish one. Used by the `html_content_model` WCAG rule (#579) to
    /// locate the finding's enclosing tag stack; not exposed via `location`
    /// (a formatted "line:column" string) since that would require
    /// re-parsing text back into a number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<usize>,
}

/// Runs `html_conform::check` against the live page's rendered HTML.
///
/// On a technical setup failure (`CheckError`) or HTML-extraction failure,
/// logs a warning and returns `checked: false` / `score: 100` rather than a
/// punitive 0 — mirrors Performance's `metrics_available == 0` "not measured"
/// treatment.
pub async fn analyze_html_conform(page: &Page) -> Result<HtmlConformAnalysis> {
    let html = extract_document_html(page).await?;

    let report = match html_conform::check(&html) {
        Ok(report) => report,
        Err(e) => {
            warn!("HTML conformance check setup failed: {}", e);
            return Ok(not_measured());
        }
    };

    let mut error_count = 0u32;
    let mut warning_count = 0u32;
    let mut info_count = 0u32;
    let findings: Vec<HtmlConformFinding> = report
        .findings
        .into_iter()
        .map(|f| {
            let severity = match f.severity {
                html_conform::Severity::Error => {
                    error_count += 1;
                    "error"
                }
                html_conform::Severity::Warning => {
                    warning_count += 1;
                    "warning"
                }
                html_conform::Severity::Info => {
                    info_count += 1;
                    "info"
                }
            };
            HtmlConformFinding {
                rule_id: f.rule_id,
                severity: severity.to_string(),
                message: f.message,
                location: f.location.map(|l| l.to_string()),
                byte_offset: f.location.map(|l| l.byte_offset),
            }
        })
        .collect();

    let score = score_findings(&findings);
    let distinct_defect_count = distinct_defect_count(&findings);

    Ok(HtmlConformAnalysis {
        score,
        checked: true,
        error_count,
        warning_count,
        info_count,
        distinct_defect_count,
        findings,
        raw_html: Some(html),
    })
}

/// Severity weights for one *distinct* defect (see [`score_findings`]).
const ERROR_WEIGHT: u32 = 10;
const WARNING_WEIGHT: u32 = 4;
const INFO_WEIGHT: u32 = 1;

/// Collapses a finding into the defect it reports, so the same defect found
/// in N places counts as one.
///
/// `html-conform`'s messages embed the offending value and the source
/// position (`invalid value `100%` for attribute `height` at 2:29748`), both
/// of which differ per occurrence while the defect is identical. Backtick
/// runs and the trailing ` at line:column` are therefore stripped; what
/// remains is the rule plus the message's invariant prose.
pub(crate) fn defect_key(finding: &HtmlConformFinding) -> String {
    let message = finding
        .message
        .split(" at ")
        .next()
        .unwrap_or(&finding.message);
    let mut normalized = String::with_capacity(message.len());
    let mut in_quotes = false;
    for ch in message.chars() {
        match ch {
            '`' if in_quotes => in_quotes = false,
            '`' => {
                in_quotes = true;
                normalized.push('*');
            }
            _ if in_quotes => {}
            _ => normalized.push(ch),
        }
    }
    format!("{}|{}", finding.rule_id, normalized.trim())
}

/// Penalty for one distinct defect before rank damping: its severity weight
/// plus a capped surcharge for how widely it occurs.
///
/// Repetition is a wider blast radius, not a second defect — a component
/// rendered twelve times is still one fix. The surcharge never exceeds the
/// base weight, so twelve occurrences cost at most twice what one does,
/// instead of twelve times (the earlier per-occurrence penalty floored
/// every templated site at 0).
fn defect_cost(weight: u32, occurrences: u32) -> u32 {
    let surcharge = match occurrences {
        0..=1 => 0,
        2..=4 => weight / 4,
        5..=19 => weight / 2,
        _ => weight,
    };
    weight + surcharge
}

/// How many distinct defects `findings` represents — the finding count with
/// per-occurrence repetition collapsed away.
fn distinct_defect_count(findings: &[HtmlConformFinding]) -> u32 {
    let unique: std::collections::HashSet<String> = findings.iter().map(defect_key).collect();
    unique.len() as u32
}

/// Conformance score (0-100) from deduplicated findings.
///
/// Two-stage, both deliberate:
///
/// 1. **Deduplicate** by [`defect_key`] — the raw finding count is an
///    occurrence count, not a defect count, and charging per occurrence made
///    the score a constant 0 for any real page (one bad component in a
///    template emits one finding per render).
/// 2. **Damp by rank** — distinct defects sorted most-expensive first, the
///    first charged in full, the next two at half, the rest at a quarter.
///    A page's first real defect should move the score meaningfully; its
///    fifteenth should not have to, since the score has to stay informative
///    across the whole range rather than saturating at 0 (spec conformance
///    on real-world sites is a long tail — vnu finds a dozen distinct
///    defects on most commercial pages).
fn score_findings(findings: &[HtmlConformFinding]) -> u32 {
    use std::collections::HashMap;

    let mut per_defect: HashMap<String, (u32, u32)> = HashMap::new();
    for finding in findings {
        let weight = match finding.severity.as_str() {
            "error" => ERROR_WEIGHT,
            "warning" => WARNING_WEIGHT,
            _ => INFO_WEIGHT,
        };
        let entry = per_defect.entry(defect_key(finding)).or_insert((weight, 0));
        entry.1 += 1;
    }

    let mut costs: Vec<u32> = per_defect
        .into_values()
        .map(|(weight, occurrences)| defect_cost(weight, occurrences))
        .collect();
    costs.sort_unstable_by(|a, b| b.cmp(a));

    let penalty: u32 = costs
        .into_iter()
        .enumerate()
        .map(|(rank, cost)| match rank {
            0 => cost,
            1..=2 => cost / 2,
            _ => cost / 4,
        })
        .sum();

    100u32.saturating_sub(penalty.min(100))
}

fn not_measured() -> HtmlConformAnalysis {
    HtmlConformAnalysis {
        score: 100,
        checked: false,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        distinct_defect_count: 0,
        findings: Vec::new(),
        raw_html: None,
    }
}

/// Extracts the live document's HTML (doctype + `documentElement.outerHTML`)
/// via CDP. Deliberate small duplication of
/// `seo::page_health::extract_document_html` — sharing it would mean
/// exporting it out of `page_health` and threading a new `raw_html` field
/// through `ModuleContext`/`SnapshotData` for a ~15-line CDP call.
async fn extract_document_html(page: &Page) -> Result<String> {
    let js = r#"
    (() => {
        const d = document.doctype;
        const doctype = d
            ? `<!DOCTYPE ${d.name}${d.publicId ? ` PUBLIC "${d.publicId}"` : ''}${d.systemId ? ` "${d.systemId}"` : ''}>`
            : '<!DOCTYPE html>';
        return doctype + '\n' + document.documentElement.outerHTML;
    })()
    "#;

    let result = page.evaluate(js).await.map_err(|e| {
        AuditError::CdpError(format!(
            "HTML extraction for conformance check failed: {}",
            e
        ))
    })?;

    result
        .value()
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            AuditError::CdpError(
                "HTML extraction for conformance check returned no value".to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(rule: &str, severity: &str, message: &str) -> HtmlConformFinding {
        HtmlConformFinding {
            rule_id: rule.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            location: None,
            byte_offset: None,
        }
    }

    #[test]
    fn defect_key_collapses_value_and_position_variation() {
        let a = finding(
            "schema.html5",
            "error",
            "invalid value `100%` for attribute `height` at 2:29748",
        );
        let b = finding(
            "schema.html5",
            "error",
            "invalid value `150px` for attribute `height` at 9:41",
        );
        assert_eq!(defect_key(&a), defect_key(&b));
    }

    #[test]
    fn defect_key_keeps_genuinely_different_defects_apart() {
        let height = finding(
            "schema.html5",
            "error",
            "invalid value `100%` for attribute `height`",
        );
        let element = finding("schema.html5", "error", "unexpected element `div`");
        let other_rule = finding(
            "parser.html5",
            "error",
            "invalid value `100%` for attribute `height`",
        );
        assert_ne!(defect_key(&height), defect_key(&element));
        assert_ne!(defect_key(&height), defect_key(&other_rule));
    }

    #[test]
    fn repetition_of_one_defect_does_not_floor_the_score() {
        // The regression this scoring exists for: one bad component rendered
        // twelve times used to cost 120 penalty points and score 0.
        let findings: Vec<_> = (0..12)
            .map(|i| {
                finding(
                    "assertion.elements.linkas-missing-rel",
                    "error",
                    &format!("A `link` element with `as` must have `rel` at 2:{i}"),
                )
            })
            .collect();
        assert_eq!(distinct_defect_count(&findings), 1);
        // One distinct error at full rank: 10 + surcharge 5 = 15.
        assert_eq!(score_findings(&findings), 85);
    }

    #[test]
    fn distinct_defects_are_damped_by_rank() {
        let findings: Vec<_> = (0..4)
            .map(|i| finding("schema.html5", "error", &format!("defect number {i}")))
            .collect();
        assert_eq!(distinct_defect_count(&findings), 4);
        // 10 (full) + 5 + 5 (half) + 2 (quarter) = 22.
        assert_eq!(score_findings(&findings), 78);
    }

    #[test]
    fn a_single_error_costs_its_full_weight() {
        let findings = vec![finding("schema.html5", "error", "one defect")];
        assert_eq!(score_findings(&findings), 100 - ERROR_WEIGHT);
    }

    #[test]
    fn severity_is_weighted() {
        let error = vec![finding("r", "error", "m")];
        let warning = vec![finding("r", "warning", "m")];
        let info = vec![finding("r", "info", "m")];
        assert!(score_findings(&error) < score_findings(&warning));
        assert!(score_findings(&warning) < score_findings(&info));
    }

    #[test]
    fn clean_page_scores_full_marks() {
        assert_eq!(score_findings(&[]), 100);
        assert_eq!(distinct_defect_count(&[]), 0);
    }

    #[test]
    fn score_floors_at_zero() {
        let findings: Vec<_> = (0..200)
            .map(|i| finding("schema.html5", "error", &format!("defect number {i}")))
            .collect();
        assert_eq!(score_findings(&findings), 0);
    }

    #[test]
    fn not_measured_has_full_score_and_is_excluded() {
        let a = not_measured();
        assert!(!a.checked);
        assert_eq!(a.score, 100);
        assert!(a.findings.is_empty());
        assert_eq!(a.distinct_defect_count, 0);
    }
}
