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
use super::types::DetectedBrowser;
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
        }
    }
}

/// Browser Manager - handles Chrome lifecycle
pub struct BrowserManager {
    browser: Browser,
    chrome_info: ChromeInfo,
    options: BrowserOptions,
    user_data_dir: PathBuf,
    #[allow(dead_code)]
    handler: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
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
        let args = Self::build_launch_args(&options, options.disable_gpu);
        debug!("Chrome launch args: {:?}", args);

        // Use resolver to find browser
        let resolve_opts = BrowserResolveOptions {
            browser_path: options.chrome_path.clone(),
            browser_preference: None,
            strict: false,
        };

        let resolved = match resolver::resolve_browser(&resolve_opts) {
            Ok(resolved) => {
                info!(
                    "Using {}: {} v{}",
                    resolved.browser.kind.display_name(),
                    resolved.browser.path.display(),
                    resolved.browser.version.as_deref().unwrap_or("unknown")
                );
                resolved
            }
            Err(_) => {
                // Fallback: try managed install via legacy installer
                info!("No system browser found, trying managed install...");
                let chromium_path = super::installer::ChromiumInstaller::ensure_chromium().await?;

                let browser = DetectedBrowser {
                    kind: super::types::BrowserKind::ChromeForTesting,
                    path: chromium_path,
                    version: Some("managed".to_string()),
                    source: super::types::BrowserSource::ManagedInstall,
                };
                super::types::ResolvedBrowser {
                    browser,
                    mode: super::types::BrowserMode::Standard,
                    all_candidates: vec![],
                }
            }
        };

        verify_executable(&resolved.browser.path)?;

        let chrome_info = ChromeInfo::from(&resolved.browser);
        let mut launch_errors = Vec::new();
        let mut launched = None;

        for plan in Self::launch_plans(&options) {
            let config =
                Self::build_browser_config(&resolved.browser.path, &user_data_dir, &options, plan)?;

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
            handler: Arc::new(Mutex::new(Some(handler_task))),
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
    ) -> Result<BrowserConfig> {
        let mut builder = BrowserConfig::builder()
            .chrome_executable(browser_path)
            .user_data_dir(user_data_dir)
            .window_size(options.window_size.0, options.window_size.1)
            .headless_mode(plan.headless_mode)
            .args(Self::build_launch_args(options, plan.disable_gpu))
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

    fn build_launch_args(options: &BrowserOptions, disable_gpu: bool) -> Vec<String> {
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
            // Replace HeadlessChrome UA with a standard Chrome UA
            "--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
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
        self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to create new page: {}", e),
            })
    }

    /// Navigate a page to a URL and wait for load
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
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

    /// Close the browser gracefully
    pub async fn close(self) -> Result<()> {
        info!("Closing browser...");

        if let Ok(pages) = self.browser.pages().await {
            for page in pages {
                if let Err(e) = page.close().await {
                    warn!("Failed to close page: {}", e);
                }
            }
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

impl std::fmt::Debug for BrowserManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserManager")
            .field("chrome_info", &self.chrome_info)
            .field("options", &self.options)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu);

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
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu);

        assert!(!args.iter().any(|a| a == "--no-sandbox"));
        assert!(!args.iter().any(|a| a == "--disable-dev-shm-usage"));
    }

    #[test]
    fn test_build_launch_args_with_images_disabled() {
        let opts = BrowserOptions {
            disable_images: true,
            ..Default::default()
        };
        let args = BrowserManager::build_launch_args(&opts, opts.disable_gpu);

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
