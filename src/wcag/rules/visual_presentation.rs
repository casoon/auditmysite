//! WCAG 1.4.8 Visual Presentation (Level AAA)
//!
//! For blocks of text: foreground/background colours can be selected by the
//! user; width is no more than 80 characters; text is not fully justified;
//! line spacing is at least 1.5; text can be resized without AT.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const VISUAL_PRESENTATION_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.8",
    name: "Visual Presentation",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Text blocks must not be fully justified and must have adequate line spacing",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/visual-presentation.html",
    axe_id: "visual-presentation",
    tags: &["wcag2aaa", "wcag148", "cat.sensory-and-visual-cues"],
};

const VISUAL_PRESENTATION_JS: &str = r#"
(function() {
  var hasJustified = false;
  var hasNarrowLineHeight = false;

  try {
    for (var i = 0; i < document.styleSheets.length; i++) {
      var sheet = document.styleSheets[i];
      var rules;
      try { rules = Array.from(sheet.cssRules || []); } catch(e) { continue; }
      for (var j = 0; j < rules.length; j++) {
        var r = rules[j];
        if (r.type === CSSRule.STYLE_RULE && r.style) {
          var sel = (r.selectorText || '').toLowerCase();
          var isBodyText = /^(body|p|div|article|section|main)/.test(sel) || sel === '*';
          if (isBodyText) {
            if (r.style.textAlign === 'justify') hasJustified = true;
            var lh = r.style.lineHeight;
            if (lh && lh !== 'normal') {
              var num = parseFloat(lh);
              if (!isNaN(num) && num < 1.5) hasNarrowLineHeight = true;
            }
          }
        }
      }
    }
  } catch(e) {}

  return { hasJustified: hasJustified, hasNarrowLineHeight: hasNarrowLineHeight };
})()
"#;

pub async fn check_visual_presentation_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        VISUAL_PRESENTATION_RULE.id,
        VISUAL_PRESENTATION_RULE.name,
        VISUAL_PRESENTATION_RULE.level,
        Severity::Low,
        "Some aspects of 1.4.8 (user-selectable colours, column width) require manual review \
         and cannot be automatically verified.",
        "page",
    )
    .with_fix(
        "Ensure users can select foreground/background colours, text width does not exceed \
         80 characters, and text can be resized up to 200% without assistive technology.",
    )
    .with_rule_id(VISUAL_PRESENTATION_RULE.axe_id)
    .with_help_url(VISUAL_PRESENTATION_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(VISUAL_PRESENTATION_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let has_justified = val
        .get("hasJustified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_narrow_line_height = val
        .get("hasNarrowLineHeight")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut findings = vec![not_testable];

    if has_justified {
        findings.push(
            Violation::new(
                VISUAL_PRESENTATION_RULE.id,
                VISUAL_PRESENTATION_RULE.name,
                VISUAL_PRESENTATION_RULE.level,
                Severity::Low,
                "Body text uses text-align: justify, which creates uneven spacing between words \
                 that can reduce readability for users with dyslexia.",
                "stylesheet",
            )
            .with_fix(
                "Remove text-align: justify from body text. Use text-align: left (or start) instead.",
            )
            .with_rule_id(VISUAL_PRESENTATION_RULE.axe_id)
            .with_help_url(VISUAL_PRESENTATION_RULE.help_url),
        );
    }

    if has_narrow_line_height {
        findings.push(
            Violation::new(
                VISUAL_PRESENTATION_RULE.id,
                VISUAL_PRESENTATION_RULE.name,
                VISUAL_PRESENTATION_RULE.level,
                Severity::Low,
                "Body text has line-height below 1.5, making text harder to read for users \
                 with cognitive or visual disabilities.",
                "stylesheet",
            )
            .with_fix("Set line-height to at least 1.5 for body text paragraphs.")
            .with_rule_id(VISUAL_PRESENTATION_RULE.axe_id)
            .with_help_url(VISUAL_PRESENTATION_RULE.help_url),
        );
    }

    findings
}
