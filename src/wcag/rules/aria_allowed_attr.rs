//! WCAG 4.1.2 - ARIA Allowed Attributes
//!
//! Validates that ARIA attributes used on elements are allowed for their role.
//!
//! DOM-level rule: matches against the explicit `role="..."` attribute and
//! the element's own `aria-*` attributes. An earlier tree-based
//! implementation guarded on `prop.name.starts_with("aria-")`, which is
//! never true for CDP AX property names (never prefixed) — 100% dead code,
//! with no DOM fallback (#QA-030). Scope note: only elements with an
//! *explicit* `role` attribute are checked, matching `check_invalid_role`'s
//! scoping — native implicit roles (e.g. a bare `<button>`) are not
//! evaluated here.

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

/// Rule metadata for ARIA allowed attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Allowed Attributes",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes must be allowed for the element's role",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-allowed-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

const ALLOWED_ATTR_CAP: usize = 250;

/// Global ARIA attributes (allowed on any role) and role-specific
/// additions, per the ARIA 1.2 specification. Roles not listed are skipped
/// (no allowed-attr judgement made for roles we have no mapping for).
const ROLE_TABLES_JS: &str = r#"
  var globalAttrs = [
    'aria-atomic', 'aria-busy', 'aria-controls', 'aria-current', 'aria-describedby',
    'aria-details', 'aria-disabled', 'aria-dropeffect', 'aria-errormessage', 'aria-flowto',
    'aria-grabbed', 'aria-haspopup', 'aria-hidden', 'aria-invalid', 'aria-keyshortcuts',
    'aria-label', 'aria-labelledby', 'aria-live', 'aria-owns', 'aria-relevant',
    'aria-roledescription'
  ];
  var roleAttrs = {
    button: ['aria-expanded', 'aria-pressed'],
    link: ['aria-expanded'],
    checkbox: ['aria-checked', 'aria-readonly', 'aria-required'],
    radio: ['aria-checked', 'aria-posinset', 'aria-setsize'],
    textbox: ['aria-activedescendant', 'aria-autocomplete', 'aria-multiline', 'aria-placeholder', 'aria-readonly', 'aria-required'],
    combobox: ['aria-activedescendant', 'aria-autocomplete', 'aria-expanded', 'aria-required'],
    listbox: ['aria-activedescendant', 'aria-expanded', 'aria-multiselectable', 'aria-orientation', 'aria-readonly', 'aria-required'],
    slider: ['aria-orientation', 'aria-readonly', 'aria-valuemax', 'aria-valuemin', 'aria-valuenow', 'aria-valuetext'],
    tab: ['aria-expanded', 'aria-posinset', 'aria-selected', 'aria-setsize'],
    tabpanel: [],
    dialog: ['aria-modal'],
    alert: [],
    img: [],
    heading: ['aria-level'],
    list: [],
    listitem: ['aria-level', 'aria-posinset', 'aria-setsize'],
    navigation: [],
    main: [],
    banner: [],
    contentinfo: [],
    complementary: [],
    form: [],
    search: [],
    menu: ['aria-activedescendant', 'aria-orientation'],
    menuitem: ['aria-expanded', 'aria-posinset', 'aria-setsize'],
    menuitemcheckbox: ['aria-checked', 'aria-posinset', 'aria-setsize'],
    menuitemradio: ['aria-checked', 'aria-posinset', 'aria-setsize'],
    tree: ['aria-activedescendant', 'aria-multiselectable', 'aria-orientation', 'aria-required'],
    treeitem: ['aria-checked', 'aria-expanded', 'aria-level', 'aria-posinset', 'aria-selected', 'aria-setsize'],
    grid: ['aria-activedescendant', 'aria-colcount', 'aria-multiselectable', 'aria-readonly', 'aria-rowcount'],
    gridcell: ['aria-colindex', 'aria-colspan', 'aria-expanded', 'aria-readonly', 'aria-required', 'aria-rowindex', 'aria-rowspan', 'aria-selected'],
    row: ['aria-activedescendant', 'aria-colindex', 'aria-expanded', 'aria-level', 'aria-posinset', 'aria-rowindex', 'aria-selected', 'aria-setsize'],
    columnheader: ['aria-colindex', 'aria-colspan', 'aria-expanded', 'aria-readonly', 'aria-required', 'aria-rowindex', 'aria-rowspan', 'aria-selected', 'aria-sort'],
    rowheader: ['aria-colindex', 'aria-colspan', 'aria-expanded', 'aria-readonly', 'aria-required', 'aria-rowindex', 'aria-rowspan', 'aria-selected', 'aria-sort'],
    progressbar: ['aria-valuemax', 'aria-valuemin', 'aria-valuenow', 'aria-valuetext'],
    scrollbar: ['aria-orientation', 'aria-valuemax', 'aria-valuemin', 'aria-valuenow'],
    spinbutton: ['aria-readonly', 'aria-required', 'aria-valuemax', 'aria-valuemin', 'aria-valuenow', 'aria-valuetext'],
    switch: ['aria-checked', 'aria-readonly', 'aria-required'],
    separator: ['aria-orientation', 'aria-valuemax', 'aria-valuemin', 'aria-valuenow', 'aria-valuetext'],
    toolbar: ['aria-activedescendant', 'aria-orientation']
  };
"#;

const ALLOWED_ATTR_BODY: &str = r#"
  var issues = [];
  var elems = document.querySelectorAll('[role]');
  for (var i = 0; i < elems.length && issues.length < CAP; i++) {
    var el = elems[i];
    var role = (el.getAttribute('role') || '').trim().toLowerCase().split(/\s+/)[0];
    if (!Object.prototype.hasOwnProperty.call(roleAttrs, role)) continue;
    var allowed = roleAttrs[role];

    var attrs = el.attributes;
    for (var a = 0; a < attrs.length; a++) {
      var attrName = attrs[a].name;
      if (attrName.indexOf('aria-') !== 0) continue;
      if (globalAttrs.indexOf(attrName) !== -1) continue;
      if (allowed.indexOf(attrName) !== -1) continue;

      issues.push({ attr: attrName, role: role, selector: __amsCssSelector(el) });
      if (issues.length >= CAP) break;
    }
  }
  return { issues: issues };
"#;

/// Check that ARIA attributes are allowed for each element's explicit role.
pub async fn check_aria_allowed_attr_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        ROLE_TABLES_JS,
        &ALLOWED_ATTR_BODY.replace("CAP", &ALLOWED_ATTR_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("aria-allowed-attr JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "aria-allowed-attr",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure_for(
                "aria-allowed-attr",
                crate::cli::WcagLevel::A,
                "missing_evaluation_value",
            )]
        }
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let attr = issue.get("attr")?.as_str()?;
            let role = issue.get("role")?.as_str()?;
            let selector = issue.get("selector")?.as_str()?.to_string();

            Some(
                Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "ARIA attribute '{}' is not allowed on role '{}'",
                        attr, role
                    ),
                    selector.clone(),
                )
                .with_selector(selector)
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!(
                    "Remove '{}' from this element or change its role",
                    attr
                ))
                .with_help_url(RULE_META.help_url),
            )
        })
        .collect()
}
