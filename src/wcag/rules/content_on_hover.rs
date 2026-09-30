//! WCAG 1.4.13 Content on Hover or Focus (Level AA)
//!
//! Detects two common anti-patterns:
//! - Elements relying solely on the native `title` attribute for important
//!   information (not keyboard-accessible, not screen-reader-friendly on
//!   touch devices). Reported as its own best-practice rule
//!   `title-only-description` (#711), not under 1.4.13.
//! - `role="tooltip"` elements that are not referenced by any
//!   `aria-describedby` attribute (orphaned, never announced).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const CONTENT_ON_HOVER_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.13",
    name: "Content on Hover or Focus",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Content shown on hover/focus must be dismissible, hoverable, and persistent",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/content-on-hover-or-focus.html",
    axe_id: "content-on-hover-focus",
    tags: &["wcag2aa", "wcag1413", "cat.color"],
};

// Eigene Regel fuer das `title`-Attribut als einzige Beschreibung (#711).
// 1.4.13 betrifft Inhalte, die bei Hover/Fokus *erscheinen* (schliessbar,
// hoverbar, dauerhaft) -- der Browser-Tooltip des `title`-Attributs ist davon
// ausgenommen (Understanding 1.4.13: vom User Agent gesteuert).
// Verankert an 4.1.2, nicht an 2.5.3: `title` ist nach accname eine gueltige
// Namensquelle (Technik H65 ist fuer 4.1.2 hinreichend), der Befund verletzt
// 4.1.2 also nicht, sondern ist Best Practice rund um den Namen. 2.5.3 gilt
// laut Understanding nur fuer Bedienelemente mit sichtbarer Textbeschriftung,
// die ein reines `title`-Element gerade nicht hat.
pub(super) const TITLE_ONLY_DESCRIPTION_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Title attribute as only description",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Interactive elements should not rely on the native title attribute as their only descriptive text",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "title-only-description",
    tags: &["best-practice", "wcag412", "cat.name-role-value"],
};

// Counts:
// - interactive elements (buttons, links) carrying a `title` attribute as
//   their only descriptive text (no aria-label, no visible text content)
// - role="tooltip" elements with no inbound aria-describedby reference
const CONTENT_ON_HOVER_JS: &str = r#"
(function() {
  /*CSS_SELECTOR*/
  const titleOnly = [];
  const orphanTooltips = [];
  try {
    // Collect all aria-describedby targets for tooltip orphan detection.
    const describedTargets = new Set();
    const allDescribers = document.querySelectorAll('[aria-describedby]');
    for (const el of Array.from(allDescribers)) {
      const ids = (el.getAttribute('aria-describedby') || '').split(/\s+/);
      for (const id of ids) if (id) describedTargets.add(id);
    }

    // Detect title-attribute-only patterns on interactive elements.
    const interactive = document.querySelectorAll('button[title], a[title][href], input[title]');
    for (const el of Array.from(interactive)) {
      const title = el.getAttribute('title');
      if (!title || !title.trim()) continue;
      const hasAriaLabel = !!(el.getAttribute('aria-label') || '').trim();
      const hasAriaLabelledby = !!(el.getAttribute('aria-labelledby') || '').trim();
      const textContent = (el.textContent || '').trim();
      // Weitere Namensquellen vor `title` (accname 1.2): ein zugeordnetes
      // <label>, der Wert eines Buttons und das alt eines Bildes im Element.
      // Ohne sie galt ein <input title> mit echtem <label> als title-only (#711).
      const hasLabel = !!(el.labels && Array.from(el.labels).some(l => (l.textContent || '').trim()));
      const type = (el.getAttribute('type') || '').toLowerCase();
      const hasValueName = el.localName === 'input' &&
        ['submit', 'button', 'reset'].includes(type) && !!(el.getAttribute('value') || '').trim();
      const hasImgAlt = Array.from(el.querySelectorAll('img[alt]')).some(i => i.getAttribute('alt').trim()) ||
        (el.localName === 'input' && type === 'image' && !!(el.getAttribute('alt') || '').trim());
      // If accessible name comes from text/aria, title is supplemental — fine.
      if (hasAriaLabel || hasAriaLabelledby || hasLabel || hasValueName || hasImgAlt ||
          textContent.length > 0) continue;
      __amsPush(titleOnly, el, __amsCssSelector(el), 10);
      if (__amsReal(titleOnly) >= 10) break;
    }

    // Detect orphan role="tooltip" elements.
    const tooltips = document.querySelectorAll('[role="tooltip"]');
    for (const tip of Array.from(tooltips)) {
      const id = tip.id;
      if (!id || !describedTargets.has(id)) {
        __amsPush(orphanTooltips, tip, '#' + (id || '(no-id)'), 10);
      }
      if (__amsReal(orphanTooltips) >= 10) break;
    }
  } catch(e) {}
  return { titleOnly, orphanTooltips };
})()
"#;

pub async fn check_content_on_hover_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &CONTENT_ON_HOVER_RULE,
        &CONTENT_ON_HOVER_JS.replace(
            "/*CSS_SELECTOR*/",
            crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let title_only: Vec<String> = val
        .get("titleOnly")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let orphans: Vec<String> = val
        .get("orphanTooltips")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut violations = Vec::new();
    for sel in title_only {
        violations.push(
            Violation::new(
                TITLE_ONLY_DESCRIPTION_RULE.id,
                TITLE_ONLY_DESCRIPTION_RULE.name,
                TITLE_ONLY_DESCRIPTION_RULE.level,
                TITLE_ONLY_DESCRIPTION_RULE.severity,
                format!(
                    "Interactive element uses the native `title` attribute as its only descriptive text ({sel}). Tooltips from `title` are not keyboard-accessible and unreliable on touch devices."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Provide a visible label, aria-label, or aria-labelledby on the element; reserve `title` for supplemental content only.",
            )
            .with_rule_id(TITLE_ONLY_DESCRIPTION_RULE.axe_id)
            .with_help_url(TITLE_ONLY_DESCRIPTION_RULE.help_url)
            .with_tags(
                TITLE_ONLY_DESCRIPTION_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        );
    }
    for sel in orphans {
        violations.push(
            Violation::new(
                CONTENT_ON_HOVER_RULE.id,
                CONTENT_ON_HOVER_RULE.name,
                CONTENT_ON_HOVER_RULE.level,
                Severity::Low,
                format!(
                    "role=\"tooltip\" element ({sel}) is not referenced by any aria-describedby — assistive tech will never announce it."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Reference the tooltip from its trigger via aria-describedby=\"<tooltip-id>\".",
            )
            .with_rule_id(CONTENT_ON_HOVER_RULE.axe_id)
            .with_help_url(CONTENT_ON_HOVER_RULE.help_url),
        );
    }
    violations
}
