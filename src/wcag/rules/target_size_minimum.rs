//! WCAG 2.5.8 Target Size (Minimum) (Level AA, WCAG 2.2)
//!
//! The size of the target for pointer inputs is at least 24 by 24 CSS pixels,
//! except where the target is a link in a sentence or block of text, or where
//! it is spaced so that a 24 px circle around it touches no other target.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const TARGET_SIZE_MINIMUM_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.8",
    name: "Target Size (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Interactive targets are at least 24×24 CSS pixels",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html",
    axe_id: "target-size-minimum",
    tags: &["wcag22aa", "wcag258", "cat.sensory-and-visual-cues"],
};

/// Shared with 2.5.5: whether a target is "in a sentence or block of text" —
/// the inline exception both criteria grant. The target is inline, and the
/// line it sits in carries text that is not itself a target: the inline
/// content of its block (text nodes and inline elements, not nested blocks),
/// minus every link and button. Comparing against the block's whole text was
/// wrong — a skip link directly under `<body>` counted as "in running text"
/// because the body holds the whole page, and links side by side in a `<nav>`
/// counted each other's labels as surrounding text.
pub(super) const TARGET_HELPERS_JS: &str = r#"
function isInlineInText(el) {
  if (getComputedStyle(el).display !== 'inline') return false;
  var block = el.parentElement;
  while (block && getComputedStyle(block).display === 'inline') block = block.parentElement;
  if (!block) return false;
  var isTarget = function(n) {
    return n.matches && n.matches('a[href], button, [role="button"], [role="link"], input');
  };
  var text = '';
  (function collect(node) {
    for (var c = node.firstChild; c; c = c.nextSibling) {
      if (c.nodeType === 3) { text += c.textContent; continue; }
      if (c.nodeType !== 1 || isTarget(c)) continue;
      if (getComputedStyle(c).display === 'inline') collect(c);
    }
  })(block);
  return text.replace(/\s+/g, '').length > 0;
}

// `tag#id`, or an nth-of-type path. The bare tag name used before made every
// link on a page the same occurrence ("a"), so findings merged into one.
function targetSelector(el) {
  if (el.id) return el.tagName.toLowerCase() + '#' + el.id;
  var parts = [];
  var node = el;
  while (node && node.nodeType === 1 && parts.length < 6) {
    var tag = node.nodeName.toLowerCase();
    if (node.id) { parts.unshift(tag + '#' + node.id); break; }
    var parent = node.parentNode;
    if (parent && parent.children) {
      var same = Array.prototype.filter.call(parent.children, function (c) {
        return c.nodeName === node.nodeName;
      });
      if (same.length > 1) tag += ':nth-of-type(' + (same.indexOf(node) + 1) + ')';
    }
    parts.unshift(tag);
    node = parent;
  }
  return parts.join(' > ');
}
"#;

/// Undersized targets pass when the spacing exception holds: a 24 px circle
/// centred on the target intersects no other target and no other undersized
/// target's circle. Before, every lone small button and every link in running
/// text was reported — neither is a 2.5.8 failure.
const TARGET_SIZE_JS: &str = r#"
(function() {
  var MIN_SIZE = 24;
  var RADIUS = MIN_SIZE / 2;
  /*TARGET_HELPERS*/
  var selectors = 'button, a[href], [role="button"], [role="link"], input[type="submit"], input[type="button"], input[type="reset"]';
  var targets = [];
  var all = document.querySelectorAll(selectors);
  for (var i = 0; i < all.length && targets.length < 1000; i++) {
    var r = all[i].getBoundingClientRect();
    // A near-zero rect (commonly exactly 1x1) is the standard "visually
    // hidden until focus" CSS technique for things like skip links — not
    // actually laid out/visible, or intentionally not a pointer target in its
    // resting state. A genuinely too-small but real button/icon is virtually
    // never this tiny, so this threshold doesn't mask real violations.
    if (r.width <= 2 || r.height <= 2) continue;
    targets.push({ el: all[i], rect: r, cx: r.left + r.width / 2, cy: r.top + r.height / 2,
                   small: r.width < MIN_SIZE || r.height < MIN_SIZE });
  }
  function distToRect(x, y, r) {
    var dx = Math.max(r.left - x, 0, x - r.right);
    var dy = Math.max(r.top - y, 0, y - r.bottom);
    return Math.sqrt(dx * dx + dy * dy);
  }
  function spaced(t) {
    for (var k = 0; k < targets.length; k++) {
      var o = targets[k];
      if (o === t || o.el.contains(t.el) || t.el.contains(o.el)) continue;
      // The circle may touch no other target at all, and — if that target
      // is undersized too — not its circle either.
      if (distToRect(t.cx, t.cy, o.rect) < RADIUS) return false;
      if (o.small && Math.hypot(t.cx - o.cx, t.cy - o.cy) < 2 * RADIUS) return false;
    }
    return true;
  }
  var violations = [];
  for (var j = 0; j < targets.length && violations.length < 5; j++) {
    var t = targets[j];
    if (!t.small || isInlineInText(t.el) || spaced(t)) continue;
    var el = t.el;
    var desc = el.getAttribute('aria-label') || el.textContent.trim().substring(0, 40) || el.tagName.toLowerCase();
    violations.push({
      selector: targetSelector(el),
      label: desc,
      width: Math.round(t.rect.width),
      height: Math.round(t.rect.height)
    });
  }
  return { violations: violations };
})()
"#;

pub async fn check_target_size_minimum_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &TARGET_SIZE_MINIMUM_RULE,
        &TARGET_SIZE_JS.replace("/*TARGET_HELPERS*/", TARGET_HELPERS_JS),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_MINIMUM_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    violations
        .iter()
        .map(|item| {
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            let label = item
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            let width = item.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
            let height = item.get("height").and_then(|v| v.as_u64()).unwrap_or(0);

            Violation::new(
                TARGET_SIZE_MINIMUM_RULE.id,
                TARGET_SIZE_MINIMUM_RULE.name,
                TARGET_SIZE_MINIMUM_RULE.level,
                Severity::Medium,
                format!(
                    "Interactive target '{}' is {}×{} CSS pixels, below the 24×24 minimum.",
                    label, width, height
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Increase the target size to at least 24×24 CSS pixels using padding, \
                 min-width/min-height, or by enlarging the element.",
            )
            .with_rule_id(TARGET_SIZE_MINIMUM_RULE.axe_id)
            .with_help_url(TARGET_SIZE_MINIMUM_RULE.help_url)
        })
        .collect()
}
