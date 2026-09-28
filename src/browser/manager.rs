//! Browser Manager - Chrome lifecycle management
//!
//! Handles launching Chrome in headless mode with optimized flags,
//! managing CDP connections, and graceful shutdown.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig, HeadlessMode};
use chromiumoxide::Page;
use futures::StreamExt;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::detection::{verify_executable, ChromeInfo};
use super::resolver::{self, BrowserResolveOptions};
use crate::error::{AuditError, Result};

static BROWSER_PROFILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Browser configuration options
#[derive(Debug, Clone)]
pub struct BrowserOptions {
    /// Manual Chrome path override
    pub chrome_path: Option<String>,
    /// Run in headless mode (default: true)
    pub headless: bool,
    /// Disable GPU acceleration (default: true for headless)
    pub disable_gpu: bool,
    /// Disable sandbox (required for Docker/root)
    pub no_sandbox: bool,
    /// Disable images for faster loading
    pub disable_images: bool,
    /// Window size for consistent viewport
    pub window_size: (u32, u32),
    /// Page load timeout in seconds
    pub timeout_secs: u64,
    /// Enable verbose browser logging
    pub verbose: bool,
    /// Override the user-agent string (None = use default browser simulation UA)
    pub user_agent_override: Option<String>,
}

impl Default for BrowserOptions {
    fn default() -> Self {
        Self {
            chrome_path: None,
            headless: true,
            disable_gpu: true,
            no_sandbox: false,
            disable_images: false,
            window_size: (1920, 1080),
            timeout_secs: 30,
            verbose: false,
            user_agent_override: None,
        }
    }
}

/// Browser Manager - handles Chrome lifecycle
pub struct BrowserManager {
    browser: Browser,
    chrome_info: ChromeInfo,
    options: BrowserOptions,
    user_data_dir: PathBuf,
    _handler: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone, Copy)]
struct LaunchPlan {
    headless_mode: HeadlessMode,
    disable_gpu: bool,
    label: &'static str,
}

impl BrowserManager {
    /// Create a new BrowserManager with default options
    pub async fn new() -> Result<Self> {
        Self::with_options(BrowserOptions::default()).await
    }

    /// Create a new BrowserManager with custom options
    pub async fn with_options(options: BrowserOptions) -> Result<Self> {
        let user_data_dir = Self::create_user_data_dir()?;

        // Resolve browser first so we can use the real version in the UA string
        let resolve_opts = BrowserResolveOptions {
            browser_path: options.chrome_path.clone(),
            browser_preference: None,
            strict: false,
        };

        let resolved = resolver::resolve_browser(&resolve_opts)?;
        info!(
            "Using {}: {} v{}",
            resolved.browser.kind.display_name(),
            resolved.browser.path.display(),
            resolved.browser.version.as_deref().unwrap_or("unknown")
        );
        if resolver::stalls_on_keyboard_journeys(&resolved.browser) {
            static WARNED: std::sync::Once = std::sync::Once::new();
            WARNED.call_once(|| {
                warn!(
                    "{} on macOS can stall during keyboard journeys, most often in batch runs. \
                     Install the headless shell once: auditmysite browser install --headless-shell",
                    resolved.browser.kind.display_name()
                );
            });
        }

        verify_executable(&resolved.browser.path)?;

        let chrome_info = ChromeInfo::from(&resolved.browser);

        let args = Self::build_launch_args(
            &options,
            options.disable_gpu,
            chrome_info.version.as_deref(),
        );
        debug!("Chrome launch args: {:?}", args);

        let mut launch_errors = Vec::new();
        let mut launched = None;

        for plan in Self::launch_plans(&options) {
            let config = Self::build_browser_config(
                &resolved.browser.path,
                &user_data_dir,
                &options,
                plan,
                chrome_info.version.as_deref(),
            )?;

            info!(
                "Launching browser with strategy '{}' (headless={:?}, disable_gpu={})",
                plan.label, plan.headless_mode, plan.disable_gpu
            );

            match Browser::launch(config).await {
                Ok((browser, handler)) => {
                    launched = Some((browser, handler, plan));
                    break;
                }
                Err(e) => {
                    warn!("Browser launch strategy '{}' failed: {}", plan.label, e);
                    launch_errors.push(format!("{}: {}", plan.label, e));
                }
            }
        }

        let (browser, mut handler, plan) =
            launched.ok_or_else(|| AuditError::BrowserLaunchFailed {
                reason: launch_errors.join(" | "),
            })?;

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                debug!("Browser event: {:?}", event);
            }
        });

        info!(
            "Browser launched successfully with strategy '{}'",
            plan.label
        );

        Ok(Self {
            browser,
            chrome_info,
            options,
            user_data_dir,
            _handler: Arc::new(Mutex::new(Some(handler_task))),
        })
    }

    /// Build Chrome launch arguments based on options
    fn launch_plans(options: &BrowserOptions) -> Vec<LaunchPlan> {
        if !options.headless {
            return vec![LaunchPlan {
                headless_mode: HeadlessMode::False,
                disable_gpu: options.disable_gpu,
                label: "headful",
            }];
        }

        let mut plans = vec![
            LaunchPlan {
                headless_mode: HeadlessMode::New,
                disable_gpu: options.disable_gpu,
                label: "headless-new",
            },
            LaunchPlan {
                headless_mode: HeadlessMode::True,
                disable_gpu: options.disable_gpu,
                label: "headless-legacy",
            },
        ];

        if options.disable_gpu {
            plans.push(LaunchPlan {
                headless_mode: HeadlessMode::True,
                disable_gpu: false,
                label: "headless-legacy-gpu-enabled",
            });
        }

        plans
    }

    fn build_browser_config(
        browser_path: &std::path::Path,
        user_data_dir: &std::path::Path,
        options: &BrowserOptions,
        plan: LaunchPlan,
        chrome_version: Option<&str>,
    ) -> Result<BrowserConfig> {
        let mut builder = BrowserConfig::builder()
            .chrome_executable(browser_path)
            .user_data_dir(user_data_dir)
            .window_size(options.window_size.0, options.window_size.1)
            .headless_mode(plan.headless_mode)
            .args(Self::build_launch_args(
                options,
                plan.disable_gpu,
                chrome_version,
            ))
            .viewport(None);

        if options.no_sandbox {
            builder = builder.no_sandbox();
        }

        builder
            .build()
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: e.to_string(),
            })
    }

    fn build_launch_args(
        options: &BrowserOptions,
        disable_gpu: bool,
        chrome_version: Option<&str>,
    ) -> Vec<String> {
        let version = chrome_version.unwrap_or("131.0.0.0");
        let user_agent = options.user_agent_override.clone().unwrap_or_else(|| {
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                version
            )
        });

        let mut args = vec![
            // Note: headless mode, user data dir, window size, and sandbox
            // are configured through BrowserConfig and should not be duplicated here.
            "--no-default-browser-check".to_string(),
            "--disable-translate".to_string(),
            "--disable-infobars".to_string(),
            // Suppress first-run dialogs and visible windows (important on Windows)
            "--no-first-run".to_string(),
            "--disable-default-apps".to_string(),
            "--hide-crash-restore-bubble".to_string(),
            "--disable-session-crashed-bubble".to_string(),
            "--disable-features=ChromeWhatsNewUI,MediaRouter,DialMediaRouteProvider".to_string(),
            "--metrics-recording-only".to_string(),
            "--mute-audio".to_string(),
            "--hide-scrollbars".to_string(),
            // Suppress navigator.webdriver and other headless signals that trigger bot detection
            "--disable-blink-features=AutomationControlled".to_string(),
            // Use the real installed Chrome version in the UA to avoid bot-detection fingerprint mismatch
            format!("--user-agent={}", user_agent),
            // Language header — many bot detectors reject requests with no Accept-Language
            "--lang=en-US,en;q=0.9".to_string(),
            "--accept-lang=en-US,en;q=0.9".to_string(),
        ];

        if disable_gpu {
            args.push("--disable-gpu".to_string());
            args.push("--disable-software-rasterizer".to_string());
        }

        if options.disable_images {
            args.push("--blink-settings=imagesEnabled=false".to_string());
        }

        // Windows: additional flags to prevent visible window flashes
        #[cfg(target_os = "windows")]
        {
            args.push("--disable-background-mode".to_string());
            args.push("--disable-extensions".to_string());
        }

        args
    }

    fn create_user_data_dir() -> Result<PathBuf> {
        let pid = std::process::id();
        let nonce = BROWSER_PROFILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to create browser profile timestamp: {}", e),
            })?
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("auditmysite-chrome-{}-{}-{}", pid, unique, nonce));
        std::fs::create_dir_all(&dir).map_err(|e| AuditError::BrowserLaunchFailed {
            reason: format!(
                "Failed to create browser profile directory {}: {}",
                dir.display(),
                e
            ),
        })?;
        Ok(dir)
    }

    /// Create a new page (tab) in the browser
    pub async fn new_page(&self) -> Result<Page> {
        use chromiumoxide::cdp::browser_protocol::emulation::SetFocusEmulationEnabledParams;
        use chromiumoxide::cdp::browser_protocol::network::{Headers, SetExtraHttpHeadersParams};
        use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;

        let page = self.browser.new_page("about:blank").await.map_err(|e| {
            AuditError::BrowserLaunchFailed {
                reason: format!("Failed to create new page: {}", e),
            }
        })?;

        // Tabs of one window are hidden except the active one: a hidden page
        // never runs requestAnimationFrame and has no focus. In a batch that
        // timed out the tab walk on 7 of 10 pages and measured focus
        // differently than a single-URL run of the same page. Focus
        // emulation makes every tab focused and visible; an own window per
        // page did the same but stalled Chrome with several heavy sites
        // rendering at once (plan 62).
        page.execute(SetFocusEmulationEnabledParams::new(true))
            .await
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to enable focus emulation: {}", e),
            })?;

        // Patch navigator.webdriver = undefined on every document load.
        // --disable-blink-features=AutomationControlled removes it at the Blink level,
        // but some sites probe it via JS after load; this CDP injection covers that gap.
        let _ = page
            .execute(AddScriptToEvaluateOnNewDocumentParams::new(
                "Object.defineProperty(navigator,'webdriver',{get:()=>undefined});",
            ))
            .await;

        // Send Do-Not-Track / Global Privacy Control on every request, and
        // expose the matching JS-visible navigator properties (headers alone
        // aren't visible to page scripts that check `navigator.doNotTrack`/
        // `navigator.globalPrivacyControl` instead of the request headers).
        // This is a real, non-blocking privacy signal sent by actual
        // browsers (Firefox, Brave) -- it identifies the visit as
        // opted-out-of-tracking, not as a bot, so it carries none of the
        // detection/blocking risk a custom "bot" User-Agent would. Best
        // effort by design: GA4/Hotjar-style tools mostly ignore it, but
        // GPC is increasingly honored by CMPs under US state privacy law
        // (CPRA/Colorado/Connecticut) and by consent-mode-aware setups. An
        // audit visit shouldn't pollute the site owner's real analytics
        // regardless, so this is unconditional rather than opt-in.
        let _ = page
            .execute(SetExtraHttpHeadersParams::new(Headers::new(
                serde_json::json!({
                    "DNT": "1",
                    "Sec-GPC": "1",
                }),
            )))
            .await;
        let _ = page
            .execute(AddScriptToEvaluateOnNewDocumentParams::new(
                "Object.defineProperty(navigator,'doNotTrack',{get:()=>'1'});\
                 Object.defineProperty(navigator,'globalPrivacyControl',{get:()=>true});",
            ))
            .await;

        Ok(page)
    }

    /// Navigate a page to a URL and wait for load.
    ///
    /// When the loaded document is a transient refresh interstitial (a bot or
    /// traffic check such as berlin.de's "Einen Augenblick bitte / Just a moment
    /// please" that sends the visitor back to the requested URL via a short
    /// `<meta http-equiv="refresh">`), the refresh is followed so that the audit
    /// sees the page a visitor ends up reading, not the interstitial. A genuine
    /// meta-refresh redirect to another page is not followed: the requested page
    /// is loaded again, its pending refresh is cancelled once it has finished
    /// loading, and it is audited as served.
    ///
    /// Returns what was done about a short meta refresh, `None` when the
    /// loaded document had none.
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<Option<MetaRefresh>> {
        self.load(page, url).await?;
        let refresh = follow_refresh_interstitial(page, url).await;
        if refresh.is_some_and(|r| r.outcome == MetaRefreshOutcome::RedirectNotFollowed) {
            info!(
                "Meta refresh on {} leads to another page; auditing the requested page as served",
                url
            );
            self.load(page, url).await?;
            cancel_pending_refresh(page).await;
        }
        Ok(refresh)
    }

    /// Navigate once (with retry) and wait until the document is interactive.
    async fn load(&self, page: &Page, url: &str) -> Result<()> {
        let timeout = Duration::from_secs(self.options.timeout_secs);
        let max_retries = 1;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                warn!(
                    "Retrying navigation to {} (attempt {}/{})",
                    url,
                    attempt + 1,
                    max_retries + 1
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            // Step 1: fire goto() with a shorter timeout.
            // chromiumoxide's goto() waits for the CDP Page.navigate response, which some
            // sites (e.g. Astro View Transitions, aggressive keep-alive) never send back
            // cleanly. We treat a goto timeout as a soft failure and check readyState next.
            let goto_timeout = Duration::from_secs((self.options.timeout_secs / 2).max(10));
            let goto_result = tokio::time::timeout(goto_timeout, page.goto(url)).await;

            let hard_navigation_error = match goto_result {
                Ok(Ok(_)) => None,
                Ok(Err(e)) => {
                    // CDP returned an explicit error (SSL, DNS, etc.)
                    let msg = e.to_string();
                    if msg.contains("ERR_") || msg.contains("net::") {
                        Some(AuditError::NavigationFailed {
                            url: url.to_string(),
                            reason: msg,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => {
                    // goto() timed out — page may still be loading in Chrome
                    debug!("goto() timed out for {}; checking readyState", url);
                    None
                }
            };

            if let Some(err) = hard_navigation_error {
                last_error = Some(err);
                continue;
            }

            // Step 2: poll document.readyState until interactive/complete or timeout.
            let remaining = timeout.saturating_sub(goto_timeout);
            let dom_ready = tokio::time::timeout(remaining, async {
                loop {
                    if let Ok(result) = page.evaluate("document.readyState").await {
                        let state = result.value().and_then(|v| v.as_str()).unwrap_or("");
                        if state == "complete" || state == "interactive" {
                            return true;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            })
            .await
            .unwrap_or(false);

            if dom_ready {
                debug!("Successfully navigated to: {}", url);
                return Ok(());
            }

            last_error = Some(AuditError::PageLoadTimeout {
                url: url.to_string(),
                timeout_secs: self.options.timeout_secs,
            });
        }

        Err(last_error.unwrap_or(AuditError::NavigationFailed {
            url: url.to_string(),
            reason: "Navigation failed with no recorded error".to_string(),
        }))
    }

    /// Get Chrome installation info
    pub fn chrome_info(&self) -> &ChromeInfo {
        &self.chrome_info
    }

    /// Get Chrome binary path
    pub fn chrome_path(&self) -> &PathBuf {
        &self.chrome_info.path
    }

    /// Get Chrome version
    pub fn chrome_version(&self) -> Option<&str> {
        self.chrome_info.version.as_deref()
    }

    /// Whether this browser session runs headless. Exposed for report
    /// provenance without leaking executable paths or launch arguments.
    pub fn is_headless(&self) -> bool {
        self.options.headless
    }

    /// Close the browser gracefully
    pub async fn close(mut self) -> Result<()> {
        info!("Closing browser...");

        if let Ok(pages) = self.browser.pages().await {
            for page in pages {
                if let Err(e) = page.close().await {
                    warn!("Failed to close page: {}", e);
                }
            }
        }

        // Close the Chrome process explicitly so chromiumoxide's own `Drop`
        // (which runs when `self.browser` is dropped below) sees the child
        // already exited instead of logging its own "was not closed
        // manually" warning on every single run.
        if let Err(e) = self.browser.close().await {
            warn!("Failed to close browser: {}", e);
        }
        if let Err(e) = self.browser.wait().await {
            warn!("Failed to wait for browser process exit: {}", e);
        }

        if let Err(e) = std::fs::remove_dir_all(&self.user_data_dir) {
            warn!(
                "Failed to remove browser profile directory {}: {}",
                self.user_data_dir.display(),
                e
            );
        }

        info!("Browser closed");
        Ok(())
    }
}

impl Drop for BrowserManager {
    fn drop(&mut self) {
        // `close()` already removes the profile directory on the graceful
        // path; this is a backstop for panics and early returns that skip
        // it, so a crashed audit doesn't leak a Chrome profile directory
        // under the OS temp dir forever (#QA-041). Only synchronous
        // filesystem cleanup is possible here — `Drop` can't run the async
        // CDP page-close calls `close()` does, but the Chrome child process
        // itself is already handled by `Browser`'s own `kill_on_drop`.
        if self.user_data_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.user_data_dir) {
                warn!(
                    "Failed to remove browser profile directory {} on drop: {}",
                    self.user_data_dir.display(),
                    e
                );
            }
        }
    }
}

impl std::fmt::Debug for BrowserManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserManager")
            .field("chrome_info", &self.chrome_info)
            .field("options", &self.options)
            .finish()
    }
}

/// A `<meta http-equiv="refresh">` firing within this many seconds marks the
/// loaded document as a transient interstitial candidate.
const INTERSTITIAL_REFRESH_MAX_SECS: u64 = 5;
/// Upper bound on consecutive refreshes followed for one navigation.
const INTERSTITIAL_MAX_HOPS: u32 = 3;
/// Extra time, beyond the refresh delay, for the next document to become interactive.
const INTERSTITIAL_LOAD_BUDGET: Duration = Duration::from_secs(10);

const META_REFRESH_CONTENT_JS: &str = r#"(() => {
  const m = document.querySelector('meta[http-equiv="refresh" i]');
  return m ? (m.getAttribute('content') || '') : null;
})()"#;

/// What [`BrowserManager::navigate`] did about a short `<meta http-equiv="refresh">`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MetaRefresh {
    pub outcome: MetaRefreshOutcome,
    /// Refreshes followed before the chain settled.
    pub hops: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaRefreshOutcome {
    /// The refresh chain ended on the requested URL: the loaded document was a
    /// transient interstitial, and the page after it is what was audited.
    InterstitialFollowed,
    /// The refresh chain ended on another URL: the requested page was loaded
    /// again with its refresh cancelled, and audited as served.
    RedirectNotFollowed,
}

/// Delay in seconds of a meta-refresh `content` value (`"2; url=/x"`, `"0"`)
/// when it fires within [`INTERSTITIAL_REFRESH_MAX_SECS`].
fn short_refresh_delay_secs(content: &str) -> Option<u64> {
    let digits: String = content
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let secs: u64 = digits.parse().ok()?;
    (secs <= INTERSTITIAL_REFRESH_MAX_SECS).then_some(secs)
}

/// Whether two URLs address the same document (fragment ignored).
fn same_document_url(a: &str, b: &str) -> bool {
    match (url::Url::parse(a), url::Url::parse(b)) {
        (Ok(mut a), Ok(mut b)) => {
            a.set_fragment(None);
            b.set_fragment(None);
            a == b
        }
        _ => a == b,
    }
}

async fn main_frame(page: &Page) -> Option<chromiumoxide::cdp::browser_protocol::page::Frame> {
    use chromiumoxide::cdp::browser_protocol::page::GetFrameTreeParams;
    page.execute(GetFrameTreeParams::default())
        .await
        .ok()
        .map(|r| r.result.frame_tree.frame.clone())
}

async fn pending_short_refresh(page: &Page) -> Option<u64> {
    let value = page.evaluate(META_REFRESH_CONTENT_JS).await.ok()?;
    short_refresh_delay_secs(value.value()?.as_str()?)
}

/// Waits (bounded) for the document to finish loading, then cancels its
/// scheduled meta refresh with `window.stop()`, which at that point has nothing
/// else left to stop.
async fn cancel_pending_refresh(page: &Page) {
    let _ = tokio::time::timeout(INTERSTITIAL_LOAD_BUDGET, async {
        loop {
            if let Ok(state) = page.evaluate("document.readyState").await {
                if state.value().and_then(|v| v.as_str()) == Some("complete") {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;
    if let Err(e) = page.evaluate("window.stop()").await {
        warn!("Cancelling the meta refresh failed: {}", e);
    }
}

/// Follows short meta refreshes until the main frame holds a document without
/// one, then reports whether that document is the requested URL.
///
/// `None` when the loaded document had no short meta refresh.
async fn follow_refresh_interstitial(page: &Page, url: &str) -> Option<MetaRefresh> {
    let mut hops = 0;
    while hops < INTERSTITIAL_MAX_HOPS {
        let Some(delay) = pending_short_refresh(page).await else {
            break;
        };
        let Some(before) = main_frame(page).await else {
            break;
        };
        hops += 1;
        info!(
            "Document at {} refreshes after {}s (interstitial?); waiting for the next document",
            before.url, delay
        );
        let deadline = Duration::from_secs(delay) + INTERSTITIAL_LOAD_BUDGET;
        let replaced = tokio::time::timeout(deadline, async {
            loop {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let Some(frame) = main_frame(page).await else {
                    continue;
                };
                if frame.loader_id == before.loader_id {
                    continue;
                }
                if let Ok(state) = page.evaluate("document.readyState").await {
                    if matches!(
                        state.value().and_then(|v| v.as_str()),
                        Some("interactive" | "complete")
                    ) {
                        return;
                    }
                }
            }
        })
        .await
        .is_ok();
        if !replaced {
            warn!(
                "Meta refresh on {} did not produce a new document",
                before.url
            );
            break;
        }
        // Back on the requested URL: that document is the page (a self-reloading
        // page is audited after its first reload, not followed further).
        if main_frame(page)
            .await
            .is_some_and(|frame| same_document_url(&frame.url, url))
        {
            break;
        }
    }
    if hops == 0 {
        return None;
    }
    let outcome = match main_frame(page).await {
        Some(frame) if !same_document_url(&frame.url, url) => {
            MetaRefreshOutcome::RedirectNotFollowed
        }
        _ => {
            info!(
                "Refresh interstitial on {} resolved after {} hop(s)",
                url, hops
            );
            MetaRefreshOutcome::InterstitialFollowed
        }
    };
    Some(MetaRefresh { outcome, hops })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_refresh_delay_secs() {
        assert_eq!(short_refresh_delay_secs("2; url=/~~delay/"), Some(2));
        assert_eq!(short_refresh_delay_secs("0"), Some(0));
        assert_eq!(
            short_refresh_delay_secs(" 5;URL='https://example.com/'"),
            Some(5)
        );
        assert_eq!(short_refresh_delay_secs("6; url=/x"), None);
        assert_eq!(short_refresh_delay_secs("300"), None);
        assert_eq!(short_refresh_delay_secs("; url=/x"), None);
        assert_eq!(short_refresh_delay_secs(""), None);
    }

    #[test]
    fn test_same_document_url_ignores_fragment_only() {
        assert!(same_document_url(
            "https://www.berlin.de/",
            "https://www.berlin.de/#top"
        ));
        assert!(same_document_url(
            "https://www.berlin.de",
            "https://www.berlin.de/"
        ));
        assert!(!same_document_url(
            "https://www.berlin.de/",
            "https://www.berlin.de/~~delay/"
        ));
        assert!(!same_document_url(
            "https://example.com/old",
            "https://example.com/new"
        ));
    }

    #[test]
    fn test_default_browser_options() {
        let opts = BrowserOptions::default();
        assert!(opts.headless);
        assert!(opts.disable_gpu);
        assert!(!opts.no_sandbox);
        assert!(!opts.disable_images);
        assert_eq!(opts.window_size, (1920, 1080));
        assert_eq!(opts.timeout_secs, 30);
    }

    #[test]
    fn test_build_launch_args_headless() {
        let opts = BrowserOptions::default();
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu, None);

        // --headless is now applied via BrowserConfig::headless_mode(), not in args
        assert!(args
            .iter()
            .all(|a| a != "--headless" && a != "--headless=new"));
        assert!(args.iter().any(|a| a == "--disable-gpu"));
        assert!(args.iter().any(|a| a == "--no-default-browser-check"));
        assert!(!args.iter().any(|a| a.starts_with("--user-data-dir=")));
        assert!(!args.iter().any(|a| a.starts_with("--window-size=")));
    }

    #[test]
    fn test_build_launch_args_docker() {
        let opts = BrowserOptions {
            no_sandbox: true,
            ..Default::default()
        };
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu, None);

        assert!(!args.iter().any(|a| a == "--no-sandbox"));
        assert!(!args.iter().any(|a| a == "--disable-dev-shm-usage"));
    }

    #[test]
    fn test_build_launch_args_with_images_disabled() {
        let opts = BrowserOptions {
            disable_images: true,
            ..Default::default()
        };
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu, None);

        assert!(args.iter().any(|a| a.contains("imagesEnabled=false")));
    }

    #[test]
    fn test_launch_plans_include_fallbacks_for_headless() {
        let plans = BrowserManager::launch_plans(&BrowserOptions::default());
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].label, "headless-new");
        assert_eq!(plans[1].label, "headless-legacy");
        assert_eq!(plans[2].label, "headless-legacy-gpu-enabled");
    }
}
