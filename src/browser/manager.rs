//! Browser Manager - Chrome lifecycle management
//!
//! Handles launching Chrome in headless mode with optimized flags,
//! managing CDP connections, and graceful shutdown.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Page;
use futures::StreamExt;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::detection::{verify_executable, ChromeInfo};
use super::resolver::{self, BrowserResolveOptions};
use super::types::DetectedBrowser;
use crate::error::{AuditError, Result};

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
    #[allow(dead_code)]
    handler: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl BrowserManager {
    /// Create a new BrowserManager with default options
    pub async fn new() -> Result<Self> {
        Self::with_options(BrowserOptions::default()).await
    }

    /// Create a new BrowserManager with custom options
    pub async fn with_options(options: BrowserOptions) -> Result<Self> {
        let args = Self::build_launch_args(&options);
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

        let config = BrowserConfig::builder()
            .chrome_executable(&resolved.browser.path)
            .args(args)
            .viewport(None)
            .build()
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: e.to_string(),
            })?;

        let chrome_info = ChromeInfo::from(&resolved.browser);

        // Launch browser
        let (browser, mut handler) =
            Browser::launch(config)
                .await
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: e.to_string(),
                })?;

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                debug!("Browser event: {:?}", event);
            }
        });

        info!("Browser launched successfully");

        Ok(Self {
            browser,
            chrome_info,
            options,
            handler: Arc::new(Mutex::new(Some(handler_task))),
        })
    }

    /// Build Chrome launch arguments based on options
    fn build_launch_args(options: &BrowserOptions) -> Vec<String> {
        let mut args = vec![
            if options.headless {
                "--headless".to_string()
            } else {
                "--no-headless".to_string()
            },
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
            "--disable-extensions".to_string(),
            "--disable-background-networking".to_string(),
            "--disable-sync".to_string(),
            "--disable-translate".to_string(),
            "--disable-features=TranslateUI".to_string(),
            "--metrics-recording-only".to_string(),
            "--mute-audio".to_string(),
            "--disable-infobars".to_string(),
            "--disable-popup-blocking".to_string(),
            format!(
                "--window-size={},{}",
                options.window_size.0, options.window_size.1
            ),
        ];

        if options.disable_gpu {
            args.push("--disable-gpu".to_string());
            args.push("--disable-software-rasterizer".to_string());
        }

        if options.no_sandbox {
            args.push("--no-sandbox".to_string());
            args.push("--disable-setuid-sandbox".to_string());
            args.push("--disable-dev-shm-usage".to_string());
        }

        if options.disable_images {
            args.push("--blink-settings=imagesEnabled=false".to_string());
        }

        args
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

            match tokio::time::timeout(timeout, async {
                page.goto(url)
                    .await
                    .map_err(|e| AuditError::NavigationFailed {
                        url: url.to_string(),
                        reason: e.to_string(),
                    })?;

                page.wait_for_navigation()
                    .await
                    .map_err(|e| AuditError::NavigationFailed {
                        url: url.to_string(),
                        reason: format!("Navigation wait failed: {}", e),
                    })?;

                Ok::<(), AuditError>(())
            })
            .await
            {
                Ok(Ok(())) => {
                    debug!("Successfully navigated to: {}", url);
                    return Ok(());
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                }
                Err(_) => {
                    last_error = Some(AuditError::PageLoadTimeout {
                        url: url.to_string(),
                        timeout_secs: self.options.timeout_secs,
                    });
                }
            }
        }

        Err(last_error.unwrap())
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
        let args = BrowserManager::build_launch_args(&opts);

        assert!(args.iter().any(|a| a == "--headless"));
        assert!(args.iter().any(|a| a == "--disable-gpu"));
        assert!(args.iter().any(|a| a == "--no-first-run"));
        assert!(args.iter().any(|a| a.starts_with("--window-size=")));
    }

    #[test]
    fn test_build_launch_args_docker() {
        let opts = BrowserOptions {
            no_sandbox: true,
            ..Default::default()
        };
        let args = BrowserManager::build_launch_args(&opts);

        assert!(args.iter().any(|a| a == "--no-sandbox"));
        assert!(args.iter().any(|a| a == "--disable-dev-shm-usage"));
    }

    #[test]
    fn test_build_launch_args_with_images_disabled() {
        let opts = BrowserOptions {
            disable_images: true,
            ..Default::default()
        };
        let args = BrowserManager::build_launch_args(&opts);

        assert!(args.iter().any(|a| a.contains("imagesEnabled=false")));
    }
}
