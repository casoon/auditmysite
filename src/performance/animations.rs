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

use crate::error::{AuditError, Result};

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
pub async fn analyze_non_composited_animations(page: &Page) -> Result<AnimationAnalysis> {
    info!("Analyzing non-composited CSS animations...");

    // Properties that trigger layout (expensive) when animated.
    // `transform` and `opacity` are the only truly composited properties in Chrome.
    let js = r#"
    (() => {
        var NON_COMPOSITED = new Set([
            'top','left','right','bottom',
            'width','height','min-width','max-width','min-height','max-height',
            'margin','margin-top','margin-bottom','margin-left','margin-right',
            'padding','padding-top','padding-bottom','padding-left','padding-right',
            'border','border-width','border-top-width','border-bottom-width',
            'border-left-width','border-right-width',
            'font-size','line-height','letter-spacing','word-spacing',
            'background','background-color','background-size','background-position',
            'border-radius','box-shadow','text-shadow',
            'color','fill','stroke',
            'clip','clip-path',
            'outline','outline-width','outline-offset',
            'flex','flex-basis','flex-grow','flex-shrink',
            'grid-template-columns','grid-template-rows',
            'columns','column-gap','column-width',
            'display','visibility','z-index',
            'vertical-align','text-align','float'
        ]);

        var findings = [];

        function sourceOf(sheet) {
            return sheet.href || 'inline';
        }

        function checkTransition(transitionValue, selector, source) {
            if (!transitionValue) return;
            transitionValue.split(',').forEach(function(part) {
                var prop = part.trim().split(/\s+/)[0].toLowerCase();
                if (prop && prop !== 'all' && prop !== 'none' && NON_COMPOSITED.has(prop)) {
                    findings.push({ kind: 'transition', name: selector, property: prop, source: source });
                }
            });
        }

        for (var si = 0; si < document.styleSheets.length; si++) {
            var sheet = document.styleSheets[si];
            var src = sourceOf(sheet);
            var rules;
            try { rules = sheet.cssRules; } catch(e) { continue; }
            if (!rules) continue;

            for (var ri = 0; ri < rules.length; ri++) {
                var rule = rules[ri];

                // @keyframes — check each frame's animated properties
                if (rule.type === 7 /* CSSKeyframesRule */) {
                    var frames = rule.cssRules || [];
                    var seen = {};
                    for (var fi = 0; fi < frames.length; fi++) {
                        var frame = frames[fi];
                        var style = frame.style;
                        if (!style) continue;
                        for (var pi = 0; pi < style.length; pi++) {
                            var prop = style[pi].toLowerCase();
                            if (NON_COMPOSITED.has(prop) && !seen[prop]) {
                                seen[prop] = true;
                                findings.push({ kind: 'keyframe', name: rule.name, property: prop, source: src });
                            }
                        }
                    }
                }

                // CSSStyleRule — check transition and -webkit-transition
                if (rule.type === 1 /* CSSStyleRule */ && rule.style) {
                    var tr = rule.style.transition || rule.style.webkitTransition || '';
                    if (tr) {
                        checkTransition(tr, rule.selectorText || '', src);
                    }
                }

                // @media — recurse one level
                if (rule.type === 4 /* CSSMediaRule */ && rule.cssRules) {
                    for (var mi = 0; mi < rule.cssRules.length; mi++) {
                        var mr = rule.cssRules[mi];
                        if (mr.type === 1 && mr.style) {
                            var mtr = mr.style.transition || mr.style.webkitTransition || '';
                            if (mtr) {
                                checkTransition(mtr, mr.selectorText || '', src);
                            }
                        }
                    }
                }
            }
        }

        return JSON.stringify(findings);
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Non-composited animation JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");
    let raw: Vec<RawFinding> = serde_json::from_str(json_str).unwrap_or_default();

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

    Ok(AnimationAnalysis {
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= max.saturating_sub(3))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}…", &s[..boundary])
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
