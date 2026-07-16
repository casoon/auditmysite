//! WCAG 1.4.12 Text Spacing (Level AA, WCAG 2.1)
//!
//! No loss of content or functionality occurs when a user applies the
//! following text-spacing properties: line height (line spacing) to at least
//! 1.5 times the font size, spacing following paragraphs to at least 2 times
//! the font size, letter spacing (tracking) to at least 0.12 times the font
//! size, and word spacing to at least 0.16 times the font size.
//!
//! Injects a stylesheet enforcing these minimums, then checks whether any
//! `overflow: hidden` container now clips its own content — the standard,
//! well-established heuristic for this check (matches the W3C's own "Text
//! Spacing" testing bookmarklet technique). The injected stylesheet is
//! removed before returning, so this check has no side effects for the
//! caller to clean up (unlike the viewport-based 1.4.10/1.4.4 checks).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const TEXT_SPACING_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.12",
    name: "Text Spacing",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Content remains readable when text spacing is increased to WCAG minimums",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/text-spacing.html",
    axe_id: "text-spacing",
    tags: &["wcag2aa", "wcag1412", "cat.text"],
};

const TEXT_SPACING_JS: &str = r#"
(function() {
  var elements = document.querySelectorAll('*');
  var count = Math.min(elements.length, 2000);

  // A `backface-visibility: hidden` element (checked up to 5 ancestors) is
  // almost always one face of a flip/reveal card animation — present in the
  // DOM with normal dimensions but rotated away from the viewer in its
  // resting state, not something a user actually sees or reads by default.
  function hasHiddenBackface(el) {
    var node = el;
    for (var depth = 0; depth < 5 && node; depth++) {
      if (getComputedStyle(node).backfaceVisibility === 'hidden') return true;
      node = node.parentElement;
    }
    return false;
  }

  function clippedFlags() {
    var flags = new Array(count);
    for (var i = 0; i < count; i++) {
      var el = elements[i];
      // Skip elements whose own visible box is tiny (<=2px) regardless of
      // spacing — the standard "sr-only"/visually-hidden technique clips a
      // 1x1px box permanently and isn't a real pointer/visual target, same
      // class of false positive as the target-size check (#QA-039).
      if (el.clientWidth <= 2 || el.clientHeight <= 2 || hasHiddenBackface(el)) {
        flags[i] = false;
        continue;
      }
      var cs = getComputedStyle(el);
      var clipsX = cs.overflowX === 'hidden' && el.scrollWidth > el.clientWidth + 2;
      var clipsY = cs.overflowY === 'hidden' && el.scrollHeight > el.clientHeight + 2;
      flags[i] = (clipsX || clipsY) && el.textContent.trim().length > 0;
    }
    return flags;
  }

  // Baseline: elements already clipped regardless of spacing (e.g. the
  // standard "sr-only"/visually-hidden technique, permanently 1x1 with
  // overflow:hidden) must not be reported — only content that becomes
  // *newly* clipped once the spacing increases is a real violation.
  var before = clippedFlags();

  var style = document.createElement('style');
  style.textContent =
    '* { line-height: 1.5 !important; letter-spacing: 0.12em !important; word-spacing: 0.16em !important; }' +
    'p { margin-bottom: 2em !important; }';
  document.head.appendChild(style);

  var after = clippedFlags();
  style.remove();

  for (var i = 0; i < count; i++) {
    if (after[i] && !before[i]) {
      var el = elements[i];
      var tag = el.tagName.toLowerCase();
      var id = el.id ? '#' + el.id : '';
      var cls = (el.className && typeof el.className === 'string')
        ? '.' + el.className.trim().split(/\s+/)[0] : '';
      return tag + id + cls;
    }
  }
  return null;
})()
"#;

pub async fn check_text_spacing_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(TEXT_SPACING_JS).await {
        Ok(r) => r,
        Err(e) => {
            warn!("text-spacing probe JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "text-spacing",
                crate::cli::WcagLevel::AA,
                "page_evaluation_failed",
            )];
        }
    };

    let selector = match result.value() {
        Some(v) if !v.is_null() => v.as_str().unwrap_or("unknown element").to_string(),
        _ => return vec![],
    };

    vec![Violation::new(
        TEXT_SPACING_RULE.id,
        TEXT_SPACING_RULE.name,
        TEXT_SPACING_RULE.level,
        TEXT_SPACING_RULE.severity,
        format!(
            "Content is clipped by '{selector}' when WCAG-minimum text spacing \
             (line-height 1.5×, letter-spacing 0.12em, word-spacing 0.16em, \
             paragraph spacing 2×) is applied."
        ),
        &selector,
    )
    .with_selector(&selector)
    .with_fix(
        "Avoid fixed heights/widths with overflow: hidden on text containers; \
         use min-height or allow containers to grow with content.",
    )
    .with_rule_id(TEXT_SPACING_RULE.axe_id)
    .with_help_url(TEXT_SPACING_RULE.help_url)]
}
