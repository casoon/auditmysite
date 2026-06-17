//! SPA-Navigation-Announcement journey.
//!
//! Many React/Vue/Svelte/Astro SPA pages navigate via the History API without
//! a real page reload. When that happens screenreaders get no automatic "new
//! page" announcement unless the app explicitly:
//!   1. Updates document.title.
//!   2. Changes the main H1/heading.
//!   3. Moves focus to the new main content area.
//!
//! We detect client-side navigation by intercepting pushState/replaceState
//! before the journey runs, then clicking in-page links and observing whether
//! any of the three announcement signals appear.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use crate::audit::normalized::{InteractiveFinding, JourneyStep, JourneyTrace};
use crate::error::Result;
use crate::interaction::{focus, stability};
use crate::taxonomy::Severity;

/// JS that injects a History-API observer and returns a cleanup handle.
/// Sets `window.__ams_spa_nav_count` to 0, then increments it on each
/// pushState/replaceState call.  `popstate` events are also counted.
const INJECT_SPA_OBSERVER_JS: &str = r#"
(function() {
    window.__ams_spa_nav_count = 0;
    var orig_push = history.pushState.bind(history);
    var orig_replace = history.replaceState.bind(history);
    history.pushState = function() {
        window.__ams_spa_nav_count++;
        return orig_push.apply(this, arguments);
    };
    history.replaceState = function() {
        window.__ams_spa_nav_count++;
        return orig_replace.apply(this, arguments);
    };
    window.addEventListener('popstate', function() {
        window.__ams_spa_nav_count++;
    });
    true;
})()
"#;

/// JS that collects candidate in-page links whose href differs only in path
/// (same origin, not a hash-only jump, not a download link).
const COLLECT_SPA_LINKS_JS: &str = r#"
(function() {
    var origin = window.location.origin;
    var currentHref = window.location.href;
    var links = Array.from(document.querySelectorAll('a[href]'));
    var candidates = [];
    links.forEach(function(a) {
        var href = a.href;
        if (!href.startsWith(origin)) return;         // external
        if (href === currentHref) return;              // same page
        if (a.getAttribute('href').startsWith('#')) return; // hash-only
        if (a.download) return;                        // download
        if (a.target === '_blank') return;             // new tab
        // Only path-different links (ignore query/hash for classification).
        var aPath = new URL(href).pathname;
        var curPath = new URL(currentHref).pathname;
        if (aPath === curPath) return;                 // same path, different query
        candidates.push({ selector: cssPath(a), href: href });
        if (candidates.length >= 5) return;
    });
    function cssPath(el) {
        if (el.id) return '#' + el.id;
        var parts = [];
        var n = el;
        while (n && n.nodeType === 1 && parts.length < 5) {
            var tag = n.nodeName.toLowerCase();
            if (n.id) { parts.unshift(tag + '#' + n.id); break; }
            parts.unshift(tag);
            n = n.parentNode;
        }
        return parts.join(' > ');
    }
    return candidates;
})()
"#;

async fn eval_bool(page: &Page, js: &str) -> Option<bool> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_bool()
}

async fn eval_int(page: &Page, js: &str) -> Option<i64> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_i64()
}

async fn eval_str(page: &Page, js: &str) -> Option<String> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_str().map(|s| s.to_string())
}

/// Collect candidate SPA links. Returns list of (selector, href) pairs.
async fn collect_spa_link_candidates(page: &Page) -> Vec<(String, String)> {
    let params = EvaluateParams::builder()
        .expression(COLLECT_SPA_LINKS_JS.to_string())
        .return_by_value(true)
        .build();
    let Ok(params) = params else {
        return Vec::new();
    };
    let Ok(result) = page.execute(params).await else {
        return Vec::new();
    };
    let Some(val) = result.result.result.value else {
        return Vec::new();
    };
    let Some(arr) = val.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let selector = item.get("selector")?.as_str()?.to_string();
            let href = item.get("href")?.as_str()?.to_string();
            Some((selector, href))
        })
        .collect()
}

/// Run the SPA-navigation journey.
///
/// Returns `None` when no SPA-navigation is detected on the page (most
/// traditional multi-page sites). Only emits findings when navigation *was*
/// detected but announcement signals are missing.
pub async fn run(
    page: &Page,
    initial_url: &str,
) -> Result<Option<(JourneyTrace, Vec<InteractiveFinding>)>> {
    let journey_name = "spa_navigation".to_string();
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };

    // Inject observer before we interact with the page.
    let observer_ok = eval_bool(page, INJECT_SPA_OBSERVER_JS)
        .await
        .unwrap_or(false);
    if !observer_ok {
        tracing::debug!("spa_navigation: observer injection failed");
        return Ok(None);
    }

    // Collect in-page links that look like SPA routes.
    let candidates = collect_spa_link_candidates(page).await;
    if candidates.is_empty() {
        tracing::debug!("spa_navigation: no candidate links found");
        return Ok(None);
    }

    // Capture baseline signals.
    let title_before = eval_str(page, "document.title").await.unwrap_or_default();
    let h1_before = eval_str(
        page,
        "document.querySelector('h1')?.textContent?.trim() ?? ''",
    )
    .await
    .unwrap_or_default();

    trace.steps.push(JourneyStep {
        action: "baseline".to_string(),
        target: None,
        focus: None,
        result: Some(format!("title:{title_before:?}, h1:{h1_before:?}")),
        snapshot_label: Some("before_spa_nav".to_string()),
    });

    // Try each candidate link until we observe a History-API call.
    let mut nav_detected = false;

    for (selector, href) in &candidates {
        let nav_count_before = eval_int(page, "window.__ams_spa_nav_count ?? 0")
            .await
            .unwrap_or(0);

        // Click the link via JS (synthetic — SPA links typically need JS click).
        let click_js = format!(
            r#"(function() {{
                var el = document.querySelector({selector_json});
                if (el) {{ el.click(); return true; }}
                return false;
            }})()"#,
            selector_json = serde_json::to_string(selector).unwrap_or_default()
        );
        let clicked = eval_bool(page, &click_js).await.unwrap_or(false);

        if !clicked {
            continue;
        }

        trace.steps.push(JourneyStep {
            action: "click_spa_link".to_string(),
            target: Some(selector.clone()),
            focus: None,
            result: Some(format!("href:{href}")),
            snapshot_label: Some("after_spa_click".to_string()),
        });

        stability::settle(page).await?;

        let nav_count_after = eval_int(page, "window.__ams_spa_nav_count ?? 0")
            .await
            .unwrap_or(0);

        if nav_count_after > nav_count_before {
            nav_detected = true;
            break;
        }
    }

    if !nav_detected {
        // No History-API call observed — traditional MPA or links are not SPA
        // routes. Not an error.
        tracing::debug!("spa_navigation: no History-API navigation observed on {initial_url}");
        return Ok(None);
    }

    // SPA navigation detected — now check announcement signals.
    stability::settle(page).await?;

    let title_after = eval_str(page, "document.title").await.unwrap_or_default();
    let h1_after = eval_str(
        page,
        "document.querySelector('h1')?.textContent?.trim() ?? ''",
    )
    .await
    .unwrap_or_default();

    let focus_snap = focus::capture_focus(page).await?;
    let focus_sel = focus_snap.selector.clone();
    let focus_on_body = focus_sel.is_none()
        || focus_sel
            .as_deref()
            .map(|s| {
                let l = s.to_lowercase();
                l == "body" || l == "html"
            })
            .unwrap_or(true);

    let title_changed = !title_after.is_empty() && title_after != title_before;
    let heading_changed = !h1_after.is_empty() && h1_after != h1_before;
    let focus_moved = !focus_on_body;

    trace.steps.push(JourneyStep {
        action: "check_spa_announcement".to_string(),
        target: None,
        focus: focus_sel,
        result: Some(format!(
            "title_changed:{title_changed}, heading_changed:{heading_changed}, focus_moved:{focus_moved}"
        )),
        snapshot_label: Some("after_spa_nav".to_string()),
    });

    let mut findings = Vec::new();

    // Violation: none of the three signals are present.
    if !title_changed && !heading_changed && !focus_moved {
        findings.push(InteractiveFinding {
            category: "SpaNavigation".to_string(),
            maps_to_finding: None,
            severity: Severity::High,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_spa_nav".to_string()),
            after_snapshot_label: Some("after_spa_nav".to_string()),
            message: format!(
                "After SPA navigation neither the page title \
                (before: {title_before:?}) nor the H1 heading changed, and focus \
                remained in the same place. Screen readers will not announce \
                the new content."
            ),
            fix_suggestion: Some(
                "After each client-side navigation: (1) update document.title, \
                (2) move focus to the <main> element or the new H1 heading, \
                (3) alternatively populate an aria-live region with the new page name."
                    .to_string(),
            ),
        });
    } else if !title_changed {
        // Partial: heading or focus changed but title didn't.
        findings.push(InteractiveFinding {
            category: "SpaNavigation".to_string(),
            maps_to_finding: None,
            severity: Severity::Medium,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_spa_nav".to_string()),
            after_snapshot_label: Some("after_spa_nav".to_string()),
            message: format!(
                "After SPA navigation document.title remains unchanged ({title_before:?}). \
                Screen readers often primarily announce page transitions via the title."
            ),
            fix_suggestion: Some(
                "Update document.title to the new page name after every client-side navigation."
                    .to_string(),
            ),
        });
    } else if !focus_moved {
        // Title and/or heading changed, but focus stayed — weaker warning.
        findings.push(InteractiveFinding {
            category: "SpaNavigation".to_string(),
            maps_to_finding: None,
            severity: Severity::Medium,
            journey: journey_name.clone(),
            before_snapshot_label: Some("before_spa_nav".to_string()),
            after_snapshot_label: Some("after_spa_nav".to_string()),
            message: "After SPA navigation focus is not moved to the new main area. \
                Keyboard users must manually navigate to the new content."
                .to_string(),
            fix_suggestion: Some(
                "After navigation, move focus to the <main> element or the first \
                H1 heading of the new content."
                    .to_string(),
            ),
        });
    }

    Ok(Some((trace, findings)))
}
