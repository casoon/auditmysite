//! WCAG 4.1.2 - ARIA Valid Attribute Values (aria-valid-attr-value)
//!
//! ARIA attributes must contain values that are valid for their type:
//! - Boolean attributes: only "true" or "false"
//! - Tristate attributes: "true", "false", or "mixed"
//! - Token-list attributes: one of a defined set of allowed tokens
//! - ID-ref attributes: must reference existing DOM elements
//!
//! Level A

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Valid Attribute Values",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes must have valid values for their type",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-valid-attr-value",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

const ARIA_VALID_ATTR_VALUE_JS: &str = r#"
(function() {
  // Pure booleans: only "true" or "false"
  var booleanAttrs = [
    'aria-disabled', 'aria-expanded', 'aria-hidden', 'aria-modal',
    'aria-multiline', 'aria-multiselectable', 'aria-readonly',
    'aria-required', 'aria-selected'
  ];

  // Tristate: "true", "false", "mixed"
  var tristateAttrs = ['aria-checked', 'aria-pressed'];

  // Token-list attributes and allowed values
  var tokenAttrs = {
    'aria-autocomplete': ['inline', 'list', 'both', 'none'],
    'aria-current': ['page', 'step', 'location', 'date', 'time', 'true', 'false'],
    'aria-dropeffect': ['copy', 'execute', 'link', 'move', 'none', 'popup'],
    'aria-haspopup': ['false', 'true', 'menu', 'listbox', 'tree', 'grid', 'dialog'],
    'aria-invalid': ['grammar', 'false', 'spelling', 'true'],
    'aria-live': ['assertive', 'off', 'polite'],
    'aria-orientation': ['horizontal', 'vertical', 'undefined'],
    'aria-relevant': ['additions', 'all', 'removals', 'text'],
    'aria-sort': ['ascending', 'descending', 'none', 'other']
  };

  // ID-ref attributes: referenced IDs must exist in the DOM
  var idRefAttrs = [
    'aria-activedescendant', 'aria-controls', 'aria-describedby',
    'aria-details', 'aria-errormessage', 'aria-flowto', 'aria-labelledby',
    'aria-owns'
  ];

  var violations = [];

  document.querySelectorAll('*').forEach(function(el) {
    var attrs = el.attributes;
    if (!attrs) return;

    for (var i = 0; i < attrs.length; i++) {
      var name = attrs[i].name;
      if (!name.startsWith('aria-')) continue;
      var val = attrs[i].value;

      if (booleanAttrs.indexOf(name) !== -1) {
        if (val !== 'true' && val !== 'false') {
          violations.push({
            attr: name,
            value: val,
            expected: '"true" or "false"',
            tag: el.tagName.toLowerCase(),
            id: el.id || null,
            cls: el.className ? el.className.split(' ')[0] : null
          });
        }
      } else if (tristateAttrs.indexOf(name) !== -1) {
        if (val !== 'true' && val !== 'false' && val !== 'mixed') {
          violations.push({
            attr: name,
            value: val,
            expected: '"true", "false", or "mixed"',
            tag: el.tagName.toLowerCase(),
            id: el.id || null,
            cls: el.className ? el.className.split(' ')[0] : null
          });
        }
      } else if (tokenAttrs[name]) {
        var lower = val.trim().toLowerCase();
        if (tokenAttrs[name].indexOf(lower) === -1) {
          violations.push({
            attr: name,
            value: val,
            expected: tokenAttrs[name].join(', '),
            tag: el.tagName.toLowerCase(),
            id: el.id || null,
            cls: el.className ? el.className.split(' ')[0] : null
          });
        }
      } else if (idRefAttrs.indexOf(name) !== -1) {
        var ids = val.trim().split(/\s+/);
        var broken = ids.filter(function(id) {
          return id.length > 0 && !document.getElementById(id);
        });
        if (broken.length > 0) {
          violations.push({
            attr: name,
            value: val,
            expected: 'existing DOM ' + (broken.length === 1 ? 'id' : 'ids') + ': ' + broken.join(', ') + ' not found',
            tag: el.tagName.toLowerCase(),
            id: el.id || null,
            cls: el.className ? el.className.split(' ')[0] : null
          });
        }
      }
    }
  });

  return { violations: violations };
})()
"#;

fn make_selector(tag: &str, id: Option<&str>, cls: Option<&str>) -> String {
    let mut sel = tag.to_string();
    if let Some(id) = id.filter(|s| !s.is_empty()) {
        sel.push('#');
        sel.push_str(id);
    } else if let Some(cls) = cls.filter(|s| !s.is_empty()) {
        sel.push('.');
        sel.push_str(cls);
    }
    sel
}

pub async fn check_aria_valid_attr_value_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(ARIA_VALID_ATTR_VALUE_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let items = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    items
        .iter()
        .map(|item| {
            let attr = item.get("attr").and_then(|v| v.as_str()).unwrap_or("?");
            let value = item.get("value").and_then(|v| v.as_str()).unwrap_or("?");
            let expected = item.get("expected").and_then(|v| v.as_str()).unwrap_or("?");
            let tag = item.get("tag").and_then(|v| v.as_str()).unwrap_or("?");
            let id = item.get("id").and_then(|v| v.as_str());
            let cls = item.get("cls").and_then(|v| v.as_str());

            let selector = make_selector(tag, id, cls);

            Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                format!(
                    "'{}' has invalid value \"{}\", expected: {}",
                    attr, value, expected
                ),
                selector.clone(),
            )
            .with_selector(selector)
            .with_fix(format!(
                "Correct the value of '{}'. Expected: {}.",
                attr, expected
            ))
            .with_rule_id(RULE_META.axe_id)
            .with_help_url(RULE_META.help_url)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_meta_is_level_a() {
        assert_eq!(RULE_META.level, WcagLevel::A);
        assert_eq!(RULE_META.axe_id, "aria-valid-attr-value");
    }

    #[test]
    fn make_selector_with_id() {
        assert_eq!(
            make_selector("button", Some("my-btn"), None),
            "button#my-btn"
        );
    }

    #[test]
    fn make_selector_with_class() {
        assert_eq!(make_selector("div", None, Some("nav")), "div.nav");
    }

    #[test]
    fn make_selector_tag_only() {
        assert_eq!(make_selector("span", None, None), "span");
    }
}
