//! WCAG 1.4.10 Reflow
//!
//! Content can be presented without loss of information or functionality, and
//! without requiring scrolling in two dimensions for:
//! - Vertical scrolling content at a width equivalent to 320 CSS pixels
//! - Horizontal scrolling content at a height equivalent to 256 CSS pixels
//! Level AA
//!
//! This check temporarily sets the viewport to 320×256 CSS pixels and tests
//! for horizontal scrolling. The caller is responsible for restoring the
//! viewport to its previous state after this function returns.

use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const REFLOW_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.10",
    name: "Reflow",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Content reflows without horizontal scrolling at 320 CSS pixels width",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/reflow.html",
    axe_id: "css-overflow-hidden",
    tags: &["wcag2aa", "wcag1410", "cat.sensory-and-visual-cues"],
};

// JS that returns a brief description of the worst-offending element, or null.
const FIND_OVERFLOW_JS: &str = r#"
(function() {
  const vw = window.innerWidth;
  // Check if the document itself overflows horizontally
  if (document.documentElement.scrollWidth <= vw + 5) return null;
  // Find the widest element to give a useful selector hint
  let worst = null;
  let worstRight = vw + 5;
  for (const el of document.querySelectorAll('*')) {
    try {
      const r = el.getBoundingClientRect();
      if (r.right > worstRight) {
        worstRight = r.right;
        const tag = el.tagName.toLowerCase();
        const id = el.id ? '#' + el.id : '';
        const cls = el.className && typeof el.className === 'string'
          ? '.' + el.className.trim().split(/\s+/)[0] : '';
        worst = tag + id + cls;
      }
    } catch(e) {}
  }
  return worst || 'unknown element';
})()
"#;

/// Check reflow at 320 CSS px width.
///
/// **Side effect**: changes the page viewport to 320×256. The caller must
/// restore the viewport to the desired state after this call returns.
pub async fn check_reflow_with_page(page: &Page) -> Vec<Violation> {
    // Switch to 320 × 256 (WCAG 2.1 reference dimensions, non-mobile scaling)
    let narrow = SetDeviceMetricsOverrideParams::builder()
        .mobile(false)
        .width(320_i64)
        .height(256_i64)
        .device_scale_factor(1.0_f64)
        .build()
        .unwrap();

    if page.execute(narrow).await.is_err() {
        return vec![];
    }

    // Allow layout to reflow
    tokio::time::sleep(Duration::from_millis(350)).await;

    let result = match page.evaluate(FIND_OVERFLOW_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let overflow_element = match result.value() {
        Some(v) if !v.is_null() => v.as_str().unwrap_or("unknown element").to_string(),
        _ => return vec![],
    };

    vec![Violation::new(
        REFLOW_RULE.id,
        REFLOW_RULE.name,
        REFLOW_RULE.level,
        Severity::High,
        format!(
            "Page requires horizontal scrolling at 320 CSS px width (overflowing element: {overflow_element}). \
             Users who zoom to 400% on a 1280px display will need to scroll in two directions."
        ),
        "document",
    )
    .with_fix(
        "Use responsive CSS (max-width: 100%, flexbox/grid wrap, relative units). \
         Avoid fixed pixel widths on containers. \
         Test at 320px viewport width or 400% browser zoom.",
    )
    .with_rule_id(REFLOW_RULE.axe_id)
    .with_help_url(REFLOW_RULE.help_url)]
}
