//! WCAG 1.4.11 Non-text Contrast — CSS-level check
//!
//! Complements the AXTree-based non_text_contrast rule, which only checks
//! whether a checkbox/radio/switch exposes an accessible checked state —
//! a property real controls (native or ARIA) expose almost universally, so
//! that check essentially never fires against real-world markup.
//!
//! This check instead inspects the actual rendered boundary color of custom
//! (author-restyled, `appearance: none`) checkboxes/radios/switches/range
//! inputs against their surrounding background, and flags insufficient
//! (<3:1) contrast — the pattern axe/Pa11y catch in practice (e.g. a custom
//! checkbox whose "checked" state only changes to a barely-different shade).
//! Native, un-restyled controls are skipped: their contrast is governed by
//! OS/browser chrome that CSS inspection cannot verify, and flagging them
//! would reintroduce false positives.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

use super::contrast::{Color, ContrastRule};

pub const NON_TEXT_CONTRAST_CSS_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.11",
    name: "Non-text Contrast",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "UI components and graphical objects have a contrast ratio of at least 3:1",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html",
    axe_id: "non-text-contrast-css",
    tags: &["wcag2aa", "wcag1411", "cat.color"],
};

const NON_TEXT_CONTRAST_JS: &str = r#"
(function() {
  const results = [];
  const els = Array.from(document.querySelectorAll(
    'input[type=checkbox], input[type=radio], input[type=range], [role=checkbox], [role=radio], [role=switch]'
  ));

  function effectiveBackground(el) {
    let node = el.parentElement;
    for (let i = 0; i < 6 && node; i++) {
      const bg = getComputedStyle(node).backgroundColor;
      if (bg && bg !== 'transparent' && bg !== 'rgba(0, 0, 0, 0)') return bg;
      node = node.parentElement;
    }
    return 'rgb(255, 255, 255)';
  }

  for (const el of els) {
    const cs = getComputedStyle(el);
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) continue;
    if (cs.visibility === 'hidden' || cs.display === 'none') continue;

    // Only custom-styled controls: native rendering is governed by the
    // browser/OS and cannot be verified (or meaningfully flagged) via CSS.
    const appearance = cs.appearance || cs.webkitAppearance || '';
    if (appearance !== 'none') continue;

    const borderWidth = parseFloat(cs.borderTopWidth) || 0;
    const hasBorder = borderWidth > 0 && cs.borderTopStyle !== 'none';
    const boundaryColor = hasBorder ? cs.borderTopColor : cs.backgroundColor;
    if (!boundaryColor || boundaryColor === 'transparent' || boundaryColor === 'rgba(0, 0, 0, 0)') continue;

    const parentBg = effectiveBackground(el);
    let selector = el.tagName.toLowerCase();
    if (el.id) selector += '#' + el.id;
    const role = el.getAttribute('role') || el.type || selector;

    results.push({ selector, role, boundaryColor, parentBg });
  }

  return { results };
})()
"#;

pub async fn check_non_text_contrast_css_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &NON_TEXT_CONTRAST_CSS_RULE,
        NON_TEXT_CONTRAST_JS,
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let entries = match val.get("results").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &NON_TEXT_CONTRAST_CSS_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    let mut violations = Vec::new();

    for entry in entries {
        let selector = entry.get("selector").and_then(|v| v.as_str()).unwrap_or("");
        let role = entry
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("control");
        let boundary_str = match entry.get("boundaryColor").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let bg_str = match entry.get("parentBg").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };

        let boundary = match Color::from_css(boundary_str) {
            Some(c) => c,
            None => continue,
        };
        let bg = match Color::from_css(bg_str) {
            Some(c) => c,
            None => continue,
        };

        let white = Color::new(255, 255, 255);
        let bg_eff = bg.composite_over(&white);
        let boundary_eff = boundary.composite_over(&bg_eff);
        let ratio = ContrastRule::calculate_contrast_ratio(&boundary_eff, &bg_eff);

        if ratio >= 3.0 {
            continue;
        }

        violations.push(
            Violation::new(
                NON_TEXT_CONTRAST_CSS_RULE.id,
                NON_TEXT_CONTRAST_CSS_RULE.name,
                NON_TEXT_CONTRAST_CSS_RULE.level,
                NON_TEXT_CONTRAST_CSS_RULE.severity,
                format!(
                    "Custom {} control boundary color {} has a contrast ratio of {:.2}:1 against its background {} — requires at least 3:1",
                    role, boundary_str, ratio, bg_str
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix("Increase the contrast of the control's border or fill color against its surrounding background to at least 3:1")
            .with_rule_id(NON_TEXT_CONTRAST_CSS_RULE.axe_id)
            .with_help_url(NON_TEXT_CONTRAST_CSS_RULE.help_url),
        );
    }

    violations
}
