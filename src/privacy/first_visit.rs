//! First-visit capture (#870): load a page in a clean browser profile, touch
//! nothing, and record what the browser sent, stored and kept.
//!
//! * **Clean profile.** A browser context of its own (no cookies, storage or
//!   cache from any other page), no stored consent, no Do-Not-Track / Global
//!   Privacy Control signal — the page sees an ordinary first-time visitor.
//! * **No interaction.** No banner is dismissed, nothing is clicked or
//!   scrolled. After the load the capture waits for the network to go quiet
//!   and then for a fixed settle time, so tags that fire late are included.
//! * **Raw data.** Every request (URL, host, resource type, initiator), every
//!   cookie of the context (attributes, never values) and every
//!   `localStorage` / `sessionStorage` key (never values). Canonical English,
//!   language-independent; no classification (casoon/barrierlab#91) and no
//!   verdict (#871).
//!
//! The result is a [`Collected`] under the collection contract (#777): a
//! capture that lost a part is [`Collected::Partial`] with the part named, a
//! capture that failed has no value at all. "Nothing was set" and "we could
//! not look" never read the same.
//!
//! `third_party` follows the convention of `performance::third_party` and the
//! cookie inventory: the host differs from the host of the loaded page.
//! Grouping by registrable domain is part of the classification.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::dom_storage::{
    EnableParams as DomStorageEnableParams, GetDomStorageItemsParams, Item, StorageId,
};
use chromiumoxide::cdp::browser_protocol::network::{
    Cookie, EnableParams as NetworkEnableParams, EventLoadingFailed, EventLoadingFinished,
    EventRequestWillBeSent,
};
use chromiumoxide::cdp::browser_protocol::page::{FrameTree, GetFrameTreeParams};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use url::Url;

use crate::audit::collection::{
    reason, Collected, CollectionMethod, CollectorSpec, COLLECTION_CONTRACT_VERSION,
};
use crate::browser::BrowserManager;

/// The first-visit collector. Its record belongs to the `privacy` module run
/// (created by the module of #871).
pub static FIRST_VISIT: CollectorSpec = CollectorSpec {
    id: "privacy.first_visit",
    module: "privacy",
    method: CollectionMethod::Cdp,
    method_version: 2,
    scoring: false,
};

/// Reason codes of the first-visit collector.
mod why {
    pub(super) const NAVIGATION_FAILED: &str = "navigation_failed";
    pub(super) const NO_REQUESTS: &str = "no_requests_observed";
    pub(super) const FINAL_URL_UNAVAILABLE: &str = "final_url_unavailable";
}

/// Gap names of a partial capture.
mod gap {
    pub(super) const COOKIES: &str = "cookies";
    pub(super) const STORAGE: &str = "storage";
    pub(super) const NETWORK_NOT_IDLE: &str = "network_not_idle";
    pub(super) const PAGE_SIGNALS: &str = "page_signals";
}

/// Waiting times of the capture.
#[derive(Debug, Clone, Copy)]
pub struct FirstVisitOptions {
    /// The network counts as idle after this long without a request in
    /// flight.
    pub idle_quiet: Duration,
    /// Longest wait for the network to go idle after the load.
    pub idle_max: Duration,
    /// Fixed time after the network went idle, for late tags.
    pub settle: Duration,
}

impl Default for FirstVisitOptions {
    fn default() -> Self {
        Self {
            idle_quiet: Duration::from_millis(500),
            idle_max: Duration::from_secs(10),
            settle: Duration::from_secs(2),
        }
    }
}

/// How the page was visited; part of the evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureProfile {
    /// A browser context created for this visit alone.
    pub fresh_context: bool,
    /// Nothing was clicked, dismissed or scrolled.
    pub interaction: bool,
    /// Whether Do-Not-Track / Global Privacy Control were sent.
    pub privacy_signals_sent: bool,
    pub idle_quiet_ms: u64,
    pub settle_ms: u64,
}

/// One request the browser made, redirect hops included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub url: String,
    pub host: String,
    /// Lower-case CDP resource type: `document`, `script`, `image`, `xhr`, …
    pub resource_type: String,
    /// CDP initiator type: `parser`, `script`, `preload`, `other`, …
    pub initiator: String,
    /// URL of the script or document that caused the request, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiator_url: Option<String>,
    /// The host differs from the host of the loaded page.
    pub third_party: bool,
}

/// One cookie in the context after the visit. The value is never kept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedCookie {
    pub name: String,
    pub domain: String,
    pub path: String,
    /// Expiry as Unix seconds; `None` for a session cookie. An absolute
    /// time: a cookie set with `max-age` expires at the clock of the visit
    /// plus the lifetime, so two captures differ here by the time between
    /// them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<f64>,
    pub secure: bool,
    pub http_only: bool,
    /// `Strict`, `Lax` or `None`; absent when the cookie states none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
    /// The cookie domain does not belong to the host of the loaded page.
    pub third_party: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageArea {
    Local,
    Session,
}

/// One web-storage key. The value is never kept.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapturedStorageKey {
    /// Origin of the frame whose storage holds the key.
    pub origin: String,
    pub area: StorageArea,
    pub key: String,
}

/// One anchor of the first-visit document, as the page wrote it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedLink {
    /// The `href` attribute, verbatim (may be empty, `#`, `javascript:` ...).
    pub href: String,
    /// The resolved absolute URL; empty when the browser could not resolve it.
    pub resolved: String,
    /// Visible text or `aria-label`, trimmed and shortened.
    pub text: String,
}

/// What the document itself shows about consent and legal pages, read after
/// the settle time without touching anything.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSignals {
    /// Canonical ids of consent-platform traces in the document: a known
    /// global (`global:Cookiebot`) or a known banner element
    /// (`element:onetrust-banner-sdk`). Sorted, once each.
    pub consent_signals: Vec<String>,
    /// Anchors of the document (capped), in document order.
    pub links: Vec<CapturedLink>,
}

/// What a first visit left behind. Lists are in canonical order (not event
/// order), so repeated captures of the same page compare equal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirstVisitCapture {
    /// [`COLLECTION_CONTRACT_VERSION`] the capture was made under.
    pub contract_version: u32,
    pub requested_url: String,
    pub final_url: String,
    pub page_host: String,
    pub profile: CaptureProfile,
    /// The network was idle before and after the settle time.
    pub network_idle: bool,
    pub requests: Vec<CapturedRequest>,
    pub cookies: Vec<CapturedCookie>,
    pub storage: Vec<CapturedStorageKey>,
    pub signals: PageSignals,
}

// ── Pure parsing ──────────────────────────────────────────────────────────

fn host_of(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
}

/// Host of the loaded page: the final URL, or the requested one when the
/// final URL carries none (never an empty host).
fn page_host(final_url: &str, requested_url: &str) -> Option<String> {
    host_of(final_url).or_else(|| host_of(requested_url))
}

/// The request of one CDP event; `None` for schemes that leave the browser
/// no network trace (`data:`, `blob:`, `about:`, extensions).
pub(crate) fn captured_request(
    event: &EventRequestWillBeSent,
    page_host: &str,
) -> Option<CapturedRequest> {
    let url = &event.request.url;
    let parsed = Url::parse(url).ok()?;
    if !matches!(parsed.scheme(), "http" | "https" | "ws" | "wss") {
        return None;
    }
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(CapturedRequest {
        third_party: host != page_host,
        host,
        url: url.clone(),
        resource_type: event
            .r#type
            .as_ref()
            .map(|t| t.as_ref().to_ascii_lowercase())
            .unwrap_or_else(|| "other".to_string()),
        initiator: event.initiator.r#type.as_ref().to_string(),
        initiator_url: event.initiator.url.clone(),
    })
}

/// A cookie without its value.
pub(crate) fn captured_cookie(cookie: &Cookie, page_host: &str) -> CapturedCookie {
    let domain = cookie.domain.trim_start_matches('.').to_ascii_lowercase();
    CapturedCookie {
        third_party: domain != page_host && !page_host.ends_with(&format!(".{domain}")),
        name: cookie.name.clone(),
        domain: cookie.domain.clone(),
        path: cookie.path.clone(),
        expires: (!cookie.session).then_some(cookie.expires),
        secure: cookie.secure,
        http_only: cookie.http_only,
        same_site: cookie
            .same_site
            .as_ref()
            .map(|same_site| same_site.as_ref().to_string()),
    }
}

/// The keys of one storage area; each CDP item is `[key, value]`, the value
/// is dropped here.
pub(crate) fn storage_keys(
    origin: &str,
    area: StorageArea,
    items: &[Item],
) -> Vec<CapturedStorageKey> {
    items
        .iter()
        .filter_map(|item| item.inner().first())
        .map(|key| CapturedStorageKey {
            origin: origin.to_string(),
            area,
            key: key.clone(),
        })
        .collect()
}

/// What the browser handed over, before it becomes a capture.
pub(crate) struct RawVisit {
    pub requested_url: String,
    pub final_url: Option<String>,
    pub events: Vec<Arc<EventRequestWillBeSent>>,
    /// `None`: the cookie jar could not be read.
    pub cookies: Option<Vec<Cookie>>,
    /// `None`: no storage area could be read at all.
    pub storage: Option<Vec<CapturedStorageKey>>,
    /// Some area of a known frame origin could not be read.
    pub storage_incomplete: bool,
    /// `None`: the document could not be probed.
    pub signals: Option<PageSignals>,
    pub network_idle: bool,
    pub options: FirstVisitOptions,
}

/// Turn the raw visit into the contract result. A part that could not be
/// read is a named gap — never an empty list.
pub(crate) fn assemble(raw: RawVisit) -> Collected<FirstVisitCapture> {
    let Some(final_url) = raw.final_url else {
        return Collected::Failed(why::FINAL_URL_UNAVAILABLE);
    };
    let Some(page_host) = page_host(&final_url, &raw.requested_url) else {
        return Collected::Failed(why::FINAL_URL_UNAVAILABLE);
    };

    let mut requests: Vec<CapturedRequest> = raw
        .events
        .iter()
        .filter_map(|event| captured_request(event, &page_host))
        .collect();
    // The page itself is always requested; no request at all means the
    // listener lost the events, not that the page sent nothing.
    if requests.is_empty() {
        return Collected::Failed(why::NO_REQUESTS);
    }
    requests.sort_by(|a, b| {
        (&a.url, &a.resource_type, &a.initiator, &a.initiator_url).cmp(&(
            &b.url,
            &b.resource_type,
            &b.initiator,
            &b.initiator_url,
        ))
    });

    let mut gaps = Vec::new();
    let mut cookies: Vec<CapturedCookie> = match &raw.cookies {
        Some(cookies) => cookies
            .iter()
            .map(|cookie| captured_cookie(cookie, &page_host))
            .collect(),
        None => {
            gaps.push(gap::COOKIES);
            Vec::new()
        }
    };
    cookies.sort_by(|a, b| (&a.domain, &a.path, &a.name).cmp(&(&b.domain, &b.path, &b.name)));

    let mut storage = match raw.storage {
        Some(keys) => {
            if raw.storage_incomplete {
                gaps.push(gap::STORAGE);
            }
            keys
        }
        None => {
            gaps.push(gap::STORAGE);
            Vec::new()
        }
    };
    storage.sort();
    storage.dedup();

    let signals = match raw.signals {
        Some(mut signals) => {
            signals.consent_signals.sort();
            signals.consent_signals.dedup();
            signals
        }
        None => {
            gaps.push(gap::PAGE_SIGNALS);
            PageSignals::default()
        }
    };

    if !raw.network_idle {
        gaps.push(gap::NETWORK_NOT_IDLE);
    }

    let capture = FirstVisitCapture {
        contract_version: COLLECTION_CONTRACT_VERSION,
        requested_url: raw.requested_url,
        final_url,
        page_host,
        profile: CaptureProfile {
            fresh_context: true,
            interaction: false,
            privacy_signals_sent: false,
            idle_quiet_ms: raw.options.idle_quiet.as_millis() as u64,
            settle_ms: raw.options.settle.as_millis() as u64,
        },
        network_idle: raw.network_idle,
        requests,
        cookies,
        storage,
        signals,
    };
    if gaps.is_empty() {
        Collected::Measured(capture)
    } else {
        Collected::Partial {
            value: capture,
            gaps,
        }
    }
}

/// Origins whose web storage a visit can have filled: the origin of every
/// http(s) frame of the frame tree, once each.
pub(crate) fn frame_origins(tree: &FrameTree) -> Vec<String> {
    fn walk(tree: &FrameTree, out: &mut Vec<String>) {
        if let Ok(url) = Url::parse(&tree.frame.url) {
            if matches!(url.scheme(), "http" | "https") {
                let origin = url.origin().ascii_serialization();
                if !out.contains(&origin) {
                    out.push(origin);
                }
            }
        }
        for child in tree.child_frames.iter().flatten() {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(tree, &mut out);
    out
}

// ── Collection ────────────────────────────────────────────────────────────

/// Requests seen so far and the ones still in flight.
#[derive(Default)]
struct Traffic {
    events: Vec<Arc<EventRequestWillBeSent>>,
    /// Request id → URL.
    in_flight: HashMap<String, String>,
    last_activity: Option<Instant>,
}

impl Traffic {
    fn touch(&mut self) {
        self.last_activity = Some(Instant::now());
    }
}

fn lock(traffic: &Mutex<Traffic>) -> std::sync::MutexGuard<'_, Traffic> {
    traffic
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Wait until nothing has been in flight for `quiet`, at most `max`.
async fn wait_for_idle(traffic: &Mutex<Traffic>, quiet: Duration, max: Duration) -> bool {
    let deadline = Instant::now() + max;
    loop {
        let idle = {
            let t = lock(traffic);
            t.in_flight.is_empty()
                && t.last_activity
                    .map(|at| at.elapsed() >= quiet)
                    .unwrap_or(true)
        };
        if idle {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Feed the three CDP streams into `traffic` until all of them end.
///
/// `biased`: `select!` otherwise picks a ready branch at random. On a loaded
/// runner several events are queued at once, and a `loadingFinished` handled
/// before the `requestWillBeSent` of the same request would remove nothing
/// and leave that request in flight for good, so the network never reads as
/// idle. Chrome sends the start of a request before its end, so draining the
/// start stream first keeps the order.
async fn pump_traffic(
    mut sent: impl futures::Stream<Item = Arc<EventRequestWillBeSent>> + Unpin,
    mut finished: impl futures::Stream<Item = Arc<EventLoadingFinished>> + Unpin,
    mut failed: impl futures::Stream<Item = Arc<EventLoadingFailed>> + Unpin,
    traffic: Arc<Mutex<Traffic>>,
) {
    loop {
        tokio::select! {
            biased;
            Some(event) = sent.next() => {
                let mut t = lock(&traffic);
                t.in_flight.insert(event.request_id.inner().clone(), event.request.url.clone());
                t.touch();
                t.events.push(event);
            }
            Some(event) = finished.next() => {
                let mut t = lock(&traffic);
                t.in_flight.remove(event.request_id.inner());
                t.touch();
            }
            Some(event) = failed.next() => {
                let mut t = lock(&traffic);
                t.in_flight.remove(event.request_id.inner());
                t.touch();
            }
            else => break,
        }
    }
}

/// Visit `url` in a clean browser context and capture what the first visit
/// leaves behind. The context is disposed again, whatever the outcome.
pub async fn collect_first_visit(
    manager: &BrowserManager,
    url: &str,
    options: FirstVisitOptions,
) -> Collected<FirstVisitCapture> {
    let (page, context) = match manager.new_first_visit_page().await {
        Ok(pair) => pair,
        Err(e) => {
            warn!("first-visit page could not be created: {e}");
            return Collected::Unsupported("clean_context_unavailable");
        }
    };
    let collected = visit(manager, &page, &context, url, options).await;
    let _ = page.close().await;
    manager.dispose_context(context).await;
    collected
}

async fn visit(
    manager: &BrowserManager,
    page: &chromiumoxide::Page,
    context: &chromiumoxide::cdp::browser_protocol::browser::BrowserContextId,
    url: &str,
    options: FirstVisitOptions,
) -> Collected<FirstVisitCapture> {
    let traffic = Arc::new(Mutex::new(Traffic::default()));

    // The listeners exist before Network is enabled and before the
    // navigation: the first request is the document itself.
    let (Ok(sent), Ok(finished), Ok(failed)) = (
        page.event_listener::<EventRequestWillBeSent>().await,
        page.event_listener::<EventLoadingFinished>().await,
        page.event_listener::<EventLoadingFailed>().await,
    ) else {
        return Collected::Failed(reason::CDP_COMMAND_FAILED);
    };
    if page.execute(NetworkEnableParams::default()).await.is_err() {
        return Collected::Failed(reason::CDP_COMMAND_FAILED);
    }
    let pump = tokio::spawn(pump_traffic(sent, finished, failed, Arc::clone(&traffic)));

    let collected = async {
        if let Err(e) = manager.navigate(page, url).await {
            warn!("first-visit navigation to {url} failed: {e}");
            return Collected::Failed(why::NAVIGATION_FAILED);
        }
        let settled_before = wait_for_idle(&traffic, options.idle_quiet, options.idle_max).await;
        tokio::time::sleep(options.settle).await;
        let idle_after = lock(&traffic).in_flight.is_empty();
        debug!("first visit of {url}: idle before settle {settled_before}, after {idle_after}");
        if !(settled_before && idle_after) {
            let t = lock(&traffic);
            warn!(
                "first visit of {url}: network not idle, still in flight: {:?}",
                t.in_flight.values().collect::<Vec<_>>()
            );
        }

        let final_url = page.url().await.ok().flatten();
        let cookies = read_cookies(manager, context).await;
        let (storage, storage_incomplete) = read_storage(page).await;
        let signals = read_signals(page).await;
        let events = lock(&traffic).events.clone();
        assemble(RawVisit {
            requested_url: url.to_string(),
            final_url,
            events,
            cookies,
            storage,
            storage_incomplete,
            signals,
            network_idle: settled_before && idle_after,
            options,
        })
    }
    .await;

    pump.abort();
    collected
}

/// Every cookie of the visit's context — including those set by the
/// responses of other hosts, which `Network.getCookies` of the page URL
/// would not list.
async fn read_cookies(
    manager: &BrowserManager,
    context: &chromiumoxide::cdp::browser_protocol::browser::BrowserContextId,
) -> Option<Vec<Cookie>> {
    match manager.context_cookies(context).await {
        Ok(cookies) => Some(cookies),
        Err(e) => {
            warn!("first-visit cookies could not be read: {e}");
            None
        }
    }
}

/// Read-only probe of the document: consent-platform globals and banner
/// elements, and the anchors (the classification into privacy / imprint
/// links is done on the Rust side). Returns a JSON string.
const SIGNALS_SCRIPT: &str = r#"(() => {
  const globals = ['__tcfapi', '__cmp', '__gpp', 'Cookiebot', 'OneTrust', 'Optanon', 'UC_UI',
    'Didomi', 'klaro', 'BorlabsCookie', 'cookieconsent', 'CookieConsent', 'cmplz_cookiebanner',
    'Cookiefirst', 'CookieScript', 'axeptio', '_axcb'];
  const elements = {
    'cookiebot': '#CybotCookiebotDialog',
    'onetrust-banner-sdk': '#onetrust-banner-sdk',
    'usercentrics': '#usercentrics-root, #usercentrics-cmp-ui',
    'didomi': '#didomi-host',
    'borlabs': '#BorlabsCookieBox, [data-borlabs-cookie-wrap]',
    'klaro': '.klaro',
    'complianz': '.cmplz-cookiebanner, #cmplz-cookiebanner-container',
    'cookie-law-info': '#cookie-law-info-bar',
    'cookie-notice': '#cookie-notice',
    'cookieconsent': '.cc-window, .cc-banner',
    'generic-banner': '#cookiebanner, #cookie-banner, .cookie-banner, #cookie-consent, .cookie-consent, [class*="cookie-consent" i], [class*="consent-banner" i], [id*="consent-banner" i]',
    'consent-dialog': '[role="dialog"][aria-label*="cookie" i], [role="dialog"][aria-label*="consent" i], [role="dialog"][aria-label*="datenschutz" i], dialog[id*="cookie" i], dialog[id*="consent" i]'
  };
  const out = { consent_signals: [], links: [] };
  for (const g of globals) { try { if (typeof window[g] !== 'undefined') out.consent_signals.push('global:' + g); } catch (e) {} }
  for (const [id, sel] of Object.entries(elements)) {
    try { if (document.querySelector(sel)) out.consent_signals.push('element:' + id); } catch (e) {}
  }
  for (const a of Array.from(document.querySelectorAll('a[href]')).slice(0, 2000)) {
    const text = ((a.textContent || '').replace(/\s+/g, ' ').trim() || a.getAttribute('aria-label') || a.getAttribute('title') || '').slice(0, 120);
    out.links.push({ href: a.getAttribute('href') || '', resolved: a.href || '', text });
  }
  return JSON.stringify(out);
})()"#;

async fn read_signals(page: &chromiumoxide::Page) -> Option<PageSignals> {
    match crate::audit::collection::evaluate_js::<PageSignals>(page, SIGNALS_SCRIPT).await {
        Collected::Measured(signals) => Some(signals),
        other => {
            warn!("first-visit page signals not readable: {:?}", other.state());
            None
        }
    }
}

/// Keys of both storage areas of every frame origin. `None` when not a
/// single area could be read; the flag tells that some could not.
async fn read_storage(page: &chromiumoxide::Page) -> (Option<Vec<CapturedStorageKey>>, bool) {
    let Ok(tree) = page.execute(GetFrameTreeParams::default()).await else {
        return (None, true);
    };
    if page
        .execute(DomStorageEnableParams::default())
        .await
        .is_err()
    {
        return (None, true);
    }
    let mut keys = Vec::new();
    let (mut read, mut unread) = (0usize, 0usize);
    for origin in frame_origins(&tree.result.frame_tree) {
        for (area, is_local) in [(StorageArea::Local, true), (StorageArea::Session, false)] {
            let mut id = StorageId::new(is_local);
            id.security_origin = Some(origin.clone());
            match page.execute(GetDomStorageItemsParams::new(id)).await {
                Ok(items) => {
                    read += 1;
                    keys.extend(storage_keys(&origin, area, &items.result.entries));
                }
                Err(e) => {
                    unread += 1;
                    debug!("storage {area:?} of {origin} not readable: {e}");
                }
            }
        }
    }
    if read == 0 {
        (None, true)
    } else {
        (Some(keys), unread > 0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    fn event(url: &str, ty: &str, initiator: serde_json::Value) -> Arc<EventRequestWillBeSent> {
        Arc::new(
            serde_json::from_value(json!({
                "requestId": "1",
                "loaderId": "L",
                "documentURL": "https://example.com/",
                "request": {
                    "url": url, "method": "GET", "headers": {},
                    "initialPriority": "High", "referrerPolicy": "no-referrer"
                },
                "timestamp": 1.0,
                "wallTime": 1.0,
                "initiator": initiator,
                "redirectHasExtraInfo": false,
                "type": ty
            }))
            .expect("valid CDP event"),
        )
    }

    fn started(id: &str) -> Arc<EventRequestWillBeSent> {
        let mut e = (*event("https://example.com/a", "Image", json!({"type": "parser"}))).clone();
        e.request_id = id.to_string().into();
        Arc::new(e)
    }

    /// A finish that is ready together with its start must not be handled
    /// first (random `select!` order left the request in flight forever).
    #[tokio::test]
    async fn a_request_that_finished_is_not_left_in_flight_whatever_the_poll_order() {
        for round in 0..200 {
            let traffic = Arc::new(Mutex::new(Traffic::default()));
            let done: EventLoadingFinished = serde_json::from_value(
                json!({"requestId": "r1", "timestamp": 1.0, "encodedDataLength": 0.0}),
            )
            .unwrap();
            pump_traffic(
                futures::stream::iter(vec![started("r1")]),
                futures::stream::iter(vec![Arc::new(done)]),
                futures::stream::empty(),
                Arc::clone(&traffic),
            )
            .await;
            let t = lock(&traffic);
            assert!(t.in_flight.is_empty(), "round {round}: {:?}", t.in_flight);
            assert_eq!(t.events.len(), 1);
        }
    }

    fn cookie(name: &str, domain: &str, session: bool) -> Cookie {
        serde_json::from_value(json!({
            "name": name, "value": "secret-value", "domain": domain, "path": "/",
            "expires": if session { -1.0 } else { 1900000000.0 },
            "size": 10, "httpOnly": true, "secure": true, "session": session,
            "sameSite": "Lax", "priority": "Medium",
            "sourceScheme": "Secure", "sourcePort": 443
        }))
        .expect("valid CDP cookie")
    }

    /// Counter-cases of `privacy.first_visit` (#777): lost events, a lost
    /// final URL, an unreadable cookie jar or storage and a network that
    /// never went quiet leave no regular value; valid empty cookies and
    /// storage are a measurement.
    pub(crate) fn counter_cases() {
        use crate::audit::collection::CollectionState;
        let state = |edit: fn(&mut RawVisit)| {
            let mut r = raw();
            edit(&mut r);
            assemble(r).state()
        };
        assert_eq!(state(|_| {}), CollectionState::Measured);
        assert_eq!(
            state(|r| {
                r.cookies = Some(vec![]);
                r.storage = Some(vec![]);
            }),
            CollectionState::Measured
        );
        assert_eq!(state(|r| r.events.clear()), CollectionState::Failed);
        assert_eq!(state(|r| r.final_url = None), CollectionState::Failed);
        assert_eq!(state(|r| r.cookies = None), CollectionState::Partial);
        assert_eq!(state(|r| r.storage = None), CollectionState::Partial);
        assert_eq!(state(|r| r.network_idle = false), CollectionState::Partial);
        assert_eq!(state(|r| r.signals = None), CollectionState::Partial);
    }

    fn raw() -> RawVisit {
        RawVisit {
            requested_url: "https://example.com/".into(),
            final_url: Some("https://example.com/".into()),
            events: vec![
                event("https://example.com/", "Document", json!({"type": "other"})),
                event(
                    "https://stats.example.net/t.js",
                    "Script",
                    json!({"type": "parser", "url": "https://example.com/"}),
                ),
            ],
            cookies: Some(vec![cookie("sid", "example.com", true)]),
            storage: Some(vec![]),
            storage_incomplete: false,
            signals: Some(PageSignals::default()),
            network_idle: true,
            options: FirstVisitOptions::default(),
        }
    }

    #[test]
    fn a_complete_visit_is_measured_with_raw_fields_and_no_values() {
        let Collected::Measured(c) = assemble(raw()) else {
            panic!("expected a measured capture");
        };
        assert_eq!(c.page_host, "example.com");
        assert_eq!(c.requests.len(), 2);
        let tracker = c.requests.iter().find(|r| r.host == "stats.example.net");
        let tracker = tracker.expect("third-party request");
        assert!(tracker.third_party);
        assert_eq!(tracker.resource_type, "script");
        assert_eq!(tracker.initiator, "parser");
        assert_eq!(
            tracker.initiator_url.as_deref(),
            Some("https://example.com/")
        );
        assert!(!c
            .requests
            .iter()
            .any(|r| r.host == "example.com" && r.third_party));
        assert_eq!(c.cookies.len(), 1);
        assert_eq!(c.cookies[0].expires, None, "session cookie has no expiry");
        assert!(!serde_json::to_string(&c).unwrap().contains("secret-value"));
        assert!(!c.profile.privacy_signals_sent && !c.profile.interaction);
    }

    #[test]
    fn requests_are_in_canonical_order_whatever_the_event_order() {
        let mut reversed = raw();
        reversed.events.reverse();
        let (Collected::Measured(a), Collected::Measured(b)) =
            (assemble(raw()), assemble(reversed))
        else {
            panic!("expected measured captures");
        };
        assert_eq!(a, b);
    }

    #[test]
    fn schemes_without_a_network_trace_are_not_requests() {
        let mut r = raw();
        r.events.push(event(
            "data:text/plain,x",
            "Image",
            json!({"type": "parser"}),
        ));
        r.events.push(event(
            "blob:https://example.com/1",
            "Image",
            json!({"type": "script"}),
        ));
        let Collected::Measured(c) = assemble(r) else {
            panic!("expected measured");
        };
        assert_eq!(c.requests.len(), 2);
    }

    #[test]
    fn no_request_at_all_is_a_failure_not_an_empty_page() {
        let mut r = raw();
        r.events.clear();
        assert_eq!(assemble(r), Collected::Failed(why::NO_REQUESTS));
    }

    #[test]
    fn an_unknown_final_url_is_a_failure() {
        let mut r = raw();
        r.final_url = None;
        assert_eq!(assemble(r), Collected::Failed(why::FINAL_URL_UNAVAILABLE));
    }

    #[test]
    fn an_unreadable_cookie_jar_is_a_gap_not_zero_cookies() {
        let mut r = raw();
        r.cookies = None;
        let Collected::Partial { value, gaps } = assemble(r) else {
            panic!("expected partial");
        };
        assert_eq!(gaps, vec![gap::COOKIES]);
        assert!(value.cookies.is_empty());
    }

    #[test]
    fn unreadable_storage_is_a_gap_not_zero_keys() {
        let mut none = raw();
        none.storage = None;
        let mut some = raw();
        some.storage_incomplete = true;
        for r in [none, some] {
            let Collected::Partial { gaps, .. } = assemble(r) else {
                panic!("expected partial");
            };
            assert_eq!(gaps, vec![gap::STORAGE]);
        }
    }

    #[test]
    fn a_network_that_never_went_idle_is_named() {
        let mut r = raw();
        r.network_idle = false;
        let Collected::Partial { gaps, .. } = assemble(r) else {
            panic!("expected partial");
        };
        assert_eq!(gaps, vec![gap::NETWORK_NOT_IDLE]);
    }

    #[test]
    fn valid_empty_cookies_and_storage_are_not_a_gap() {
        let mut r = raw();
        r.cookies = Some(vec![]);
        assert!(matches!(assemble(r), Collected::Measured(_)));
    }

    #[test]
    fn cookie_party_follows_the_page_host() {
        let own = captured_cookie(&cookie("a", ".example.com", false), "www.example.com");
        let other = captured_cookie(&cookie("b", "ads.example.net", false), "www.example.com");
        assert!(!own.third_party);
        assert!(other.third_party);
        assert_eq!(own.expires, Some(1900000000.0));
        assert_eq!(own.same_site.as_deref(), Some("Lax"));
    }

    #[test]
    fn storage_keeps_keys_and_drops_values() {
        let items = vec![
            Item::new(vec!["theme".to_string(), "dark".to_string()]),
            Item::new(vec!["_ga_id".to_string(), "GA1.2.3".to_string()]),
        ];
        let keys = storage_keys("https://example.com", StorageArea::Local, &items);
        assert_eq!(
            keys.iter().map(|k| k.key.as_str()).collect::<Vec<_>>(),
            vec!["theme", "_ga_id"]
        );
        assert!(!serde_json::to_string(&keys).unwrap().contains("GA1.2.3"));
    }

    #[test]
    fn frame_origins_are_http_origins_once_each() {
        let tree: FrameTree = serde_json::from_value(json!({
            "frame": {"id": "1", "loaderId": "L", "url": "https://example.com/a?x=1",
                      "domainAndRegistry": "", "securityOrigin": "https://example.com",
                      "mimeType": "text/html", "secureContextType": "Secure",
                      "crossOriginIsolatedContextType": "NotIsolated",
                      "gatedAPIFeatures": []},
            "childFrames": [
                {"frame": {"id": "2", "parentId": "1", "loaderId": "L", "url": "about:blank",
                           "domainAndRegistry": "", "securityOrigin": "://",
                           "mimeType": "text/html", "secureContextType": "Secure",
                           "crossOriginIsolatedContextType": "NotIsolated",
                           "gatedAPIFeatures": []}},
                {"frame": {"id": "3", "parentId": "1", "loaderId": "L",
                           "url": "https://example.com/b",
                           "domainAndRegistry": "", "securityOrigin": "https://example.com",
                           "mimeType": "text/html", "secureContextType": "Secure",
                           "crossOriginIsolatedContextType": "NotIsolated",
                           "gatedAPIFeatures": []}},
                {"frame": {"id": "4", "parentId": "1", "loaderId": "L",
                           "url": "https://embed.example.org:8443/w",
                           "domainAndRegistry": "", "securityOrigin": "https://embed.example.org:8443",
                           "mimeType": "text/html", "secureContextType": "Secure",
                           "crossOriginIsolatedContextType": "NotIsolated",
                           "gatedAPIFeatures": []}}
            ]
        }))
        .expect("valid frame tree");
        assert_eq!(
            frame_origins(&tree),
            vec!["https://example.com", "https://embed.example.org:8443"]
        );
    }
}
