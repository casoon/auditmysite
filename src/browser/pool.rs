//! Browser Pool - Manages multiple browser instances for concurrent auditing
//!
//! Provides a pool of browser pages that can be checked out for use,
//! enabling concurrent URL processing while managing resource usage.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Page;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

use super::manager::{BrowserManager, BrowserOptions};
use crate::error::{AuditError, Result};

/// Configuration for the browser pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of concurrent browser pages
    pub max_pages: usize,
    /// Browser options for all instances
    pub browser_options: BrowserOptions,
    /// Timeout for acquiring a page from the pool
    pub acquire_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pages: 4,
            browser_options: BrowserOptions::default(),
            acquire_timeout_secs: 60,
        }
    }
}

/// A pooled page that automatically returns to the pool when dropped
pub struct PooledPage {
    /// The underlying page
    page: Option<Page>,
    /// Reference to the pool for returning the page
    pool: Arc<BrowserPoolInner>,
}

impl PooledPage {
    /// Get a reference to the underlying page
    pub fn page(&self) -> &Page {
        self.page.as_ref().expect("Page already returned")
    }
}

impl Drop for PooledPage {
    fn drop(&mut self) {
        if let Some(page) = self.page.take() {
            let pool = Arc::clone(&self.pool);
            // Spawn a task to return the page to the pool
            tokio::spawn(async move {
                pool.return_page(page).await;
            });
        }
    }
}

/// Inner pool state
struct BrowserPoolInner {
    /// The browser manager
    browser: BrowserManager,
    /// Available pages
    pages: Mutex<Vec<Page>>,
    /// Semaphore for limiting concurrent pages
    semaphore: Semaphore,
    /// Total pages created
    pages_created: AtomicUsize,
    /// Maximum pages allowed
    max_pages: usize,
    /// Acquire timeout
    acquire_timeout: Duration,
}

impl BrowserPoolInner {
    /// Return a page to the pool
    async fn return_page(&self, page: Page) {
        // Try to reset the page for reuse with timeout
        let reset_result =
            tokio::time::timeout(Duration::from_secs(5), page.goto("about:blank")).await;

        match reset_result {
            Ok(Ok(_)) => {
                // Page reset successfully, return to pool
                let mut pages = self.pages.lock().await;
                pages.push(page);
                self.semaphore.add_permits(1);
                debug!("Page returned to pool ({} available)", pages.len());
            }
            Ok(Err(e)) => {
                warn!("Failed to reset page: {}", e);
                // Page is unusable, don't return it
                self.semaphore.add_permits(1);
            }
            Err(_) => {
                warn!("Page reset timed out after 5 seconds");
                // Page is unusable, don't return it
                self.semaphore.add_permits(1);
            }
        }
    }
}

/// Browser Pool - manages multiple browser pages for concurrent processing
pub struct BrowserPool {
    inner: Arc<BrowserPoolInner>,
}

impl BrowserPool {
    /// Create a new browser pool with the given configuration
    ///
    /// # Arguments
    /// * `config` - Pool configuration
    ///
    /// # Returns
    /// * `Ok(BrowserPool)` - Pool created successfully
    /// * `Err(AuditError)` - Failed to create pool
    pub async fn new(config: PoolConfig) -> Result<Self> {
        info!("Creating browser pool with max {} pages", config.max_pages);

        // Launch the browser
        let browser = BrowserManager::with_options(config.browser_options).await?;

        let inner = Arc::new(BrowserPoolInner {
            browser,
            pages: Mutex::new(Vec::with_capacity(config.max_pages)),
            semaphore: Semaphore::new(config.max_pages),
            pages_created: AtomicUsize::new(0),
            max_pages: config.max_pages,
            acquire_timeout: Duration::from_secs(config.acquire_timeout_secs),
        });

        Ok(Self { inner })
    }

    /// Create a pool with default configuration
    pub async fn with_concurrency(concurrency: usize) -> Result<Self> {
        Self::new(PoolConfig {
            max_pages: concurrency,
            ..Default::default()
        })
        .await
    }

    /// Acquire a page from the pool
    ///
    /// This will block until a page is available or timeout occurs.
    ///
    /// # Returns
    /// * `Ok(PooledPage)` - A page from the pool
    /// * `Err(AuditError)` - Failed to acquire page
    pub async fn acquire(&self) -> Result<PooledPage> {
        debug!("Acquiring page from pool...");

        // Wait for a permit with timeout
        let permit =
            tokio::time::timeout(self.inner.acquire_timeout, self.inner.semaphore.acquire())
                .await
                .map_err(|_| AuditError::PoolTimeout {
                    timeout_secs: self.inner.acquire_timeout.as_secs(),
                })?
                .map_err(|_| AuditError::PoolClosed)?;

        // Forget the permit since we manage page count manually
        permit.forget();

        // Try to get an existing page from the pool
        let mut pages = self.inner.pages.lock().await;
        if let Some(page) = pages.pop() {
            debug!("Reusing existing page from pool");
            return Ok(PooledPage {
                page: Some(page),
                pool: Arc::clone(&self.inner),
            });
        }
        drop(pages); // Release the lock before creating a new page

        // Create a new page if we haven't hit the limit
        let current = self.inner.pages_created.fetch_add(1, Ordering::SeqCst);
        if current >= self.inner.max_pages {
            // This shouldn't happen due to semaphore, but handle it anyway
            self.inner.pages_created.fetch_sub(1, Ordering::SeqCst);
            return Err(AuditError::PoolExhausted);
        }

        debug!(
            "Creating new page ({}/{})",
            current + 1,
            self.inner.max_pages
        );
        let page = self.inner.browser.new_page().await?;

        Ok(PooledPage {
            page: Some(page),
            pool: Arc::clone(&self.inner),
        })
    }

    /// Get the browser manager for navigation
    pub fn browser(&self) -> &BrowserManager {
        &self.inner.browser
    }

    /// Get the number of available pages in the pool
    pub async fn available_pages(&self) -> usize {
        self.inner.pages.lock().await.len()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            max_pages: self.inner.max_pages,
            pages_created: self.inner.pages_created.load(Ordering::SeqCst),
            permits_available: self.inner.semaphore.available_permits(),
        }
    }

    /// Close the pool and release all resources
    pub async fn close(self) -> Result<()> {
        info!("Closing browser pool...");

        // Close all pooled pages
        let mut pages = self.inner.pages.lock().await;
        for page in pages.drain(..) {
            if let Err(e) = page.close().await {
                warn!("Failed to close pooled page: {}", e);
            }
        }

        // The browser will be closed when the Arc is dropped
        info!("Browser pool closed");
        Ok(())
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Maximum number of pages allowed
    pub max_pages: usize,
    /// Total pages created
    pub pages_created: usize,
    /// Currently available permits
    pub permits_available: usize,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool: {}/{} pages, {} available",
            self.pages_created, self.max_pages, self.permits_available
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_pages, 4);
        assert_eq!(config.acquire_timeout_secs, 60);
    }

    #[test]
    fn test_pool_stats_display() {
        let stats = PoolStats {
            max_pages: 4,
            pages_created: 2,
            permits_available: 2,
        };
        let display = format!("{}", stats);
        assert!(display.contains("2/4"));
        assert!(display.contains("2 available"));
    }
}
