//! Modul-Scores des normalisierten Reports.
//!
//! Baut aus dem rohen `AuditReport` die `ModuleScoreEntry`-Liste, aus der
//! `normalize` den gewichteten `overall_score` ableitet. Die Gewichte selbst
//! stehen in `taxonomy::module_weight` und werden von
//! `ModuleScoreEntry::new` nachgeschlagen. Die Accessibility-Abzüge auf die
//! UX- und Journey-Indikatoren liegen hier, weil sie nur beim Aufbau der
//! Einträge angewandt werden.

use crate::audit::normalized::{ModuleScoreEntry, SeverityCounts};
use crate::audit::report::AuditReport;

pub(super) fn build_module_scores(
    report: &AuditReport,
    accessibility_score: u32,
    occurrence_counts: &SeverityCounts,
    vuln_security_penalty: u32,
) -> Vec<ModuleScoreEntry> {
    let mut module_scores = Vec::new();

    module_scores.push(ModuleScoreEntry::new(
        "Accessibility",
        accessibility_score,
        "measured",
        true,
    ));

    if let Some(ref perf) = report.performance {
        // No Core Web Vitals could be measured (e.g. collection failed) — a
        // score of 0 here means "not measured", not "unusably slow". Exclude
        // it from the weighted overall score rather than tanking it (#QA-023).
        let measured = perf.score.metrics_available > 0;
        module_scores.push(ModuleScoreEntry::new(
            "Performance",
            perf.score.overall,
            if measured { "measured" } else { "not_measured" },
            measured,
        ));
    }
    if let Some(ref seo) = report.discoverability.seo {
        module_scores.push(ModuleScoreEntry::new("SEO", seo.score, "measured", true));
    }
    if let Some(ref sec) = report.security {
        let adjusted = sec.score.saturating_sub(vuln_security_penalty);
        module_scores.push(ModuleScoreEntry::new(
            "Security", adjusted, "measured", true,
        ));
    }
    if let Some(ref mob) = report.experience.mobile {
        module_scores.push(ModuleScoreEntry::new("Mobile", mob.score, "measured", true));
    }
    if let Some(ref hc) = report.html_conform {
        // Weighted since the score became trustworthy again (it is now charged
        // per *distinct* defect, not per occurrence — see
        // `html_conform::score_findings`). Two separate defects had made the
        // old number meaningless and kept this module at weight 0:
        //
        // 1. The vendored HTML5 schema had no RDFa/Open-Graph awareness, so
        //    every `<meta property="og:...">` counted as an error. Closed
        //    upstream in html-conform 0.2.1 (`schema/html5/meta.rnc`, "RDFa
        //    Lite Property Metadata"); verified directly against a page
        //    carrying `og:` tags, which now produces no such finding.
        // 2. One faulty element in a template emitted one finding per render,
        //    and the old `error_count * 10` penalty charged each one, flooring
        //    any real-world page at 0 regardless of how few things were
        //    actually wrong.
        //
        // `measurement_type` is "measured" accordingly: schema conformance is
        // a checkable property of the document, not a heuristic reading of it.
        module_scores.push(ModuleScoreEntry::new(
            "HTML Conformance",
            hc.score,
            "measured",
            true,
        ));
    }
    if let Some(ref ux) = report.ux {
        // Accessibility flows into UX: critical a11y issues penalize UX score
        // Rationale: for users with disabilities, Accessibility IS the UX.
        // Penalty thresholds reflect total affected elements, not distinct rules.
        let a11y_penalty = ux_a11y_penalty(occurrence_counts);
        module_scores.push(ModuleScoreEntry::new(
            "UX",
            ux.score.saturating_sub(a11y_penalty),
            "heuristic",
            false,
        ));
    }
    if let Some(ref journey) = report.journey {
        // Journey also gets a11y penalty — inaccessible journeys are broken journeys.
        // Threshold uses occurrence-level severity, not finding count.
        let a11y_penalty = journey_a11y_penalty(occurrence_counts);
        module_scores.push(ModuleScoreEntry::new(
            "Journey",
            journey.score.saturating_sub(a11y_penalty),
            "heuristic",
            false,
        ));
    }
    if let Some(ref bp) = report.best_practices {
        module_scores.push(ModuleScoreEntry::new(
            "Best Practices",
            bp.score,
            "measured",
            false,
        ));
    }

    // Indicator modules that compute a 0–100 score but do not feed the overall
    // score. Previously their score was serialized raw with no grade and no
    // entry here, so the report showed a bare number with no relation to the
    // rest (#447). They appear here consistently as non-contributing
    // indicators — with a band word rather than a letter grade, because a
    // grade on a module of weight 0 claimed an authority it does not have
    // (plan 29, D2).
    if let Some(ref dm) = report.experience.dark_mode {
        // Dark Mode is an optional/non-normative product feature, not a WCAG
        // conformance criterion — distinct from the "heuristic" indicators
        // below (#577).
        module_scores.push(ModuleScoreEntry::new(
            "Dark Mode",
            dm.score,
            "optional",
            false,
        ));
    }
    if let Some(ref ai) = report.discoverability.ai_visibility {
        module_scores.push(ModuleScoreEntry::new(
            "AI Visibility",
            ai.score,
            "heuristic",
            false,
        ));
    }
    if let Some(ref sq) = report.discoverability.source_quality {
        module_scores.push(ModuleScoreEntry::new(
            "Source Quality",
            sq.score,
            "heuristic",
            false,
        ));
    }
    // Content Visibility carries no 0-100 score: it used to be
    // "share of checks that did not complain", so adding a check that usually
    // passes raised every site's value (plan 43 §3, resolved by plan 29 D4).
    // The counts are reported in the module payload instead.

    // Tech Stack is detection-only — no score in module_scores.
    // Stack-specific security findings (WordPress admin exposure, etc.) flow
    // into the Security module score instead.

    module_scores
}

/// Points the Accessibility result costs the UX indicator.
///
/// Split out of `build_module_scores` so the applied penalty can be reported
/// next to the score instead of being folded in silently (plan 42 §2).
pub(crate) fn ux_a11y_penalty(occurrence_counts: &SeverityCounts) -> u32 {
    let critical = occurrence_counts.critical;
    let high = occurrence_counts.high;
    if critical >= 10 {
        25 // severe: many critical barriers
    } else if critical >= 5 {
        15
    } else if critical > 0 {
        10
    } else if high >= 5 {
        5
    } else {
        0
    }
}

/// Points the Accessibility result costs the Journey indicator. See
/// [`ux_a11y_penalty`].
pub(crate) fn journey_a11y_penalty(occurrence_counts: &SeverityCounts) -> u32 {
    let critical = occurrence_counts.critical;
    if critical >= 10 {
        20
    } else if critical >= 5 {
        10
    } else if critical > 0 {
        5
    } else {
        0
    }
}
