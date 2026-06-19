//! Violation enrichment — resolves AX node IDs to human-readable DOM locations
//!
//! After WCAG rules produce violations (which only know AX node IDs), this module
//! uses CDP `DOM.describeNode` with the AXNode's `backend_dom_node_id` to fetch
//! the actual DOM element and writes a useful `selector` string
//! (e.g. `img#banner [src: …/hero.jpg]`, `a.nav-link > svg`) back into each violation.
//!
//! For elements with no id/class/attributes, a second pass uses
//! `DOM.resolveNode` + `Runtime.callFunctionOn` to include parent context
//! so the developer can locate the element in the DOM.

use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, DescribeNodeParams, GetOuterHtmlParams, ResolveNodeParams,
};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::Page;
use tracing::warn;

use super::code_gen::{generate_suggested_code, truncate_html};
use super::tree::AXTree;
use crate::wcag::types::Violation;

// ─── Public entry point ──────────────────────────────────────────────────────

/// Enrich all violations in-place with readable selectors from the live DOM.
///
/// For each violation that has no selector yet, looks up the corresponding AXNode
/// in `ax_tree`, extracts its `backend_dom_node_id`, and calls `DOM.describeNode`
/// via CDP to get the element's tag name and attributes. Contrast violations
/// (1.4.3) already carry selectors from the style extractor and are skipped.
pub async fn enrich_violations_with_page(
    page: &Page,
    violations: &mut [Violation],
    ax_tree: &AXTree,
) {
    for violation in violations.iter_mut() {
        if violation.selector.is_some() {
            continue; // already enriched (e.g. contrast rule)
        }

        let backend_id = match ax_tree
            .get_node(&violation.node_id)
            .and_then(|n| n.backend_dom_node_id)
        {
            Some(id) => id,
            // AX node exists in tree but has no DOM node ID — ghost element.
            // Demote to warning: cannot be verified or located by a developer.
            None => {
                violation.kind = crate::wcag::types::FindingKind::Warning;
                continue;
            }
        };

        match describe_node_selector(page, backend_id).await {
            Some(sel) => {
                violation
                    .evidence
                    .push(crate::wcag::types::ViolationEvidence::ax_tree(&sel));
                violation.selector = Some(sel);
            }
            None => {
                warn!(
                    "Could not resolve backend DOM node {} for violation {}",
                    backend_id, violation.rule
                );
                // Element not locatable in live DOM — cannot be confirmed.
                violation.kind = crate::wcag::types::FindingKind::Warning;
            }
        }

        // Fetch outer HTML and generate a concrete code fix
        if let Some(raw_html) = get_outer_html(page, backend_id).await {
            // Two 1.1.1 false-positive classes are demoted to warnings so they do
            // not inflate violation counts or scoring (#487):
            //   1. Lazy-load <img> placeholders (data-src/data-srcset/lazyload)
            //      receive their real src+alt only on scroll, which a headless
            //      audit never triggers.
            //   2. Decorative inline SVG icons (<svg><use …> sprite refs with no
            //      <title>/aria-label) — almost always UI chrome that should carry
            //      aria-hidden. A meaningful SVG has a title/label and thus an
            //      accessible name, so it never reaches this check.
            if violation.rule == "1.1.1"
                && (is_lazyload_image_placeholder(&raw_html) || is_decorative_svg_icon(&raw_html))
            {
                violation.kind = crate::wcag::types::FindingKind::Warning;
            }
            let snippet = truncate_html(raw_html);
            let suggested = generate_suggested_code(
                &violation.rule,
                Some(&snippet),
                violation.role.as_deref(),
                violation.fix_suggestion.as_deref(),
            );
            violation.html_snippet = Some(snippet);
            violation.suggested_code = suggested;
        } else {
            // No live DOM available — still try to generate a template fix
            violation.suggested_code = generate_suggested_code(
                &violation.rule,
                None,
                violation.role.as_deref(),
                violation.fix_suggestion.as_deref(),
            );
        }
    }
}

/// Detects JS lazy-load `<img>` placeholders whose real source/alt is deferred to
/// a `data-*` attribute and only swapped in on scroll. Native `loading="lazy"`
/// images keep a real `src`/`alt` and are intentionally *not* matched here, so a
/// genuinely missing alt on a native-lazy image still counts (#487).
fn is_lazyload_image_placeholder(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    if !lower.trim_start().starts_with("<img") {
        return false;
    }
    const LAZY_MARKERS: &[&str] = &[
        "data-src",
        "data-srcset",
        "data-lazy",
        "data-original",
        "lazyload",
    ];
    LAZY_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// Detects decorative inline SVG icons: an `<svg>` that references a sprite via
/// `<use>` and carries no `<title>` or `aria-label`. These are UI chrome that
/// should be `aria-hidden`; flagging each as a missing-alt 1.1.1 barrier inflates
/// counts on icon-heavy sites. A meaningful SVG exposes a name (title/aria-label)
/// and therefore never reaches the 1.1.1 check at all (#487).
fn is_decorative_svg_icon(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    lower.trim_start().starts_with("<svg")
        && lower.contains("<use")
        && !lower.contains("<title")
        && !lower.contains("aria-label")
}

// ─── CDP element resolution ───────────────────────────────────────────────────

/// Call CDP `DOM.describeNode` for the given backend node ID and build a
/// human-readable selector string from the returned element data.
/// Falls back to a parent-context lookup for bare elements with no id/class/attrs.
async fn describe_node_selector(page: &Page, backend_node_id: i64) -> Option<String> {
    let params = DescribeNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();

    let response = page
        .execute(params)
        .await
        .map_err(|e| warn!("DOM.describeNode failed: {}", e))
        .ok()?;

    let node = &response.node;
    let tag = node.node_name.to_lowercase();

    // Attributes arrive as a flat array: [name0, val0, name1, val1, …]
    let attrs: Vec<String> = node.attributes.as_deref().unwrap_or(&[]).to_vec();
    let attr_pairs: Vec<(&str, &str)> = attrs
        .chunks(2)
        .filter_map(|p| {
            if p.len() == 2 {
                Some((p[0].as_str(), p[1].as_str()))
            } else {
                None
            }
        })
        .collect();

    let get = |name: &str| -> Option<&str> {
        attr_pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    };

    // Build base selector: tag + id, or tag + first class
    let base = if let Some(id) = get("id").filter(|v| !v.is_empty()) {
        format!("{}#{}", tag, id)
    } else if let Some(cls) = get("class").filter(|v| !v.is_empty()) {
        let first = cls.split_whitespace().next().unwrap_or("");
        format!("{}.{}", tag, first)
    } else {
        tag.clone()
    };

    // Append a short contextual hint for common element types
    let hint = match tag.as_str() {
        "img" => get("src").map(|s| format!("src: {}", short_url(s))),
        "a" => get("href").map(|s| format!("href: {}", short_url(s))),
        "input" => get("type")
            .map(|t| format!("type={}", t))
            .or_else(|| get("name").map(|n| format!("name={}", n))),
        "svg" => get("viewBox")
            .map(|vb| format!("viewBox={}", vb))
            .or_else(|| get("width").map(|w| format!("width={}", w))),
        _ => None,
    };

    let selector = if let Some(h) = hint {
        format!("{} [{}]", base, h)
    } else {
        base.clone()
    };

    // If we have just a bare tag (no distinguishing info), try to get parent context
    // via DOM.resolveNode + Runtime.callFunctionOn so the developer can locate it.
    if selector == tag {
        if let Some(ctx) = parent_context_selector(page, backend_node_id).await {
            return Some(ctx);
        }
    }

    Some(selector)
}

/// Use `DOM.resolveNode` → `Runtime.callFunctionOn` to run JS on the element
/// and return `"parentSel > tag"` (e.g. `a.nav-link > svg`).
async fn parent_context_selector(page: &Page, backend_node_id: i64) -> Option<String> {
    // Resolve backend node ID to a JS remote object
    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();

    let resolved = page
        .execute(resolve)
        .await
        .map_err(|e| warn!("DOM.resolveNode failed: {}", e))
        .ok()?;

    let object_id = resolved.object.object_id.clone()?;

    // Run JS on the element: returns a precise path up to 3 levels deep,
    // including :nth-of-type when siblings of the same tag exist.
    let js = r#"function() {
        function seg(el) {
            if (!el) return '';
            const tag = el.tagName.toLowerCase();
            if (el.id) return tag + '#' + el.id;
            const cls = (typeof el.className === 'string' && el.className.trim())
                ? '.' + el.className.trim().split(/\s+/)[0]
                : '';
            // Add :nth-of-type when there are multiple siblings with the same tag
            const p = el.parentElement;
            if (p) {
                const siblings = Array.from(p.children).filter(c => c.tagName === el.tagName);
                if (siblings.length > 1) {
                    const idx = siblings.indexOf(el) + 1;
                    return tag + cls + ':nth-of-type(' + idx + ')';
                }
            }
            return tag + cls;
        }
        const parts = [];
        let cur = this;
        for (let i = 0; i < 3 && cur && cur !== document.documentElement && cur !== document.body; i++) {
            parts.unshift(seg(cur));
            // Stop early if this segment is already unique (has id or unique class)
            if (cur.id) break;
            cur = cur.parentElement;
        }
        return parts.join(' > ');
    }"#;

    let call = match CallFunctionOnParams::builder()
        .function_declaration(js)
        .object_id(object_id)
        .return_by_value(true)
        .build()
    {
        Ok(p) => p,
        Err(e) => {
            warn!("CallFunctionOn build failed: {}", e);
            return None;
        }
    };

    let result = page
        .execute(call)
        .await
        .map_err(|e| warn!("Runtime.callFunctionOn failed: {}", e))
        .ok()?;

    result
        .result
        .result
        .value
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Call CDP `DOM.getOuterHTML` for the given backend node ID.
/// Returns the raw HTML string, or `None` on failure.
async fn get_outer_html(page: &Page, backend_node_id: i64) -> Option<String> {
    let params = GetOuterHtmlParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();

    let response = page
        .execute(params)
        .await
        .map_err(|e| warn!("DOM.getOuterHTML failed: {}", e))
        .ok()?;

    let html = response.outer_html.trim().to_string();
    if html.is_empty() {
        None
    } else {
        Some(html)
    }
}

/// Truncate a URL to at most 55 characters, keeping the tail.
fn short_url(s: &str) -> String {
    if s.len() > 55 {
        format!("\u{2026}{}", &s[s.len() - 52..])
    } else {
        s.to_string()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_url_truncates_long_src() {
        let long = "https://example.com/assets/images/hero-banner-2024-large.jpg";
        let result = short_url(long);
        assert!(result.starts_with('\u{2026}'));
        assert!(result.chars().count() <= 53);
    }

    #[test]
    fn short_url_keeps_short_src() {
        let short = "/img/logo.png";
        assert_eq!(short_url(short), short);
    }

    #[test]
    fn lazyload_placeholder_detection() {
        // JS lazy-load placeholders → demoted.
        assert!(is_lazyload_image_placeholder(
            r#"<img class="lazyload" alt="" src="data:image/svg+xml,...">"#
        ));
        assert!(is_lazyload_image_placeholder(
            r#"<img data-src="/real.jpg" src="/placeholder.gif">"#
        ));
        assert!(is_lazyload_image_placeholder(
            r#"<img data-srcset="/real-2x.jpg 2x">"#
        ));
        // Native lazy + real image, and non-img elements → not matched.
        assert!(!is_lazyload_image_placeholder(
            r#"<img loading="lazy" src="/real.jpg">"#
        ));
        assert!(!is_lazyload_image_placeholder(r#"<img src="/real.jpg">"#));
        assert!(!is_lazyload_image_placeholder(
            r#"<div data-src="/x.jpg"></div>"#
        ));
    }

    #[test]
    fn decorative_svg_icon_detection() {
        // Sprite-ref icon with no title/label → decorative.
        assert!(is_decorative_svg_icon(
            r##"<svg width="16" height="16"><use xlink:href="#spon-text-m"></use></svg>"##
        ));
        // Labelled / titled SVGs are meaningful → not demoted.
        assert!(!is_decorative_svg_icon(
            r##"<svg aria-label="Search"><use xlink:href="#search"></use></svg>"##
        ));
        assert!(!is_decorative_svg_icon(
            r#"<svg><title>Chart</title><path d="…"/></svg>"#
        ));
        // Not an SVG / no sprite ref.
        assert!(!is_decorative_svg_icon(r#"<img src="/x.jpg">"#));
        assert!(!is_decorative_svg_icon(r#"<svg><path d="…"/></svg>"#));
    }
}
