//! Performance interpretation — locale-neutral threshold decisions plus the
//! single localized text source (#406).
//!
//! These enrich the performance module's interpretation sentence: a gap note
//! ("score reduced by … although Core Web Vitals are green") and concrete
//! qualifiers naming weak metrics (DOM complexity, throttled LCP, render-blocking
//! resources, unminified weight, non-composited animations) plus strong metrics
//! (CLS, TBT) as contrast. The decision logic (`derive_performance_qualifiers`)
//! is pure; the wording lives only in the `*_text(.., en)` functions, so the
//! builder no longer carries hardcoded de/en branches. Numeric inputs
//! (`throttled_lcp_max`) and viewport flags are passed in, so this stays decoupled
//! from `AuditContext`.

use crate::audit::PerformanceResults;
use crate::i18n::{text, text_args};
use fluent_bundle::FluentValue;

/// Worst-case throttled LCP (ms) above which the interpretation prose names a
/// "late LCP under throttling" qualifier. Shared with the throttled-table
/// cross-reference footnote in `output::pdf::detail_modules::performance` so
/// both agree on when a worst-case LCP claim is being made and quoted.
pub const LATE_THROTTLED_LCP_THRESHOLD_MS: f64 = 4000.0;

/// A concrete weak metric worth naming alongside the score band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerfCriticalQualifier {
    /// DOM element count (> 3000).
    HighDomComplexity(i64),
    /// Worst LCP under network throttling, in ms (> 4000).
    LateThrottledLcp(f64),
    /// Render-blocking resource count (> 0).
    RenderBlocking(usize),
    /// Unminified asset savings in KB (> 100).
    UnminifiedAssets(f64),
    /// Non-composited animation count (> 10).
    NonCompositedAnimations(u32),
}

/// A genuinely strong metric acknowledged as contrast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfPositiveQualifier {
    LayoutStability,
    MainThreadBlocking,
}

/// Pure threshold decision: which weak/strong metrics to name. `throttled_lcp_max`
/// is the worst LCP measured across throttle profiles (0.0 if none).
pub fn derive_performance_qualifiers(
    p: &PerformanceResults,
    throttled_lcp_max: f64,
) -> (Vec<PerfCriticalQualifier>, Vec<PerfPositiveQualifier>) {
    let mut critical = Vec::new();

    if let Some(elements) = p.vitals.dom_elements.filter(|n| *n > 3000) {
        critical.push(PerfCriticalQualifier::HighDomComplexity(elements));
    }
    if throttled_lcp_max > LATE_THROTTLED_LCP_THRESHOLD_MS {
        critical.push(PerfCriticalQualifier::LateThrottledLcp(throttled_lcp_max));
    }
    let rb_count = p
        .render_blocking
        .as_ref()
        .map(|rb| rb.blocking_scripts.len() + rb.blocking_css.len())
        .unwrap_or(0);
    if rb_count > 0 {
        critical.push(PerfCriticalQualifier::RenderBlocking(rb_count));
    }
    let unminified_kb = p
        .minification
        .as_ref()
        .map(|m| m.total_savings_bytes as f64 / 1024.0)
        .unwrap_or(0.0);
    if unminified_kb > 100.0 {
        critical.push(PerfCriticalQualifier::UnminifiedAssets(unminified_kb));
    }
    let anim_count = p.animations.as_ref().map(|a| a.total_count).unwrap_or(0);
    if anim_count > 10 {
        critical.push(PerfCriticalQualifier::NonCompositedAnimations(anim_count));
    }

    let mut positive = Vec::new();
    if p.vitals.cls.as_ref().is_some_and(|v| v.rating == "good") {
        positive.push(PerfPositiveQualifier::LayoutStability);
    }
    if p.vitals.tbt.as_ref().is_some_and(|v| v.rating == "good") {
        positive.push(PerfPositiveQualifier::MainThreadBlocking);
    }

    (critical, positive)
}

fn critical_qualifier_text(q: PerfCriticalQualifier, en: bool) -> String {
    match q {
        PerfCriticalQualifier::HighDomComplexity(elements) => {
            // English prints the plain digits, German groups thousands.
            let elements = if en {
                elements.to_string()
            } else {
                crate::i18n::number::integer(elements, false)
            };
            text_args(
                "audit-interpretation-perf-dom",
                &[("elements", FluentValue::from(elements))],
                en,
            )
        }
        PerfCriticalQualifier::LateThrottledLcp(ms) => text_args(
            "audit-interpretation-perf-lcp",
            &[("ms", FluentValue::from(format!("{ms:.0}")))],
            en,
        ),
        PerfCriticalQualifier::RenderBlocking(n) => text_args(
            "audit-interpretation-perf-render-blocking",
            &[("count", FluentValue::from(n as i64))],
            en,
        ),
        PerfCriticalQualifier::UnminifiedAssets(kb) => text_args(
            "audit-interpretation-perf-unminified",
            &[("kb", FluentValue::from(format!("{kb:.0}")))],
            en,
        ),
        PerfCriticalQualifier::NonCompositedAnimations(n) => text_args(
            "audit-interpretation-perf-animations",
            &[("count", FluentValue::from(n.to_string()))],
            en,
        ),
    }
}

fn positive_qualifier_text(q: PerfPositiveQualifier, en: bool) -> String {
    text(
        match q {
            PerfPositiveQualifier::LayoutStability => "audit-interpretation-perf-positive-layout",
            PerfPositiveQualifier::MainThreadBlocking => {
                "audit-interpretation-perf-positive-main-thread"
            }
        },
        en,
    )
}

/// Joins phrases into a natural list ("a, b and c" / "a, b und c").
fn join_phrases(items: &[String], en: bool) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        _ => {
            let (last, head) = items.split_last().unwrap();
            format!("{}{}{}", head.join(", "), text("audit-join-and", en), last)
        }
    }
}

/// Appends threshold-based qualifiers to the performance interpretation so that
/// concrete weak metrics are named instead of left to the score band alone, with
/// genuinely strong metrics acknowledged as contrast (#367). Returns `base`
/// unchanged when no critical qualifier applies.
pub fn append_performance_qualifiers_text(
    base: String,
    p: &PerformanceResults,
    throttled_lcp_max: f64,
    en: bool,
) -> String {
    let (critical, positive) = derive_performance_qualifiers(p, throttled_lcp_max);
    if critical.is_empty() {
        return base;
    }

    let critical_phrases: Vec<String> = critical
        .iter()
        .map(|q| critical_qualifier_text(*q, en))
        .collect();
    let positive_phrases: Vec<String> = positive
        .iter()
        .map(|q| positive_qualifier_text(*q, en))
        .collect();

    let mut out = base;
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(&text_args(
        "audit-interpretation-perf-critical-here",
        &[(
            "phrases",
            FluentValue::from(join_phrases(&critical_phrases, en)),
        )],
        en,
    ));
    if !positive_phrases.is_empty() {
        out.push(' ');
        out.push_str(&text_args(
            "audit-interpretation-perf-contrast",
            &[
                ("count", FluentValue::from(positive_phrases.len() as i64)),
                (
                    "phrases",
                    FluentValue::from(join_phrases(&positive_phrases, en)),
                ),
            ],
            en,
        ));
    }
    out
}

/// True only when lab LCP and CLS are both measured and rated good; a missing
/// metric is unknown, not good. INP is not measured here (TBT is the proxy).
pub fn lab_lcp_cls_good(p: &PerformanceResults) -> bool {
    p.vitals.lcp.as_ref().is_some_and(|v| v.rating == "good")
        && p.vitals.cls.as_ref().is_some_and(|v| v.rating == "good")
}

/// When lab LCP and CLS are both measured and good but the score is below excellent, append the
/// cause for the gap. Returns `base` unchanged otherwise.
pub fn performance_gap_text(
    base: String,
    p: &PerformanceResults,
    lab_lcp_cls_good: bool,
    score_below_excellent: bool,
    has_render_blocking: bool,
    en: bool,
) -> String {
    if !(lab_lcp_cls_good && score_below_excellent) {
        return base;
    }
    let mut reasons = Vec::new();
    if p.vitals.dom_elements.is_some_and(|n| n > 1500) {
        reasons.push(text("audit-interpretation-perf-reason-dom", en));
    }
    if has_render_blocking {
        reasons.push(text("audit-interpretation-perf-reason-render-blocking", en));
    }
    if p.vitals.tbt.as_ref().is_some_and(|v| v.rating != "good") {
        reasons.push("Total Blocking Time".to_string());
    }
    if reasons.is_empty() {
        base
    } else {
        let mut out = text_args(
            "audit-interpretation-perf-gap",
            &[
                ("base", FluentValue::from(base)),
                ("reasons", FluentValue::from(reasons.join(", "))),
            ],
            en,
        );
        if p.vitals.inp.is_none() {
            out.push(' ');
            out.push_str(&text("audit-interpretation-perf-inp-not-measured", en));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_qualifier_text_is_localized_and_en_has_no_german_umlauts() {
        // #406 guard plus value interpolation, across every variant.
        let cases = [
            PerfCriticalQualifier::HighDomComplexity(4200),
            PerfCriticalQualifier::LateThrottledLcp(5200.0),
            PerfCriticalQualifier::RenderBlocking(7),
            PerfCriticalQualifier::UnminifiedAssets(180.0),
            PerfCriticalQualifier::NonCompositedAnimations(14),
        ];
        for q in cases {
            let en = critical_qualifier_text(q, true);
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN qualifier contains umlaut: {en}"
            );
        }
        assert!(
            critical_qualifier_text(PerfCriticalQualifier::HighDomComplexity(4200), true)
                .contains("4200")
        );
        assert!(
            critical_qualifier_text(PerfCriticalQualifier::HighDomComplexity(4200), false)
                .contains("Komplexität")
        );
    }

    #[test]
    fn missing_lcp_is_not_good_and_gap_text_makes_no_green_claim() {
        let vitals = crate::performance::WebVitals {
            cls: Some(crate::performance::VitalMetric::new(0.0, 0.1, 0.25)),
            ..Default::default()
        };
        let score = crate::performance::calculate_performance_score(&vitals, None);
        let mut p = PerformanceResults {
            vitals,
            score,
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: Vec::new(),
            runs: 1,
            stability: Default::default(),
        };
        assert!(!lab_lcp_cls_good(&p), "missing LCP must not count as good");
        p.vitals.lcp = Some(crate::performance::VitalMetric::new(1000.0, 2500.0, 4000.0));
        assert!(lab_lcp_cls_good(&p));
        p.vitals.dom_elements = Some(2000);
        for en in [true, false] {
            let t = performance_gap_text("Base.".to_string(), &p, true, true, false, en);
            assert!(!t.contains("grünen") && !t.contains("in the green"), "{t}");
            assert!(t.contains("INP"), "{t}");
            if en {
                assert!(!t.chars().any(|c| "äöüÄÖÜß".contains(c)), "{t}");
            }
        }
    }

    #[test]
    fn join_phrases_uses_locale_conjunction() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(join_phrases(&items, true), "a, b and c");
        assert_eq!(join_phrases(&items, false), "a, b und c");
        assert_eq!(join_phrases(&["solo".to_string()], true), "solo");
    }
}
