//! WCAG 1.4.1 Use of Color (Level A)
//!
//! Color must not be the only visual means of conveying information. The
//! most common failure is inline links that differ from surrounding text
//! only by color (no underline, no weight change, no icon).
//!
//! This check inspects links inside paragraph-like containers and flags
//! those that have no underline, no font-weight delta, and no other visual
//! marker compared to their parent.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const USE_OF_COLOR_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.1",
    name: "Use of Color",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Color must not be the only visual indicator for information",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html",
    axe_id: "link-in-text-block",
    tags: &["wcag2a", "wcag141", "cat.color"],
};

// Scans for inline links (inside p, li, span, td) whose computed style
// shows no text-decoration: underline and no font-weight delta vs. the
// parent. Returns up to 10 selectors. Skips links inside <nav> elements
// where the surrounding context (menu, button-like styling) typically
// distinguishes them visually.
const USE_OF_COLOR_JS: &str = r#"
(function() {
  const findings = [];
  try {
    const links = document.querySelectorAll('a[href]');
    for (const link of Array.from(links)) {
      // Skip links inside nav/header/aside menus where context disambiguates.
      let inNav = false;
      let p = link.parentElement;
      while (p) {
        const tag = p.tagName.toLowerCase();
        if (tag === 'nav') { inNav = true; break; }
        if (tag === 'body') break;
        p = p.parentElement;
      }
      if (inNav) continue;

      const parent = link.parentElement;
      if (!parent) continue;
      const parentTag = parent.tagName.toLowerCase();
      // Only consider links embedded in textual containers.
      if (!['p', 'li', 'span', 'td', 'div', 'figcaption', 'blockquote'].includes(parentTag)) continue;
      // Skip empty links or links whose only child is an image/icon.
      if (link.children.length === 1 && ['img', 'svg', 'i'].includes(link.children[0].tagName.toLowerCase())) continue;
      if (!link.textContent || !link.textContent.trim()) continue;

      const linkStyle = window.getComputedStyle(link);
      // Skip block-level and flex/grid links — these are card or structural
      // links that are visually distinguishable by layout, not just color.
      const display = linkStyle.display || '';
      if (display === 'block' || display === 'flex' || display === 'grid' ||
          display === 'inline-flex' || display === 'inline-grid') continue;
      const parentStyle = window.getComputedStyle(parent);
      // Skip if the link is a flex/grid item — even if its own display is inline,
      // the parent container makes it visually block-like (card, row, tile).
      const parentDisplay = parentStyle.display || '';
      if (parentDisplay === 'flex' || parentDisplay === 'grid' ||
          parentDisplay === 'inline-flex' || parentDisplay === 'inline-grid') continue;

      const linkDecoration = (linkStyle.textDecorationLine || linkStyle.textDecoration || '').toLowerCase();
      const hasUnderline = linkDecoration.includes('underline');
      const sameWeight = linkStyle.fontWeight === parentStyle.fontWeight;
      const sameFontStyle = linkStyle.fontStyle === parentStyle.fontStyle;
      const sameFontFamily = linkStyle.fontFamily === parentStyle.fontFamily;
      const sameBorder = linkStyle.borderBottomStyle === parentStyle.borderBottomStyle;
      const sameBackground = linkStyle.backgroundColor === parentStyle.backgroundColor;

      if (!hasUnderline && sameWeight && sameFontStyle && sameFontFamily && sameBorder && sameBackground) {
        const tag = link.tagName.toLowerCase();
        const id = link.id ? '#' + link.id : '';
        const cls = link.className && typeof link.className === 'string'
          ? '.' + link.className.trim().split(/\s+/).slice(0, 2).join('.')
          : '';
        findings.push(tag + id + cls);
        if (findings.length >= 10) break;
      }
    }
  } catch(e) {}
  return { count: findings.length, selectors: findings };
})()
"#;

pub async fn check_use_of_color_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(USE_OF_COLOR_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let count = val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    if count == 0 {
        return vec![];
    }

    let selectors: Vec<String> = val
        .get("selectors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    selectors
        .into_iter()
        .map(|sel| {
            Violation::new(
                USE_OF_COLOR_RULE.id,
                USE_OF_COLOR_RULE.name,
                USE_OF_COLOR_RULE.level,
                Severity::Medium,
                format!(
                    "Inline link distinguishable from surrounding text by color alone ({sel}). Users with low vision or color blindness may not recognize it as a link."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Add a non-color cue: text-decoration: underline, a different font-weight, an icon, or a bottom border. Underline is the strongest convention.",
            )
            .with_rule_id(USE_OF_COLOR_RULE.axe_id)
            .with_help_url(USE_OF_COLOR_RULE.help_url)
        })
        .collect()
}
