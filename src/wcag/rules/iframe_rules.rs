//! WCAG 1.1.1, 1.3.1, 2.4.4, 3.1.1, 4.1.1, 4.1.2 — same-origin iframe content.
//!
//! Runs a set of focused WCAG checks inside each same-origin iframe by
//! executing JavaScript against `contentDocument`. Cross-origin iframes are
//! skipped (they are handled by `frame-tested`).
//!
//! Checks performed inside each accessible iframe:
//! - 1.1.1 / image-alt      — images without alt attribute
//! - 2.4.4 / link-name      — links without accessible text
//! - 4.1.2 / button-name    — buttons without accessible name
//! - 1.3.1 / label          — form inputs without associated label
//! - 4.1.1 / duplicate-id   — duplicate IDs within the iframe document
//! - 3.1.1 / html-has-lang  — missing lang on the iframe's html element

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
  function __iframeAccessibleName(el, doc) {
    var label = (el.getAttribute('aria-label') || '').trim();
    if (label) return label;
    var lbId = (el.getAttribute('aria-labelledby') || '').trim();
    if (lbId) {
      var text = lbId.split(/\s+/).map(function(id) {
        var ref = doc.getElementById(id);
        return ref ? ref.textContent.trim() : '';
      }).join(' ').trim();
      if (text) return text;
    }
    var elId = el.id;
    if (elId) {
      var safeId = elId.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
      var labelEl = doc.querySelector('label[for="' + safeId + '"]');
      if (labelEl && labelEl.textContent.trim()) return labelEl.textContent.trim();
    }
    var p = el.parentElement;
    while (p) {
      if (p.tagName && p.tagName.toLowerCase() === 'label') {
        var t = p.textContent.trim();
        if (t) return t;
      }
      p = p.parentElement;
    }
    return (el.getAttribute('title') || '').trim();
  }

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

    // 1.1.1 image-alt
    var imgs = iframeDoc.querySelectorAll('img');
    for (var ii = 0; ii < imgs.length; ii++) {
      var img = imgs[ii];
      var imgRole = (img.getAttribute('role') || '').toLowerCase();
      if (imgRole === 'none' || imgRole === 'presentation') continue;
      if (img.getAttribute('aria-hidden') === 'true') continue;
      if (img.getAttribute('alt') === null) {
        results.push({
          rule_id: 'image-alt', rule: '1.1.1', severity: 'high',
          message: 'Image inside iframe is missing an alt attribute',
          iframe_selector: ifSel, iframe_src: ifSrc,
          selector: __amsCssSelector(img),
          snippet: img.outerHTML.substring(0, 200)
        });
      }
    }

    // 4.1.2 button-name
    var btns = iframeDoc.querySelectorAll('button, [role="button"]');
    for (var bi = 0; bi < btns.length; bi++) {
      var btn = btns[bi];
      if (btn.getAttribute('aria-hidden') === 'true') continue;
      if (__iframeAccessibleName(btn, iframeDoc)) continue;
      if ((btn.textContent || '').trim()) continue;
      results.push({
        rule_id: 'button-name', rule: '4.1.2', severity: 'critical',
        message: 'Button inside iframe is missing an accessible name',
        iframe_selector: ifSel, iframe_src: ifSrc,
        selector: __amsCssSelector(btn),
        snippet: btn.outerHTML.substring(0, 200)
      });
    }

    // 2.4.4 link-name
    var links = iframeDoc.querySelectorAll('a[href]');
    for (var li = 0; li < links.length; li++) {
      var link = links[li];
      if (link.getAttribute('aria-hidden') === 'true') continue;
      if (__iframeAccessibleName(link, iframeDoc)) continue;
      if ((link.textContent || '').trim()) continue;
      var imgWithAlt = link.querySelector('img[alt]');
      if (imgWithAlt && (imgWithAlt.getAttribute('alt') || '').trim()) continue;
      results.push({
        rule_id: 'link-name', rule: '2.4.4', severity: 'high',
        message: 'Link inside iframe is missing accessible text',
        iframe_selector: ifSel, iframe_src: ifSrc,
        selector: __amsCssSelector(link),
        snippet: link.outerHTML.substring(0, 200)
      });
    }

    // 1.3.1 label
    var inputs = iframeDoc.querySelectorAll(
      'input:not([type="hidden"]):not([type="submit"]):not([type="button"])' +
      ':not([type="reset"]):not([type="image"]), select, textarea'
    );
    for (var ini = 0; ini < inputs.length; ini++) {
      var input = inputs[ini];
      if (input.getAttribute('aria-hidden') === 'true') continue;
      if (__iframeAccessibleName(input, iframeDoc)) continue;
      results.push({
        rule_id: 'label', rule: '1.3.1', severity: 'critical',
        message: 'Form input inside iframe is missing an associated label',
        iframe_selector: ifSel, iframe_src: ifSrc,
        selector: __amsCssSelector(input),
        snippet: input.outerHTML.substring(0, 200)
      });
    }

    // 4.1.1 duplicate-id
    var allIdEls = iframeDoc.querySelectorAll('[id]');
    var seenIds = {};
    var reportedIds = {};
    for (var di = 0; di < allIdEls.length; di++) {
      var idVal = allIdEls[di].id;
      if (!idVal) continue;
      if (idVal in seenIds) {
        if (!(idVal in reportedIds)) {
          results.push({
            rule_id: 'duplicate-id', rule: '4.1.1', severity: 'critical',
            message: 'Duplicate id "' + idVal + '" inside iframe',
            iframe_selector: ifSel, iframe_src: ifSrc,
            selector: __amsCssSelector(allIdEls[di]),
            snippet: allIdEls[di].outerHTML.substring(0, 200)
          });
          reportedIds[idVal] = true;
        }
      } else {
        seenIds[idVal] = true;
      }
    }

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
            return vec![];
        }
    };

    let Some(value) = result.value() else {
        return vec![];
    };
    let Some(findings) = value.as_array() else {
        return vec![];
    };

    findings.iter().filter_map(build_violation).collect()
}

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
        "image-alt" => (
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Add a descriptive alt attribute. Use alt=\"\" for decorative images.",
            "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
        ),
        "button-name" => (
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::Critical,
            "Add visible text, aria-label, or aria-labelledby to the button.",
            "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
        ),
        "link-name" => (
            "2.4.4",
            "Link Purpose (In Context)",
            WcagLevel::A,
            Severity::High,
            "Add descriptive text inside the link, or use aria-label to describe its destination.",
            "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context.html",
        ),
        "label" => (
            "1.3.1",
            "Info and Relationships",
            WcagLevel::A,
            Severity::Critical,
            "Associate a <label> element using for/id, or use aria-label on the input.",
            "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
        ),
        "duplicate-id" => (
            "4.1.1",
            "Parsing",
            WcagLevel::A,
            Severity::Critical,
            "Ensure all id attributes are unique within the iframe document.",
            "https://www.w3.org/WAI/WCAG21/Understanding/parsing.html",
        ),
        "html-has-lang" => (
            "3.1.1",
            "Language of Page",
            WcagLevel::A,
            Severity::Medium,
            "Add a lang attribute to the html element inside the iframe (e.g. lang=\"en\").",
            "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
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
