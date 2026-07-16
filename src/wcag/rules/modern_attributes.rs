//! DOM checks for modern interaction attributes (`popover`, `inert`).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, ViolationEvidence};

pub const MODERN_ATTRIBUTES_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value - Modern Interaction Attributes",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Modern interaction attributes such as popover and inert must expose valid targets, names, and states",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "modern-attribute-misuse",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Check common misuse patterns for `popover` and `inert`.
pub async fn check_modern_attributes_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r##"
        const issues = [];
        const isVisible = el => {
          const style = window.getComputedStyle(el);
          const rect = el.getBoundingClientRect();
          return style.display !== 'none' &&
            style.visibility !== 'hidden' &&
            rect.width > 0 &&
            rect.height > 0;
        };
        const accessibleName = el => {
          const labelledby = el.getAttribute('aria-labelledby');
          if (labelledby) {
            const text = labelledby
              .split(/\s+/)
              .map(id => document.getElementById(id)?.textContent || '')
              .join(' ')
              .trim();
            if (text) return text;
          }
          return (el.getAttribute('aria-label') || el.getAttribute('title') || '').trim();
        };
        const push = (el, issue_type, message, attr, value) => {
          issues.push({
            selector: __amsCssSelector(el),
            issue_type,
            message,
            attr,
            value,
            snippet: el.outerHTML.substring(0, 200)
          });
        };

        document.querySelectorAll('[popovertarget]').forEach(trigger => {
          const id = trigger.getAttribute('popovertarget');
          const target = id ? document.getElementById(id) : null;
          if (!target) {
            push(trigger, 'popover_target_missing', 'Popover trigger references a missing target', 'popovertarget', id || '');
          } else if (!target.hasAttribute('popover')) {
            push(trigger, 'popover_target_invalid', 'Popover trigger references an element without the popover attribute', 'popovertarget', id);
          }
        });

        document.querySelectorAll('[popover], dialog, [role="dialog"], [role="menu"]').forEach(el => {
          const isOpenPopover = el.matches('[popover]:popover-open');
          const isOpenDialog = el.matches('dialog[open]');
          const isAriaDialog = el.getAttribute('role') === 'dialog' && isVisible(el);
          const isMenu = el.getAttribute('role') === 'menu' && isVisible(el);
          if ((isOpenPopover || isOpenDialog || isAriaDialog || isMenu) && !accessibleName(el)) {
            push(el, 'interactive_surface_missing_name', 'Open dialog, menu, or popover has no accessible name', 'aria-label', null);
          }
          if ((isOpenPopover || isOpenDialog || isAriaDialog || isMenu) && el.hasAttribute('inert')) {
            push(el, 'active_surface_inert', 'Active dialog, menu, or popover is marked inert', 'inert', '');
          }
        });

        const active = document.activeElement;
        const inertAncestor = active && active.closest ? active.closest('[inert]') : null;
        if (inertAncestor) {
          push(inertAncestor, 'focus_inside_inert', 'Keyboard focus is currently inside an inert subtree', 'inert', '');
        }

        return issues;
        "##,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("modern attributes DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "modern-attributes",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "modern-attributes",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let issue_type = issue
                .get("issue_type")
                .and_then(|v| v.as_str())
                .unwrap_or("modern_attribute_misuse");
            let message = issue
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or(MODERN_ATTRIBUTES_RULE.description);
            let attr = issue.get("attr").and_then(|v| v.as_str()).unwrap_or("");
            let value = issue
                .get("value")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            let mut violation = Violation::new(
                MODERN_ATTRIBUTES_RULE.id,
                MODERN_ATTRIBUTES_RULE.name,
                MODERN_ATTRIBUTES_RULE.level,
                MODERN_ATTRIBUTES_RULE.severity,
                message,
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(MODERN_ATTRIBUTES_RULE.axe_id)
            .with_tags(
                MODERN_ATTRIBUTES_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix("Fix the popover/inert attributes so the active surface has a valid target, accessible name, and operable state.")
            .with_help_url(MODERN_ATTRIBUTES_RULE.help_url)
            .with_evidence_item(ViolationEvidence::dom_attribute(attr, value));
            violation.message = format!("{message} ({issue_type})");

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}
