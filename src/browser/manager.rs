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

use super::detection::{find_chrome, verify_chrome_executable, ChromeInfo};
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
            no_sandbox: false,     // Only enable when needed (Docker/root)
            disable_images: false, // Keep images for contrast checking
            window_size: (1920, 1080),
            timeout_secs: 30,
            verbose: false,
        }
    }
}

/// Browser Manager - handles Chrome lifecycle
pub struct BrowserManager {
    /// The chromiumoxide browser instance
    browser: Browser,
    /// Chrome installation info
    chrome_info: ChromeInfo,
    /// Configuration options
    options: BrowserOptions,
    /// Handler for browser events
    #[allow(dead_code)]
    handler: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl BrowserManager {
    /// Create a new BrowserManager with default options
    ///
    /// # Returns
    /// * `Ok(BrowserManager)` - Browser launched successfully
    /// * `Err(AuditError)` - Failed to launch browser
    pub async fn new() -> Result<Self> {
        Self::with_options(BrowserOptions::default()).await
    }

    /// Create a new BrowserManager with custom options
    pub async fn with_options(options: BrowserOptions) -> Result<Self> {
        // Build launch arguments
        let args = Self::build_launch_args(&options);
        debug!("Chrome launch args: {:?}", args);

        // Configure browser with auto-download support
        let config = if let Some(chrome_path) = &options.chrome_path {
            // User specified a Chrome path - use it
            let chrome_info = find_chrome(Some(chrome_path.as_str()))?;
            info!(
                "Using specified Chrome at: {:?} (version: {:?})",
                chrome_info.path, chrome_info.version
            );
            verify_chrome_executable(&chrome_info.path)?;

            BrowserConfig::builder()
                .chrome_executable(&chrome_info.path)
                .args(args)
                .viewport(None)
                .build()
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: e.to_string(),
                })?
        } else {
            // No path specified - try system Chrome first, then download
            match find_chrome(None) {
                Ok(chrome_info) => {
                    info!("Found system Chrome: {:?}", chrome_info.path);
                    verify_chrome_executable(&chrome_info.path)?;

                    BrowserConfig::builder()
                        .chrome_executable(&chrome_info.path)
                        .args(args)
                        .viewport(None)
                        .build()
                        .map_err(|e| AuditError::BrowserLaunchFailed {
                            reason: e.to_string(),
                        })?
                }
                Err(_) => {
                    // System Chrome not found - download Chromium
                    info!("No system Chrome found, downloading Chromium...");
                    let chromium_path =
                        super::installer::ChromiumInstaller::ensure_chromium().await?;

                    BrowserConfig::builder()
                        .chrome_executable(&chromium_path)
                        .args(args)
                        .viewport(None)
                        .build()
                        .map_err(|e| AuditError::BrowserLaunchFailed {
                            reason: e.to_string(),
                        })?
                }
            }
        };

        let chrome_info = if options.chrome_path.is_some() {
            find_chrome(options.chrome_path.as_deref())?
        } else {
            // For auto-downloaded Chromium, create a placeholder info
            ChromeInfo {
                path: PathBuf::from("~/.cache/chromiumoxide/"),
                version: Some("auto-downloaded".to_string()),
                detection_method: super::detection::DetectionMethod::AutoDownload,
            }
        };

        // Launch browser
        let (browser, mut handler) =
            Browser::launch(config)
                .await
                .map_err(|e| AuditError::BrowserLaunchFailed {
                    reason: e.to_string(),
                })?;

        // Spawn handler task to process browser events
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
            // Headless mode (use old mode for better compatibility)
            if options.headless {
                "--headless".to_string()
            } else {
                "--no-headless".to_string()
            },
            // Disable first-run wizards and prompts
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
            // Disable unnecessary features for performance
            "--disable-extensions".to_string(),
            "--disable-background-networking".to_string(),
            "--disable-sync".to_string(),
            "--disable-translate".to_string(),
            "--disable-features=TranslateUI".to_string(),
            "--metrics-recording-only".to_string(),
            "--mute-audio".to_string(),
            "--disable-infobars".to_string(),
            "--disable-popup-blocking".to_string(),
            // Consistent viewport
            format!(
                "--window-size={},{}",
                options.window_size.0, options.window_size.1
            ),
        ];

        // GPU settings
        if options.disable_gpu {
            args.push("--disable-gpu".to_string());
            args.push("--disable-software-rasterizer".to_string());
        }

        // Sandbox (required for Docker/root, but security risk otherwise)
        if options.no_sandbox {
            args.push("--no-sandbox".to_string());
            args.push("--disable-setuid-sandbox".to_string());
        }

        // Shared memory (required for Docker)
        if options.no_sandbox {
            args.push("--disable-dev-shm-usage".to_string());
        }

        // Disable images for faster loading (but keep for contrast checks)
        if options.disable_images {
            args.push("--blink-settings=imagesEnabled=false".to_string());
        }

        args
    }

    /// Create a new page (tab) in the browser
    ///
    /// # Returns
    /// * `Ok(Page)` - New page created
    /// * `Err(AuditError)` - Failed to create page
    pub async fn new_page(&self) -> Result<Page> {
        self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| AuditError::BrowserLaunchFailed {
                reason: format!("Failed to create new page: {}", e),
            })
    }

    /// Navigate a page to a URL and wait for load
    ///
    /// # Arguments
    /// * `page` - The page to navigate
    /// * `url` - The URL to navigate to
    ///
    /// # Returns
    /// * `Ok(())` - Navigation successful
    /// * `Err(AuditError)` - Navigation failed
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        let timeout = Duration::from_secs(self.options.timeout_secs);

        tokio::time::timeout(timeout, async {
            page.goto(url)
                .await
                .map_err(|e| AuditError::NavigationFailed {
                    url: url.to_string(),
                    reason: e.to_string(),
                })?;

            // Wait for network idle
            page.wait_for_navigation()
                .await
                .map_err(|e| AuditError::NavigationFailed {
                    url: url.to_string(),
                    reason: format!("Navigation wait failed: {}", e),
                })?;

            Ok::<(), AuditError>(())
        })
        .await
        .map_err(|_| AuditError::PageLoadTimeout {
            url: url.to_string(),
            timeout_secs: self.options.timeout_secs,
        })??;

        debug!("Successfully navigated to: {}", url);
        Ok(())
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

        // Close all pages first
        if let Ok(pages) = self.browser.pages().await {
            for page in pages {
                if let Err(e) = page.close().await {
                    warn!("Failed to close page: {}", e);
                }
            }
        }

        // Close browser
        // Note: Browser will be dropped when self is dropped
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
