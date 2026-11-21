//! WCAG 3.1.1 — language of same-origin iframe documents.
//!
//! Checks by JavaScript against `contentDocument` whether the `<html>` of
//! each same-origin iframe carries a `lang`, including frames inside open
//! shadow roots. Cross-origin iframes are skipped (they are handled by
//! `frame-tested`). A scan that cannot run is recorded under `html-has-lang`
//! as not run, never as a clean pass.
//!
//! The element-level checks this scan used to carry (image-alt, link-name,
//! button-name, label, duplicate-id) run since #715 as the full frame pass in
//! `audit::frames`: the tree and shared rules against each frame's own AX
//! tree and DOM. The document-level `lang` stays here — the frame pass runs
//! element-level rules only, and `document/lang-missing` is one of the
//! page-level rules it leaves out.

use chromiumoxide::Page;
use tracing::warn;

use crate::wcag::types::{Severity, Violation};
use crate::wcag::WcagLevel;

/// Body of the iframe content scan. Injected after `CSS_SELECTOR_JS` so that
/// `__amsCssSelector` is available for both the outer iframe element and inner
/// elements (the iframe DOM tree is isolated; the main-page stop-guard
/// `cur !== document.documentElement` simply never fires for iframe nodes,
/// and the 5-iteration cap keeps selectors reasonably short).
const IFRAME_SCAN_JS: &str = include_str!("js/iframe_scan.js");

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
                IFRAME_HTML_HAS_LANG_AXE_ID,
                crate::wcag::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            IFRAME_HTML_HAS_LANG_AXE_ID,
            crate::wcag::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(findings) = value.as_array() else {
        return vec![];
    };

    let page_url = page.url().await.ok().flatten().unwrap_or_default();
    findings
        .iter()
        .filter_map(|finding| build_violation(finding, &page_url))
        .collect()
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

fn build_violation(finding: &serde_json::Value, page_url: &str) -> Option<Violation> {
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

    let origin = crate::model::frame_origin(
        page_url,
        &[finding.get("iframe_src").and_then(|v| v.as_str())],
    );
    violation.origin = origin.origin;
    violation.provider = origin.provider;
    violation.frame_host = origin.host;

    if let Some(snippet) = finding.get("snippet").and_then(|v| v.as_str()) {
        if !snippet.is_empty() {
            violation = violation.with_html_snippet(snippet);
        }
    }

    Some(violation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FindingOrigin;

    fn lang_finding(src: &str) -> serde_json::Value {
        serde_json::json!({
            "rule_id": "html-has-lang", "rule": "3.1.1", "severity": "medium",
            "message": "Iframe document is missing a lang attribute on the html element",
            "iframe_selector": "iframe#news", "iframe_src": src,
            "selector": "html", "snippet": ""
        })
    }

    #[test]
    fn a_frame_document_without_lang_carries_the_origin_of_its_frame() {
        let page = "https://www.example.com/";
        let same =
            build_violation(&lang_finding("https://www1.example.com/newsroom"), page).unwrap();
        assert_eq!(same.origin, FindingOrigin::SameSiteFrame);
        assert_eq!(same.provider, None);

        let third =
            build_violation(&lang_finding("https://www.youtube.com/embed/x"), page).unwrap();
        assert_eq!(third.origin, FindingOrigin::CrossOriginFrame);
        assert!(third.provider.is_some());
        assert_eq!(third.frame_host.as_deref(), Some("www.youtube.com"));
        assert_eq!(third.selector.as_deref(), Some("iframe#news [frame] html"));
    }
}
