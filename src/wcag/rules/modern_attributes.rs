//! DOM checks for modern interaction attributes (`popover`, `inert`).
//!
//! Das Popover-Ziel (`popover/target-missing`, `popover/target-invalid`) und
//! der inerte Dialog (`inert/dialog-inert`) laufen seit #691 im geteilten
//! Bestand (siehe `wcag::shared`), der fehlende Name offener Dialoge seit
//! #692 als `dialog/name-missing`. Hier bleibt, was dort keine Entsprechung
//! hat: der fehlende Name offener Menüs und Popover (nur Prüfhinweis -- weder
//! HTML noch WAI-ARIA 1.2 verlangen dort einen Namen) und das offene Popover
//! mit `inert`. Fokus in einem inerten Teilbaum gibt es nicht: HTMLs Focus
//! Fixup nimmt ihn heraus, sobald der Teilbaum inert wird (#783).

use chromiumoxide::Page;
use tracing::warn;

use crate::wcag::types::{Evidence, Outcome, RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

pub(super) const MODERN_ATTRIBUTES_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value - Modern Interaction Attributes",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Modern interaction attributes such as popover and inert must expose valid targets, names, and states",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
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
          // The element's own window: frame content has its own styles.
          const style = el.ownerDocument.defaultView.getComputedStyle(el);
          const rect = el.getBoundingClientRect();
          return style.display !== 'none' &&
            style.visibility !== 'hidden' &&
            rect.width > 0 &&
            rect.height > 0;
        };
        // Every element matching `selector` in the document, in all open
        // shadow roots below it and in same-origin frames: a popover or menu
        // inside a web component or an embedded frame is just as open, inert
        // or unnamed as one in the light DOM (#783). A frame's elements carry
        // the frames.rs selector shape "{iframe} [frame] {element}".
        const deepQuery = selector => {
          const found = [];
          const walk = (root, frameSel) => {
            root.querySelectorAll(selector).forEach(el => found.push({ el, frameSel }));
            root.querySelectorAll('*').forEach(el => {
              if (el.shadowRoot) walk(el.shadowRoot, frameSel);
              if (el.tagName !== 'IFRAME' && el.tagName !== 'FRAME') return;
              let doc = null;
              try { doc = el.contentDocument; } catch (e) {}
              if (!doc || !doc.documentElement) return;
              const sel = __amsCssSelector(el);
              walk(doc, frameSel ? frameSel + ' [frame] ' + sel : sel);
            });
          };
          walk(document, null);
          return found;
        };
        const accessibleName = el => {
          const labelledby = el.getAttribute('aria-labelledby');
          if (labelledby) {
            // IDREFs resolve within the element's own tree (document or
            // shadow root).
            const root = el.getRootNode();
            const text = labelledby
              .split(/\s+/)
              .map(id => root.getElementById(id)?.textContent || '')
              .join(' ')
              .trim();
            if (text) return text;
          }
          return (el.getAttribute('aria-label') || el.getAttribute('title') || '').trim();
        };
        const push = (el, frameSel, issue_type, message, attr, value, outcome) => {
          const sel = __amsCssSelector(el);
          issues.push({
            selector: frameSel ? frameSel + ' [frame] ' + sel : sel,
            issue_type,
            outcome,
            message,
            attr,
            value,
            snippet: el.outerHTML.substring(0, 200)
          });
        };

        deepQuery('[popover], [role="menu"]').forEach(({ el, frameSel }) => {
          const isOpenPopover = el.matches('[popover]:popover-open');
          const isMenu = el.getAttribute('role') === 'menu' && isVisible(el);
          // Ein Dialog -- auch als Popover -- ist `dialog/name-missing` im
          // geteilten Bestand (#692).
          const isDialog = el.matches('dialog, [role="dialog"], [role="alertdialog"]');
          // WAI-ARIA 1.2 requires no name for `menu`, HTML none for a
          // popover: whether the open surface is identifiable without one is a
          // judgement call, never a confirmed violation.
          if ((isOpenPopover || isMenu) && !isDialog && !accessibleName(el)) {
            push(el, frameSel, 'interactive_surface_missing_name', 'Open menu or popover has no accessible name', 'aria-label', null, 'review');
          }
          // Der inerte Dialog ist `inert/dialog-inert`, das sichtbare Menue
          // mit `inert` bewusst kein Befund mehr (#691); das offene Popover
          // deckt die geteilte Regel nicht ab.
          if (isOpenPopover && el.hasAttribute('inert')) {
            push(el, frameSel, 'active_surface_inert', 'Open popover is marked inert', 'inert', '', 'fail');
          }
        });

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
                crate::wcag::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "modern-attributes",
            crate::wcag::WcagLevel::A,
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
            let outcome = if issue.get("outcome").and_then(|v| v.as_str()) == Some("review") {
                Outcome::Review
            } else {
                Outcome::Fail
            };

            let mut violation = Violation::new(
                MODERN_ATTRIBUTES_RULE.id,
                MODERN_ATTRIBUTES_RULE.name,
                MODERN_ATTRIBUTES_RULE.level,
                MODERN_ATTRIBUTES_RULE.severity,
                message,
                &selector,
            )
            .with_selector(&selector)
            .with_kind(outcome)
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
            .with_evidence_item(Evidence::dom_attribute(attr, value));
            violation.message = format!("{message} ({issue_type})");

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}
