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
//!
//! # Gemessen wird an Aufnahmen, nicht an `querySelector`
//!
//! Die drei Signale kommen aus der Differenz zweier [`AXSnapshot`]s um den
//! Klick herum (Plan 53): Titel und URL trägt die Aufnahme selbst, die
//! Überschrift wird aus dem Accessibility-Tree gelesen statt über
//! `document.querySelector('h1')`, und die Fokusbewegung über die
//! Backend-Node-ID statt über „heißt der Selektor `body`".
//!
//! Ein clientseitiger Routenwechsel **ersetzt das Dokument nicht** — die
//! Backend-IDs bleiben über ihn hinweg gültig. Das unterscheidet diesen Fall
//! von einem echten Seitenwechsel, den `AXTreeDiff::url_changed` zusätzlich
//! ausweist.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;

use super::eval_bool;
use crate::accessibility::{AXSnapshot, AXTree, AXTreeDiff};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::stability;
use crate::taxonomy::Severity;

/// Die Texte der Überschriften erster Ebene, aus dem Accessibility-Tree.
///
/// `document.querySelector('h1')` liest nur das erste `<h1>` des Light DOM
/// und sieht weder Shadow Roots noch `role="heading" aria-level="1"`.
fn top_headings(tree: &AXTree) -> Vec<String> {
    tree.iter()
        .filter(|n| n.heading_level() == Some(1))
        .filter_map(|n| n.name.as_ref().map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Die drei Ankündigungssignale einer clientseitigen Navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Signals {
    title_changed: bool,
    heading_changed: bool,
    focus_moved: bool,
}

fn signals(before: &AXSnapshot, after: &AXSnapshot, diff: &AXTreeDiff) -> Signals {
    Signals {
        // Ein Titelwechsel zählt nur, wenn danach etwas dasteht.
        title_changed: diff.title_changed.is_some() && !after.document_title.trim().is_empty(),
        heading_changed: {
            let b = top_headings(&before.tree);
            let a = top_headings(&after.tree);
            !a.is_empty() && a != b
        },
        // Ohne aufgenommenen Fokus danach ist keine Bewegung belegt.
        focus_moved: after.focus.active_backend_node_id.is_some()
            && after.focus.active_backend_node_id != before.focus.active_backend_node_id,
    }
}

async fn snapshot(page: &Page, label: &str) -> Option<AXSnapshot> {
    match AXSnapshot::capture(page, label, 0).await {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            tracing::warn!("spa_navigation: Aufnahme '{label}' fehlgeschlagen: {e}");
            None
        }
    }
}

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

async fn eval_int(page: &Page, js: &str) -> Option<i64> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    let result = page.execute(params).await.ok()?;
    result.result.result.value?.as_i64()
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

    // Die Ausgangsaufnahme wird vor *jedem* Versuch neu genommen: ein Klick,
    // der nicht navigiert, verschiebt trotzdem den Fokus auf den Link. Eine
    // einmal vor der Schleife genommene Aufnahme würde die Fokusbewegung des
    // vorigen Fehlversuchs der Navigation zuschreiben.
    let mut nav_detected = false;
    let mut before: Option<AXSnapshot> = None;

    for (selector, href) in &candidates {
        let Some(baseline) = snapshot(page, "before_spa_nav").await else {
            return Ok(None);
        };
        trace.steps.push(JourneyStep {
            action: "baseline".to_string(),
            target: Some(selector.clone()),
            focus: baseline.focus.selector.clone(),
            result: Some(format!(
                "title:{:?}, h1:{:?}",
                baseline.document_title,
                top_headings(&baseline.tree)
                    .first()
                    .cloned()
                    .unwrap_or_default()
            )),
            snapshot_label: Some("before_spa_nav".to_string()),
        });
        before = Some(baseline);

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

        let _ = stability::settle_after_action(page).await;

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
    let _ = stability::settle_after_action(page).await;

    let (Some(before), Some(after)) = (before, snapshot(page, "after_spa_nav").await) else {
        // Ohne beide Aufnahmen ist nichts belegt. Kein Befund.
        return Ok(Some((trace, Vec::new())));
    };
    let diff = AXTreeDiff::between(&before, &after);
    let Signals {
        title_changed,
        heading_changed,
        focus_moved,
    } = signals(&before, &after, &diff);
    let title_before = before.document_title.clone();

    trace.steps.push(JourneyStep {
        action: "check_spa_announcement".to_string(),
        target: None,
        focus: after.focus.selector.clone(),
        result: Some(format!(
            "title_changed:{title_changed}, heading_changed:{heading_changed}, \
             focus_moved:{focus_moved}, url_changed:{}",
            diff.url_changed.is_some()
        )),
        snapshot_label: Some("after_spa_nav".to_string()),
    });

    let mut findings = Vec::new();

    // Violation: none of the three signals are present.
    if !title_changed && !heading_changed && !focus_moved {
        findings.push(InteractiveFinding::new(
            "SpaNavigation",
            InteractiveFindingKind::SpaNoAnnouncementSignal,
            None,
            Severity::High,
            journey_name.clone(),
            Some("before_spa_nav".to_string()),
            Some("after_spa_nav".to_string()),
            InteractiveFindingValues {
                title_before: Some(title_before.clone()),
                ..Default::default()
            },
        ));
    } else if !title_changed {
        // Partial: heading or focus changed but title didn't.
        findings.push(InteractiveFinding::new(
            "SpaNavigation",
            InteractiveFindingKind::SpaTitleUnchanged,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_spa_nav".to_string()),
            Some("after_spa_nav".to_string()),
            InteractiveFindingValues {
                title_before: Some(title_before.clone()),
                ..Default::default()
            },
        ));
    } else if !focus_moved {
        // Title and/or heading changed, but focus stayed — weaker warning.
        findings.push(InteractiveFinding::new(
            "SpaNavigation",
            InteractiveFindingKind::SpaFocusNotMoved,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("before_spa_nav".to_string()),
            Some("after_spa_nav".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok(Some((trace, findings)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue, FocusSnapshot};

    fn heading(ax_id: &str, backend: i64, level: i64, name: &str) -> AXNode {
        AXNode {
            node_id: ax_id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some("heading".to_string()),
            name: Some(name.to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "level".to_string(),
                value: AXValue::Int(level),
            }],
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }
    }

    fn snap(title: &str, nodes: Vec<AXNode>, focus: Option<i64>) -> AXSnapshot {
        AXSnapshot::new(
            "s",
            "https://x/a",
            title,
            0,
            AXTree::from_nodes(nodes),
            FocusSnapshot {
                active_backend_node_id: focus,
                ..Default::default()
            },
        )
    }

    /// Nur Ebene 1 zählt, und nur mit Text.
    #[test]
    fn ueberschriften_erster_ebene_werden_gelesen() {
        let tree = AXTree::from_nodes(vec![
            heading("1", 1, 1, "Startseite"),
            heading("2", 2, 2, "Abschnitt"),
            heading("3", 3, 1, "   "),
        ]);
        assert_eq!(top_headings(&tree), vec!["Startseite".to_string()]);
    }

    #[test]
    fn alle_drei_signale_fehlen() {
        let before = snap("Titel", vec![heading("1", 1, 1, "A")], Some(9));
        let after = snap("Titel", vec![heading("1", 1, 1, "A")], Some(9));
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(
            signals(&before, &after, &diff),
            Signals {
                title_changed: false,
                heading_changed: false,
                focus_moved: false
            }
        );
    }

    #[test]
    fn titel_ueberschrift_und_fokus_werden_einzeln_erkannt() {
        let before = snap("Alt", vec![heading("1", 1, 1, "A")], Some(9));
        let after = snap("Neu", vec![heading("1", 1, 1, "B")], Some(10));
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(
            signals(&before, &after, &diff),
            Signals {
                title_changed: true,
                heading_changed: true,
                focus_moved: true
            }
        );
    }

    /// Ein leerer Titel nach der Navigation ist keine Ankündigung.
    #[test]
    fn leerer_titel_zaehlt_nicht_als_wechsel() {
        let before = snap("Alt", vec![], None);
        let after = snap("   ", vec![], None);
        let diff = AXTreeDiff::between(&before, &after);
        assert!(!signals(&before, &after, &diff).title_changed);
    }

    /// Ohne aufgenommenen Fokus danach ist keine Bewegung belegt — auch dann
    /// nicht, wenn vorher einer dastand.
    #[test]
    fn fehlender_fokus_danach_ist_keine_bewegung() {
        let before = snap("T", vec![], Some(5));
        let after = snap("T", vec![], None);
        let diff = AXTreeDiff::between(&before, &after);
        assert!(!signals(&before, &after, &diff).focus_moved);
    }
}
