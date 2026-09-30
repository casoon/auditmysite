//! WCAG 1.4.1 Use of Color (Level A)
//!
//! Color must not be the only visual means of conveying information. The
//! most common failure is inline links that differ from surrounding text
//! only by color (no underline, no weight change, no icon).
//!
//! This check inspects links inside paragraph-like containers and flags
//! those that have no underline, no font-weight delta, and no other visual
//! marker compared to their parent.
//!
//! Nur Links im Fliesstext zaehlen (#710), wie bei axe `link-in-text-block`
//! (`isInTextBlock`): Der Abschnitt des umgebenden Blocks (begrenzt durch
//! `<br>`/`<hr>`) muss mehr Nicht-Link-Text enthalten als Link-/Widget-Text,
//! und dieser Text muss aus mindestens zwei Woertern bestehen — Trenner wie
//! `|` oder `·` zwischen Footer-Links sind kein Fliesstext. Links in `nav`
//! bzw. Menues und Logo-Links (Bild/SVG im Link, Logo-Container) werden
//! nicht geprueft. Die Entscheidung faellt in Rust (`is_in_running_text`).

use chromiumoxide::Page;
use serde::Deserialize;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const USE_OF_COLOR_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.1",
    name: "Use of Color",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Color must not be the only visual indicator for information",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html",
    axe_id: "link-in-text-block",
    tags: &["wcag2a", "wcag141", "cat.color"],
};

// Scans for inline links whose computed style shows no
// text-decoration: underline and no font-weight delta vs. the parent.
// Liefert Kandidaten mit dem Text ihres Blocks; ob der Link im Fliesstext
// steht, entscheidet `is_in_running_text`.
const USE_OF_COLOR_JS: &str = r#"
(function() {
  /*CSS_SELECTOR*/
  const LIMIT = 300;
  const candidates = [];
  const clean = (s) => String(s || '').replace(/\s+/g, ' ').trim();
  const isBlock = (el) => {
    const d = window.getComputedStyle(el).display || '';
    return d !== 'inline' && d !== 'contents';
  };
  const isWidget = (el) => {
    const tag = el.tagName.toLowerCase();
    if ((tag === 'a' && el.hasAttribute('href')) ||
        ['button', 'input', 'select', 'textarea'].includes(tag)) return true;
    const role = (el.getAttribute('role') || '').toLowerCase();
    return ['link', 'button', 'menuitem', 'tab', 'checkbox', 'radio', 'switch', 'option'].includes(role);
  };
  // Nach axe `isInTextBlock`: Text des Blockabschnitts um den Link, getrennt
  // in Widget-Text (Links, Buttons, der Link selbst) und uebrigen Text.
  const blockTexts = (link) => {
    let block = link.parentElement;
    while (block && block !== document.body && !isBlock(block)) block = block.parentElement;
    if (!block) return null;
    let other = '';
    let widget = '';
    let state = 0; // 0 vor dem Link, 1 danach, 2 Abschnitt zu Ende
    const walk = (node) => {
      for (const child of Array.from(node.childNodes)) {
        if (state === 2) return;
        if (child.nodeType === 3) { other += child.nodeValue; continue; }
        if (child.nodeType !== 1) continue;
        const tag = child.tagName.toLowerCase();
        if (tag === 'br' || tag === 'hr') {
          if (state === 0) { other = ''; widget = ''; } else { state = 2; }
          continue;
        }
        if (child === link) { state = 1; widget += ' ' + (child.textContent || ''); continue; }
        if (['script', 'style', 'template', 'noscript'].includes(tag)) continue;
        const cs = window.getComputedStyle(child);
        if (cs.display === 'none' || cs.visibility === 'hidden') continue;
        if (cs.float !== 'none' || (cs.position !== 'static' && cs.position !== 'relative')) continue;
        if (isWidget(child)) { widget += ' ' + (child.textContent || ''); continue; }
        // Verschachtelte Bloecke (Unterliste, Ueberschrift) sind eigene Zeilen.
        if (isBlock(child)) continue;
        walk(child);
      }
    };
    walk(block);
    return { other: clean(other), widget: clean(widget) };
  };
  try {
    const links = document.querySelectorAll('a[href]');
    for (const link of Array.from(links)) {
      if (candidates.length >= LIMIT) break;
      const parent = link.parentElement;
      if (!parent) continue;
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
        const texts = blockTexts(link);
        if (!texts) continue;
        candidates.push({
          selector: __amsCssSelector(link),
          other_text: texts.other.slice(0, 200),
          link_text_len: texts.widget.length,
          in_nav: !!link.closest('nav, [role="navigation"], [role="menubar"], [role="menu"]'),
          logo: !!(link.querySelector('img, svg') ||
                   link.closest('[class*="logo" i], [id*="logo" i]')),
          excluded: __amsIsExcludedEl(link),
        });
      }
    }
  } catch(e) {}
  return { candidates };
})()
"#;

/// Hoechstzahl gemeldeter Links, getrennt fuer echte und ausgeschlossene
/// Treffer (wie `__amsPush`).
const MAX_FINDINGS: usize = 10;

/// Mindestzahl an Woertern im Nicht-Link-Text des Blocks.
const MIN_RUNNING_TEXT_WORDS: usize = 2;

/// Ein Link ohne nicht-farbliches Merkmal, mit dem Text seines Blocks.
#[derive(Debug, Default, Deserialize)]
struct LinkCandidate {
    selector: String,
    /// Nicht-Link-Text im Blockabschnitt des Links (bereinigt, gekuerzt).
    #[serde(default)]
    other_text: String,
    /// Laenge des Link-/Widget-Texts im selben Abschnitt.
    #[serde(default)]
    link_text_len: usize,
    #[serde(default)]
    in_nav: bool,
    #[serde(default)]
    logo: bool,
    #[serde(default)]
    excluded: bool,
}

/// Steht der Link im Fliesstext? Wie axe `isInTextBlock`: mehr uebriger Text
/// als Link-Text, zusaetzlich mindestens zwei Woerter — Trenner zwischen
/// Footer-Links oder ein einzelnes Wort sind kein Fliesstext.
fn is_in_running_text(c: &LinkCandidate) -> bool {
    if c.in_nav || c.logo {
        return false;
    }
    let words = c
        .other_text
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count();
    words >= MIN_RUNNING_TEXT_WORDS && c.other_text.chars().count() > c.link_text_len
}

/// Links im Fliesstext, begrenzt auf je `MAX_FINDINGS` echte und
/// ausgeschlossene Treffer.
fn select_findings(candidates: Vec<LinkCandidate>) -> Vec<String> {
    let (mut real, mut excluded) = (0, 0);
    candidates
        .into_iter()
        .filter(is_in_running_text)
        .filter(|c| {
            let count = if c.excluded { &mut excluded } else { &mut real };
            *count += 1;
            *count <= MAX_FINDINGS
        })
        .map(|c| c.selector)
        .collect()
}

pub async fn check_use_of_color_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &USE_OF_COLOR_RULE,
        &USE_OF_COLOR_JS.replace(
            "/*CSS_SELECTOR*/",
            crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let candidates: Vec<LinkCandidate> = val
        .get("candidates")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    select_findings(candidates)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(other_text: &str, link_text: &str) -> LinkCandidate {
        LinkCandidate {
            selector: "a".into(),
            other_text: other_text.into(),
            link_text_len: link_text.chars().count(),
            ..Default::default()
        }
    }

    #[test]
    fn link_in_paragraph_is_running_text() {
        assert!(is_in_running_text(&candidate(
            "Read our for details.",
            "privacy policy"
        )));
    }

    #[test]
    fn list_item_with_only_the_link_is_not_running_text() {
        assert!(!is_in_running_text(&candidate("", "Kontakt")));
    }

    #[test]
    fn separators_between_footer_links_are_not_running_text() {
        assert!(!is_in_running_text(&candidate(
            "| |",
            "Impressum Datenschutz Barrierefreiheit"
        )));
        assert!(!is_in_running_text(&candidate("· ·", "A B C")));
    }

    #[test]
    fn link_text_longer_than_surrounding_text_is_not_running_text() {
        assert!(!is_in_running_text(&candidate(
            "Mehr zu",
            "Barrierefreiheit und Datenschutz"
        )));
    }

    #[test]
    fn nav_and_logo_links_are_never_running_text() {
        let mut c = candidate("Welcome to the official portal of the ministry.", "Home");
        assert!(is_in_running_text(&c));
        c.in_nav = true;
        assert!(!is_in_running_text(&c));
        c.in_nav = false;
        c.logo = true;
        assert!(!is_in_running_text(&c));
    }

    #[test]
    fn cap_counts_real_and_excluded_separately() {
        let mut candidates: Vec<LinkCandidate> = (0..12)
            .map(|i| LinkCandidate {
                selector: format!("a#r{i}"),
                ..candidate("some running text here", "x")
            })
            .collect();
        candidates.push(LinkCandidate {
            selector: "a#ex".into(),
            excluded: true,
            ..candidate("some running text here", "x")
        });
        candidates.push(candidate("", "Menu"));
        let found = select_findings(candidates);
        assert_eq!(found.len(), MAX_FINDINGS + 1);
        assert!(found.contains(&"a#ex".to_string()));
    }
}
