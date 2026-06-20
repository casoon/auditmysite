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

/// A concrete weak metric worth naming alongside the score band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerfCriticalQualifier {
    /// DOM node count (> 3000).
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

    if let Some(nodes) = p.vitals.dom_nodes.filter(|n| *n > 3000) {
        critical.push(PerfCriticalQualifier::HighDomComplexity(nodes));
    }
    if throttled_lcp_max > 4000.0 {
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
        PerfCriticalQualifier::HighDomComplexity(nodes) => {
            if en {
                format!("high DOM complexity ({nodes} nodes)")
            } else {
                format!("die hohe DOM-Komplexität ({nodes} Knoten)")
            }
        }
        PerfCriticalQualifier::LateThrottledLcp(ms) => {
            if en {
                format!("a late largest contentful paint under throttling ({ms:.0} ms)")
            } else {
                format!("der spät erscheinende Hauptinhalt unter Drosselung ({ms:.0} ms LCP)")
            }
        }
        PerfCriticalQualifier::RenderBlocking(n) => {
            if en {
                format!("{n} render-blocking resources")
            } else {
                format!("{n} render-blockierende Ressourcen")
            }
        }
        PerfCriticalQualifier::UnminifiedAssets(kb) => {
            if en {
                format!("unminified assets (~{kb:.0} KB savings)")
            } else {
                format!("unminifizierte Assets (~{kb:.0} KB Einsparpotenzial)")
            }
        }
        PerfCriticalQualifier::NonCompositedAnimations(n) => {
            if en {
                format!("{n} non-composited animations")
            } else {
                format!("{n} nicht-composited Animationen")
            }
        }
    }
}

fn positive_qualifier_text(q: PerfPositiveQualifier, en: bool) -> &'static str {
    match q {
        PerfPositiveQualifier::LayoutStability => {
            if en {
                "layout stability (CLS)"
            } else {
                "Layout-Stabilität (CLS)"
            }
        }
        PerfPositiveQualifier::MainThreadBlocking => {
            if en {
                "main-thread blocking (TBT)"
            } else {
                "Hauptthread-Blockierung (TBT)"
            }
        }
    }
}

/// Joins phrases into a natural list ("a, b and c" / "a, b und c").
fn join_phrases(items: &[String], en: bool) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        _ => {
            let (last, head) = items.split_last().unwrap();
            format!(
                "{} {} {}",
                head.join(", "),
                if en { "and" } else { "und" },
                last
            )
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
        .map(|q| positive_qualifier_text(*q, en).to_string())
        .collect();

    let mut out = base;
    if !out.is_empty() {
        out.push(' ');
    }
    if en {
        out.push_str(&format!(
            "Critical here are {}.",
            join_phrases(&critical_phrases, en)
        ));
        if !positive_phrases.is_empty() {
            let verb = if positive_phrases.len() == 1 {
                "remains"
            } else {
                "remain"
            };
            out.push_str(&format!(
                " In contrast, {} {verb} unobtrusive.",
                join_phrases(&positive_phrases, en)
            ));
        }
    } else {
        out.push_str(&format!(
            "Kritisch sind hier {}.",
            join_phrases(&critical_phrases, en)
        ));
        if !positive_phrases.is_empty() {
            let verb = if positive_phrases.len() == 1 {
                "bleibt"
            } else {
                "bleiben"
            };
            out.push_str(&format!(
                " Im Kontrast dazu {verb} {} unauffällig.",
                join_phrases(&positive_phrases, en)
            ));
        }
    }
    out
}

/// When all core vitals are green but the score is below excellent, append the
/// cause for the gap. Returns `base` unchanged otherwise.
pub fn performance_gap_text(
    base: String,
    p: &PerformanceResults,
    cwv_all_good: bool,
    score_below_excellent: bool,
    has_render_blocking: bool,
    en: bool,
) -> String {
    if !(cwv_all_good && score_below_excellent) {
        return base;
    }
    let mut reasons = Vec::new();
    if p.vitals.dom_nodes.is_some_and(|n| n > 1500) {
        reasons.push(if en { "DOM size" } else { "DOM-Größe" });
    }
    if has_render_blocking {
        reasons.push(if en {
            "render-blocking resources"
        } else {
            "Render-blockierende Ressourcen"
        });
    }
    if p.vitals.tbt.as_ref().is_some_and(|v| v.rating != "good") {
        reasons.push("Total Blocking Time");
    }
    if reasons.is_empty() {
        base
    } else if en {
        format!(
            "{} Score reduced by {} although Core Web Vitals are in the green.",
            base,
            reasons.join(", ")
        )
    } else {
        format!(
            "{} Score durch {} reduziert, obwohl Core Web Vitals im grünen Bereich liegen.",
            base,
            reasons.join(", ")
        )
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
    fn join_phrases_uses_locale_conjunction() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(join_phrases(&items, true), "a, b and c");
        assert_eq!(join_phrases(&items, false), "a, b und c");
        assert_eq!(join_phrases(&["solo".to_string()], true), "solo");
    }
}
