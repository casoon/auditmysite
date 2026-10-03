//! WCAG 1.1.1 - Non-text Content: `area-alt`
//!
//! `<area>` elements in image maps must have alt text. `input-image-alt` and
//! `object-alt` run as `images/input-alt-missing` and `objects/alt-missing`
//! in the shared rules since #696. `area-alt` stays here until
//! `images/area-alt-missing` sees an `<area>` in a document with computed
//! styles: the UA stylesheet gives `<area>` `display: none`, and the shared
//! rules' rendered view drops it as hidden (reported to barrierlab).
//!
//! DOM-level rule: `htmlTag`/`type` are not AX properties (the AX tree
//! synthesizes an accessible name for `<input type="image">` from its `alt`
//! attribute directly, and never exposes the tag/type), so an earlier
//! tree-based implementation of this check never fired in production
//! (#QA-030).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const RULE_AREA_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Area Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Active <area> elements in image maps must have alternative text",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    axe_id: "area-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

const IMAGE_INPUT_CAP: usize = 250;

const IMAGE_INPUT_BODY: &str = r#"
  var issues = [];

  var areas = document.querySelectorAll('area[href]');
  for (var i = 0; i < areas.length && __amsReal(issues) < CAP; i++) {
    var area = areas[i];
    var alt = (area.getAttribute('alt') || '').trim();
    if (!alt) {
      __amsPush(issues, area, { kind: 'area', selector: __amsCssSelector(area) }, CAP);
    }
  }

  return { issues: issues };
"#;

/// Run the `area-alt` check.
pub async fn check_image_input_rules_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &IMAGE_INPUT_BODY.replace("CAP", &IMAGE_INPUT_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "image-input-rules",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
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
