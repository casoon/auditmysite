//! WCAG 1.1.1 - Non-text Content: additional input/object rules
//!
//! Covers axe-core rules that extend image-alt to other non-text elements:
//! - `area-alt`:        <area> elements in image maps must have alt text
//! - `input-image-alt`: <input type="image"> must have alt text
//! - `object-alt`:      <object> elements must have a text alternative
//!
//! DOM-level rule: `htmlTag`/`type` are not AX properties (the AX tree
//! synthesizes an accessible name for `<input type="image">` from its `alt`
//! attribute directly, and never exposes the tag/type), so an earlier
//! tree-based implementation of this check never fired in production
//! (#QA-030).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_AREA_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Area Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Active <area> elements in image maps must have alternative text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "area-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

pub const RULE_INPUT_IMAGE_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Image Button Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "<input type=\"image\"> elements must have alternative text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "input-image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

pub const RULE_OBJECT_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Object Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "<object> elements must have a text alternative",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "object-alt",
    tags: &["wcag2a", "wcag111", "cat.text-alternatives"],
};

const IMAGE_INPUT_CAP: usize = 250;

const IMAGE_INPUT_BODY: &str = r#"
  var issues = [];

  var areas = document.querySelectorAll('area[href]');
  for (var i = 0; i < areas.length && issues.length < CAP; i++) {
    var area = areas[i];
    var alt = (area.getAttribute('alt') || '').trim();
    if (!alt) {
      issues.push({ kind: 'area', selector: __amsCssSelector(area) });
    }
  }

  var inputImages = document.querySelectorAll('input[type="image"]');
  for (var j = 0; j < inputImages.length && issues.length < CAP; j++) {
    var input = inputImages[j];
    var inputAlt = (input.getAttribute('alt') || '').trim();
    var ariaLabel = (input.getAttribute('aria-label') || '').trim();
    if (!inputAlt && !ariaLabel) {
      issues.push({ kind: 'input-image', selector: __amsCssSelector(input) });
    }
  }

  var objects = document.querySelectorAll('object, embed');
  for (var k = 0; k < objects.length && issues.length < CAP; k++) {
    var obj = objects[k];
    var text = (obj.textContent || '').trim();
    var objAriaLabel = (obj.getAttribute('aria-label') || '').trim();
    var title = (obj.getAttribute('title') || '').trim();
    if (!text && !objAriaLabel && !title) {
      issues.push({ kind: 'object', selector: __amsCssSelector(obj) });
    }
  }

  return { issues: issues };
"#;

/// Run all image-input/object text-alternative checks.
pub async fn check_image_input_rules_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &IMAGE_INPUT_BODY.replace("CAP", &IMAGE_INPUT_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("image-input JS failed: {}", e);
            return vec![];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let kind = issue.get("kind")?.as_str()?;
            let selector = issue.get("selector")?.as_str()?.to_string();

            let (rule, message, fix) = match kind {
                "area" => (
                    &RULE_AREA_ALT,
                    "Active <area> element is missing alternative text".to_string(),
                    "Add an alt attribute to the <area> element describing its destination",
                ),
                "input-image" => (
                    &RULE_INPUT_IMAGE_ALT,
                    "Image submit button is missing alternative text".to_string(),
                    "Add an alt attribute to the <input type=\"image\"> element \
                     describing the button action",
                ),
                "object" => (
                    &RULE_OBJECT_ALT,
                    "<object> element is missing a text alternative".to_string(),
                    "Provide a text alternative inside the <object> element or via aria-label",
                ),
                _ => return None,
            };

            Some(
                Violation::new(
                    rule.id,
                    rule.name,
                    rule.level,
                    rule.severity,
                    message,
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix(fix)
                .with_rule_id(rule.axe_id)
                .with_help_url(rule.help_url),
            )
        })
        .collect()
}
