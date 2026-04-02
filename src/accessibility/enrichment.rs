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

use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, DescribeNodeParams, ResolveNodeParams};
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::Page;
use tracing::warn;

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
    violations: &mut Vec<Violation>,
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
            None => continue,
        };

        match describe_node_selector(page, backend_id).await {
            Some(sel) => violation.selector = Some(sel),
            None => warn!(
                "Could not resolve backend DOM node {} for violation {}",
                backend_id, violation.rule
            ),
        }
    }
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
}
