//! WCAG criterion coverage manifest (issue #37).
//!
//! Explicit list of WCAG 2.1 / 2.2 criteria that this tool checks
//! automatically vs. those that fundamentally require behavioral testing.
//! Surfaced in the report so users understand the scope of automated audit.

use std::sync::OnceLock;

use crate::taxonomy::rules::RULES;

/// WCAG 2.1 AA totals — Level A: 30, Level AA: 20, sum 50.
/// WCAG 2.2 adds 6 more at AA (the 2.2.x criteria).
pub const WCAG_AA_TOTAL: usize = 50;

/// Success criteria WCAG 2.2 added that didn't exist in 2.1 — excluded from
/// `coverage_stats()`'s ratio since that's explicitly scoped to `WCAG_AA_TOTAL`'s
/// 2.1-only 50 criteria; counting a 2.2-only rule (e.g. 2.5.8) against that
/// denominator would overstate the 2.1 coverage ratio. Still included in
/// `automated_criteria()`'s full listing (e.g. the PDF's "automatically checked"
/// tag cloud), which isn't scoped to 2.1 alone.
const WCAG_22_ONLY_CRITERIA: &[&str] = &[
    "2.4.11", "2.4.12", "2.4.13", "2.5.7", "2.5.8", "3.2.6", "3.3.7", "3.3.8", "3.3.9",
];

/// Sort key for WCAG criterion ids ("2.5.10" must sort after "2.5.9", not
/// before it, so this can't be a plain string compare).
fn wcag_id_order(id: &str) -> Vec<u32> {
    id.split('.').filter_map(|p| p.parse().ok()).collect()
}

/// WCAG criteria with at least one automated rule in this tool, derived from
/// the rule catalog's own `external_ref`/`external_level` (#QA-038) instead
/// of a hand-maintained duplicate list — that list had drifted from the real
/// catalog (under-counting by ~20 criteria) and mislabeled several
/// implemented criteria (e.g. 2.5.1/2.5.2/2.5.4) as manual-review-only.
pub fn automated_criteria() -> &'static [(&'static str, &'static str)] {
    static CACHE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out: Vec<(&'static str, &'static str)> = Vec::new();
        for rule in RULES {
            let (Some(ext_ref), Some(level)) = (rule.external_ref, rule.external_level) else {
                continue;
            };
            let Some(id) = ext_ref.strip_prefix("WCAG ") else {
                continue;
            };
            if !out.iter().any(|(existing, _)| *existing == id) {
                out.push((id, level));
            }
        }
        out.sort_by_key(|a| wcag_id_order(a.0));
        out
    })
}

/// Candidate WCAG criteria that fundamentally require behavioral / manual
/// review. Filtered against `automated_criteria()` at read time (see
/// `manual_review_criteria`) so a criterion that gains automated coverage —
/// or turns out to already have it, as 2.5.1/2.5.2/2.5.4 did — can't stay
/// stuck here through a forgotten manual edit.
const MANUAL_REVIEW_CRITERIA_RAW: &[(&str, &str, &str)] = &[
    ("1.2.1", "A", "Audio-only and Video-only (Prerecorded)"),
    ("1.2.2", "A", "Captions (Prerecorded)"),
    ("1.2.3", "A", "Audio Description or Media Alternative"),
    ("1.2.5", "AA", "Audio Description (Prerecorded)"),
    ("1.4.2", "A", "Audio Control"),
    ("1.4.5", "AA", "Images of Text"),
    ("2.1.4", "A", "Character Key Shortcuts"),
    ("2.2.2", "A", "Pause, Stop, Hide"),
    ("2.3.1", "A", "Three Flashes or Below Threshold"),
    ("2.5.1", "A", "Pointer Gestures"),
    ("2.5.2", "A", "Pointer Cancellation"),
    ("2.5.4", "A", "Motion Actuation"),
    ("3.2.3", "AA", "Consistent Navigation"),
    ("3.2.4", "AA", "Consistent Identification"),
    ("3.3.3", "AA", "Error Suggestion"),
    ("3.3.4", "AA", "Error Prevention (Legal, Financial, Data)"),
];

/// WCAG criteria that fundamentally require behavioral / manual review and
/// cannot be reliably verified by an automated tool.
pub fn manual_review_criteria() -> &'static [(&'static str, &'static str, &'static str)] {
    static CACHE: OnceLock<Vec<(&'static str, &'static str, &'static str)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let automated = automated_criteria();
        MANUAL_REVIEW_CRITERIA_RAW
            .iter()
            .filter(|(id, _, _)| !automated.iter().any(|(aid, _)| aid == id))
            .copied()
            .collect()
    })
}

/// Returns (automated_count, total_aa_criteria), scoped to WCAG 2.1's A/AA
/// criteria to match `WCAG_AA_TOTAL`.
pub fn coverage_stats() -> (usize, usize) {
    let aa_count = automated_criteria()
        .iter()
        .filter(|(id, l)| (*l == "A" || *l == "AA") && !WCAG_22_ONLY_CRITERIA.contains(id))
        .count();
    (aa_count, WCAG_AA_TOTAL)
}
