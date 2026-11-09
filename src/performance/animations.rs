//! Non-composited animation detection (#105).
//!
//! Inspects all page stylesheets for CSS `@keyframes` rules and `transition`
//! declarations that animate non-compositable properties.  Animating anything
//! other than `transform` and `opacity` triggers layout or paint work on every
//! frame and can degrade performance significantly.
//!
//! Detection strategy (JS-based, no extra CDP round-trip):
//! 1. Iterate `document.styleSheets` (skips cross-origin sheets with a try/catch).
//! 2. For every `CSSKeyframesRule`, check each keyframe's `style` for non-composited
//!    properties.
//! 3. For every `CSSStyleRule`, parse its `transition` / `-webkit-transition` value
//!    and check the named property.
//! 4. Also check `animation` shorthand for references to known-slow timing functions
//!    (not the main concern but useful context).

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::util::truncate_ellipsis as truncate;

/// A single non-composited animation or transition finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompositedAnimation {
    /// "keyframe" or "transition"
    pub kind: String,
    /// For keyframes: the `@keyframes` name.  For transitions: the CSS selector.
    pub name: String,
    /// The property being animated (e.g. "top", "width")
    pub property: String,
    /// Source stylesheet URL or "inline" for `<style>` elements
    pub source: String,
}

/// Results of the non-composited animation analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationAnalysis {
    /// Individual findings
    pub findings: Vec<NonCompositedAnimation>,
    /// Total number of non-composited animations/transitions detected
    pub total_count: u32,
    /// Distinct non-composited properties observed
    pub affected_properties: Vec<String>,
}

/// Detect non-composited CSS animations and transitions on the loaded page.
pub async fn analyze_non_composited_animations(
    page: &Page,
) -> crate::audit::collection::Collected<AnimationAnalysis> {
    info!("Analyzing non-composited CSS animations...");

    // Properties that trigger layout (expensive) when animated.
    // `transform` and `opacity` are the only truly composited properties in Chrome.
    let js = include_str!("js/animations.js");
    let raw: Vec<RawFinding> = match crate::audit::collection::evaluate_js(page, js)
        .await
        .into_measurement()
    {
        Ok(v) => v,
        Err(failure) => return failure,
    };

    let findings: Vec<NonCompositedAnimation> = raw
        .into_iter()
        .map(|r| NonCompositedAnimation {
            kind: r.kind,
            name: truncate(&r.name, 80),
            property: r.property,
            source: truncate(&r.source, 120),
        })
        .collect();

    let mut affected_properties: Vec<String> = findings
        .iter()
        .map(|f| f.property.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    affected_properties.sort();

    let total_count = findings.len() as u32;

    info!(
        "Non-composited animations: {} findings across {} properties",
        total_count,
        affected_properties.len()
    );

    crate::audit::collection::Collected::Measured(AnimationAnalysis {
        findings,
        total_count,
        affected_properties,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct RawFinding {
    kind: String,
    name: String,
    property: String,
    source: String,
}

/// Counter-cases of this collector (#769): an unread style scan is no page without animations.
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<Vec<RawFinding>>(serde_json::json!([]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 80), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let s = "a".repeat(200);
        let r = truncate(&s, 80);
        assert!(r.len() <= 83);
        assert!(r.ends_with('…'));
    }
}
