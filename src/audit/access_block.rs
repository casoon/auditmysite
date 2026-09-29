//! Recognize pages that are not the site's content: bot walls, challenges,
//! access denials.
//!
//! Such a page used to be audited like any other. www.douglas.de answers
//! automated browsers with HTTP 403 and an Akamai "Access Denied" page of
//! about 100 nodes; it was scored 75 and reported as complete. With the
//! headless shell, www.hornbach.de served a Fastly "Client Challenge" that
//! ended in a CAPTCHA (355 nodes instead of 12,942) and scored 85 instead of 56.
//!
//! A blocked page fails the audit of its URL instead of producing a score.

use chromiumoxide::Page;
use serde::Deserialize;

/// Status codes that deny the document itself.
const BLOCKING_STATUS: &[u16] = &[401, 403, 407, 429];

/// Document titles used only by challenge pages of bot-protection vendors.
const CHALLENGE_TITLES: &[&str] = &[
    "Client Challenge",                    // Fastly
    "Just a moment...",                    // Cloudflare
    "Attention Required! | Cloudflare",    // Cloudflare
    "Access to this page has been denied", // HUMAN / PerimeterX
    "Pardon Our Interruption",             // Imperva / Distil
];

/// Resource paths of challenge pages. Some vendors load scripts from similar
/// paths on regular pages too (Cloudflare's JavaScript detections live under
/// `/cdn-cgi/challenge-platform/`), so a path only counts on a thin page.
const CHALLENGE_PATHS: &[&str] = &[
    "/_fs-ch-",                     // Fastly client challenge
    "/cdn-cgi/challenge-platform/", // Cloudflare
    "captcha-delivery.com",         // DataDome
    "/_Incapsula_Resource",         // Imperva
    "px-captcha",                   // HUMAN / PerimeterX
    "/_sec/cp_challenge/",          // Akamai
];

/// Fewer elements than this make a page thin. Challenge pages consist of a
/// form, a script and a few wrappers.
const THIN_PAGE_ELEMENTS: u32 = 300;

/// What the page shows, collected in one evaluation.
#[derive(Debug, Default, Deserialize)]
pub struct PageSignals {
    pub status: Option<u16>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub elements: u32,
    /// `src`/`href` of scripts, iframes, stylesheets and loaded resources.
    #[serde(default)]
    pub resources: Vec<String>,
}

/// Why the page is not the site's content, or `None` for a regular page.
pub fn classify(signals: &PageSignals) -> Option<String> {
    if let Some(reason) = blocking_status(signals.status) {
        return Some(reason);
    }
    let title = signals.title.trim();
    if CHALLENGE_TITLES.contains(&title) {
        return Some(format!("the page is a bot challenge (\"{title}\")"));
    }
    if signals.elements < THIN_PAGE_ELEMENTS {
        if let Some(path) = CHALLENGE_PATHS
            .iter()
            .find(|path| signals.resources.iter().any(|r| r.contains(*path)))
        {
            return Some(format!("the page is a bot challenge (loads {path})"));
        }
    }
    None
}

/// The status part of [`classify`], for callers that only know the status.
pub fn blocking_status(status: Option<u16>) -> Option<String> {
    status
        .filter(|s| BLOCKING_STATUS.contains(s))
        .map(|s| format!("the server answered HTTP {s}"))
}

const SIGNALS_JS: &str = r#"(() => {
    const nav = performance.getEntriesByType('navigation')[0];
    const resources = [];
    for (const el of document.querySelectorAll('script[src], iframe[src], link[href]')) {
        resources.push(el.getAttribute('src') || el.getAttribute('href') || '');
    }
    for (const entry of performance.getEntriesByType('resource')) resources.push(entry.name);
    return {
        status: nav && Number.isFinite(nav.responseStatus) && nav.responseStatus > 0 ? nav.responseStatus : null,
        title: document.title || '',
        elements: document.getElementsByTagName('*').length,
        resources: resources.slice(0, 500),
    };
})()"#;

/// Collect the signals from the loaded page and classify them.
///
/// Challenge pages often replace their document right after loading; an
/// evaluation then fails with "Cannot find context with specified id" (seen
/// on www.douglas.de). Up to three attempts; if all fail the page counts as
/// regular here, and the pipeline checks the document status again later.
pub async fn detect(page: &Page) -> Option<String> {
    for attempt in 0..3 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        let signals = page
            .evaluate(SIGNALS_JS)
            .await
            .ok()
            .and_then(|result| result.value().cloned())
            .and_then(|value| serde_json::from_value::<PageSignals>(value).ok());
        if let Some(signals) = signals {
            return classify(&signals);
        }
        tracing::debug!("Access-block signals unavailable (attempt {})", attempt + 1);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(status: u16, title: &str, elements: u32, resources: &[&str]) -> PageSignals {
        PageSignals {
            status: Some(status),
            title: title.to_string(),
            elements,
            resources: resources.iter().map(|r| r.to_string()).collect(),
        }
    }

    #[test]
    fn akamai_access_denied_is_blocked_by_status() {
        let douglas = page(403, "Access Denied", 12, &[]);
        assert_eq!(
            classify(&douglas).as_deref(),
            Some("the server answered HTTP 403")
        );
    }

    #[test]
    fn fastly_client_challenge_is_blocked_by_title_and_path() {
        let hornbach = page(
            200,
            "Client Challenge",
            40,
            &["/_fs-ch-1T1wmsGaOgGaSxcX/script.js?reload=true"],
        );
        assert!(classify(&hornbach).unwrap().contains("Client Challenge"));
        let untitled = page(200, "", 40, &["https://x.de/_fs-ch-1T1wms/errors.js"]);
        assert!(classify(&untitled).unwrap().contains("/_fs-ch-"));
    }

    #[test]
    fn challenge_scripts_on_a_full_page_do_not_block() {
        // Cloudflare injects its JavaScript detections into regular pages.
        let regular = page(
            200,
            "Startseite",
            2400,
            &["https://x.de/cdn-cgi/challenge-platform/scripts/jsd/main.js"],
        );
        assert_eq!(classify(&regular), None);
    }

    #[test]
    fn regular_and_not_found_pages_are_not_blocked() {
        assert_eq!(classify(&page(200, "Home", 20, &[])), None);
        // A missing page is its own problem, not an access block.
        assert_eq!(classify(&page(404, "Not found", 80, &[])), None);
        assert_eq!(classify(&PageSignals::default()), None);
    }
}
