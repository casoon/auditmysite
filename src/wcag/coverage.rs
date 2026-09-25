//! WCAG criterion coverage manifest (issue #37).
//!
//! Explicit list of WCAG 2.2 criteria that this tool checks automatically vs.
//! those that fundamentally require behavioral testing. Surfaced in the report
//! so users understand the scope of automated audit.

use std::sync::OnceLock;

use crate::taxonomy::rules::RULES;

/// WCAG 2.2 A/AA totals — Level A: 31, Level AA: 24, sum 55. That is WCAG 2.1's
/// 50 criteria, minus 4.1.1 Parsing (removed in WCAG 2.2), plus the six
/// criteria 2.2 added (plan 54 §1).
///
/// The WCAG-2.1-scoped view the BFSG references lives in the EN 301 549
/// V3.2.1 appendix (`wcag::en301549`), which maps findings onto that standard's
/// chapter 9 clauses — it is derived from the same run, not a second audit.
pub const WCAG_AA_TOTAL: usize = 55;

/// The WCAG version the audit and its coverage ratio are scoped to. Written
/// into the JSON (`summary.wcag_coverage.wcag_version`) so reports from before
/// the switch to 2.2 — which lack the field and were scoped to 2.1 — stay
/// machine-distinguishable (plan 54 §4).
pub const WCAG_VERSION: &str = "2.2";

/// Success criteria WCAG 2.2 added that didn't exist in 2.1. They count toward
/// `coverage_stats()` like any other A/AA criterion now that the ratio is
/// scoped to WCAG 2.2; the list stays so output layers can mark them as new in
/// 2.2 — EN 301 549 V3.2.1 (and with it the BFSG appendix) is still built on
/// WCAG 2.1 and does not cover them.
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
    // Whether a dragging operation has a single-pointer alternative (buttons,
    // tap-to-select, a track click) is behaviour, not markup (plan 54 §3).
    ("2.5.7", "AA", "Dragging Movements"),
    ("3.2.3", "AA", "Consistent Navigation"),
    ("3.2.4", "AA", "Consistent Identification"),
    // Cross-page by nature: the batch report compares help mechanisms across
    // the audited set (`audit::batch_consistency`); a single page can't show
    // whether its help sits where the other pages put it.
    ("3.2.6", "A", "Consistent Help"),
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

/// Returns (automated_count, total_aa_criteria), scoped to WCAG 2.2's A/AA
/// criteria to match `WCAG_AA_TOTAL`.
pub fn coverage_stats() -> (usize, usize) {
    let aa_count = automated_criteria()
        .iter()
        .filter(|(_, l)| *l == "A" || *l == "AA")
        .count();
    (aa_count, WCAG_AA_TOTAL)
}

/// True if `id` is a WCAG 2.2-only success criterion (didn't exist in WCAG
/// 2.1) — lets output layers mark an automated criterion or a finding as new
/// in WCAG 2.2, instead of presenting it as an undifferentiated "AA"
/// criterion (#572). Those criteria are inside the `coverage_stats()` ratio
/// but outside EN 301 549 V3.2.1, which is still built on WCAG 2.1.
pub fn is_wcag22_only(id: &str) -> bool {
    WCAG_22_ONLY_CRITERIA.contains(&id)
}

/// German titles of the A/AA criteria WCAG 2.2 added, after the authorised
/// German translation of WCAG 2.2. `en301549::EN301549_WEB_CLAUSES` is built
/// on WCAG 2.1 and has none of them (plan 54 §4).
const WCAG_22_TITLES_DE: &[(&str, &str)] = &[
    ("2.4.11", "Fokus nicht verdeckt (Minimum)"),
    ("2.5.7", "Ziehbewegungen"),
    ("2.5.8", "Zielgröße (Minimum)"),
    ("3.2.6", "Konsistente Hilfe"),
    ("3.3.7", "Redundante Eingabe"),
    ("3.3.8", "Barrierefreie Authentifizierung (Minimum)"),
];

/// German title for a manual-review criterion, reused from the canonical
/// BIK/DIAS WCAG 2.1 translation table (`en301549::EN301549_WEB_CLAUSES`)
/// instead of maintaining a second, independently drifting translation
/// (#572); criteria new in WCAG 2.2 come from `WCAG_22_TITLES_DE`. Falls back
/// to `fallback_en` for an id in neither table.
pub fn manual_review_criterion_name_de(id: &str, fallback_en: &'static str) -> &'static str {
    crate::wcag::en301549::EN301549_WEB_CLAUSES
        .iter()
        .find(|c| c.wcag == id)
        .map(|c| c.title_de)
        .or_else(|| {
            WCAG_22_TITLES_DE
                .iter()
                .find(|(wcag, _)| *wcag == id)
                .map(|(_, title)| *title)
        })
        .unwrap_or(fallback_en)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 36 §1: the report's appendix reconciles its own totals — the A/AA
    /// count the ratio is scoped to plus the AAA criteria this tool also
    /// checks — against the full listing.
    #[test]
    fn the_appendix_groups_partition_the_automated_criteria() {
        let (scoped, _) = coverage_stats();
        let aaa = automated_criteria()
            .iter()
            .filter(|(_, level)| *level == "AAA")
            .count();

        assert_eq!(
            scoped + aaa,
            automated_criteria().len(),
            "A/AA {scoped} + AAA {aaa} must equal the {} listed",
            automated_criteria().len(),
        );
    }

    /// Plan 54 §1: the ratio is scoped to WCAG 2.2's 55 A/AA criteria, so no
    /// run may report more automated A/AA criteria than that standard has.
    #[test]
    fn the_automated_aa_count_stays_within_the_wcag22_catalogue() {
        let (scoped, total) = coverage_stats();
        assert_eq!(total, WCAG_AA_TOTAL);
        assert!(
            scoped <= total,
            "{scoped} automated A/AA criteria against a catalogue of {total}"
        );
    }

    /// Plan 54 §2: 4.1.1 Parsing was removed in WCAG 2.2. The duplicate-ID
    /// checks it used to carry now report 4.1.2, so no rule may claim it
    /// again — it would count against the 2.2 denominator as a criterion that
    /// no longer exists.
    #[test]
    fn parsing_is_no_longer_a_checked_criterion() {
        assert!(
            !automated_criteria().iter().any(|(id, _)| *id == "4.1.1"),
            "WCAG 4.1.1 was removed in WCAG 2.2 and must not be reported as checked"
        );
    }

    #[test]
    fn manual_review_criterion_name_de_has_a_real_translation_for_every_current_entry() {
        // #572 guard: the PDF's manual-review tag cloud used to render the
        // English criterion name unconditionally (no locale branching at
        // all). Every currently-listed manual-review criterion must resolve
        // to a German name distinct from its English fallback — this fails
        // loudly (instead of silently falling back to English) if a future
        // `MANUAL_REVIEW_CRITERIA_RAW` entry's id is missing from
        // `en301549::EN301549_WEB_CLAUSES`.
        for (id, _level, name_en) in manual_review_criteria() {
            let name_de = manual_review_criterion_name_de(id, name_en);
            assert_ne!(
                name_de, *name_en,
                "expected a German translation for WCAG {id}, got the English fallback"
            );
        }
    }

    #[test]
    fn manual_review_criterion_name_de_matches_known_translation() {
        assert_eq!(
            manual_review_criterion_name_de("1.2.1", "Audio-only and Video-only (Prerecorded)"),
            "Nur Audio oder nur Video (aufgezeichnet)"
        );
    }

    #[test]
    fn manual_review_criterion_name_de_falls_back_for_unknown_id() {
        assert_eq!(
            manual_review_criterion_name_de("9.9.9", "Unknown Criterion"),
            "Unknown Criterion"
        );
    }
}
