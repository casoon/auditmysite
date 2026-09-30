//! WCAG 3.1.1 — language of same-origin iframe documents.
//!
//! Checks by JavaScript against `contentDocument` whether the `<html>` of
//! each same-origin iframe carries a `lang`. Cross-origin iframes are
//! skipped (they are handled by `frame-tested`).
//!
//! The element-level checks this scan used to carry (image-alt, link-name,
//! button-name, label, duplicate-id) run since #715 as the full frame pass in
//! `audit::frames`: the tree and shared rules against each frame's own AX
//! tree and DOM. The document-level `lang` stays here — the frame pass runs
//! element-level rules only, and `document/lang-missing` is one of the
//! page-level rules it leaves out.

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

/// Body of the iframe content scan. Injected after `CSS_SELECTOR_JS` so that
/// `__amsCssSelector` is available for both the outer iframe element and inner
/// elements (the iframe DOM tree is isolated; the main-page stop-guard
/// `cur !== document.documentElement` simply never fires for iframe nodes,
/// and the 5-iteration cap keeps selectors reasonably short).
const IFRAME_SCAN_JS: &str = r#"
  var iframes = document.querySelectorAll('iframe, frame');
  var results = [];

  for (var fi = 0; fi < iframes.length; fi++) {
    var iframe = iframes[fi];
    var iframeRole = (iframe.getAttribute('role') || '').toLowerCase();
    if (iframeRole === 'none' || iframeRole === 'presentation') continue;
    if (iframe.hasAttribute('hidden') || iframe.getAttribute('aria-hidden') === 'true') continue;
    var iframeStyle = window.getComputedStyle(iframe);
    if (iframeStyle && (iframeStyle.display === 'none' ||
        iframeStyle.visibility === 'hidden' || iframeStyle.visibility === 'collapse')) continue;
    var iframeRect = iframe.getBoundingClientRect();
    if (iframeRect.width <= 1 || iframeRect.height <= 1) continue;

    var iframeDoc;
    try {
      iframeDoc = iframe.contentDocument;
      if (!iframeDoc || !iframeDoc.body) continue;
    } catch(e) {
      continue;
    }

    var ifSel = __amsCssSelector(iframe);
    var ifSrc = iframe.getAttribute('src') || '';

    // 3.1.1 html-has-lang
    var iframeHtmlEl = iframeDoc.documentElement;
    if (iframeHtmlEl && !(iframeHtmlEl.getAttribute('lang') || '').trim()) {
      results.push({
        rule_id: 'html-has-lang', rule: '3.1.1', severity: 'medium',
        message: 'Iframe document is missing a lang attribute on the html element',
        iframe_selector: ifSel, iframe_src: ifSrc,
        selector: 'html', snippet: ''
      });
    }
  }

  return results;
"#;

pub async fn check_same_origin_iframes_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        IFRAME_SCAN_JS,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("iframe content scan JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "same-origin-iframe",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "same-origin-iframe",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(findings) = value.as_array() else {
        return vec![];
    };

    findings.iter().filter_map(build_violation).collect()
}

/// Die 3.1.1-Kennung fuer das Dokument *im* iframe.
///
/// Bis zur Umstellung auf die geteilten Regeln deklarierte `language.rs` diese
/// Kennung ueber sein `RuleMetadata`, und das kanonische Inventar
/// (`tests/common/rule_inventory.rs`) fand sie dort. Die Regel fuer das
/// Hauptdokument heisst jetzt `document/lang-missing`; `html-has-lang` wird
/// nur noch hier erzeugt. Ohne diese Deklaration koennte auditmysite eine
/// Kennung melden, die im Inventar nicht vorkommt.
const IFRAME_HTML_HAS_LANG_AXE_ID: &str = "html-has-lang";

fn build_violation(finding: &serde_json::Value) -> Option<Violation> {
    let rule_id = finding.get("rule_id")?.as_str()?;
    let iframe_selector = finding
        .get("iframe_selector")
        .and_then(|v| v.as_str())
        .unwrap_or("iframe");
    let inner_selector = finding
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let composite_selector = if inner_selector.is_empty() || inner_selector == "html" {
        format!("{iframe_selector} [frame] {inner_selector}")
            .trim_end()
            .to_string()
    } else {
        format!("{iframe_selector} [frame] {inner_selector}")
    };
    let message = finding
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Accessibility issue in iframe content");

    let (rule, name, level, severity, fix, help_url) = match rule_id {
        IFRAME_HTML_HAS_LANG_AXE_ID => (
            "3.1.1",
            "Language of Page",
            WcagLevel::A,
            Severity::Medium,
            "Add a lang attribute to the html element inside the iframe (e.g. lang=\"en\").",
            "https://www.w3.org/WAI/WCAG22/Understanding/language-of-page.html",
        ),
        _ => return None,
    };

    let mut violation = Violation::new(rule, name, level, severity, message, &composite_selector)
        .with_selector(&composite_selector)
        .with_rule_id(rule_id)
        .with_tags(vec!["wcag2a".to_string(), "iframe-content".to_string()])
        .with_fix(fix)
        .with_help_url(help_url);

    if let Some(snippet) = finding.get("snippet").and_then(|v| v.as_str()) {
        if !snippet.is_empty() {
            violation = violation.with_html_snippet(snippet);
        }
    }

    Some(violation)
}
