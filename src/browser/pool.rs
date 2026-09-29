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

/// How long a returned page may take to reset to `about:blank` before it is
/// discarded instead.
const PAGE_RESET_TIMEOUT: Duration = Duration::from_secs(10);
/// How long closing a discarded page may take.
const PAGE_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
/// Upper bound on how long a page's slot stays taken after its user has
/// dropped it: reset, and on failure close (#651). A waiter that should
/// outlast one in-flight page has to add this on top of that page's budget.
pub(crate) const PAGE_RETURN_BUDGET_SECS: u64 =
    PAGE_RESET_TIMEOUT.as_secs() + PAGE_CLOSE_TIMEOUT.as_secs();

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
    pub fn page(&self) -> Result<&Page> {
        self.page.as_ref().ok_or_else(|| {
            crate::error::AuditError::ConfigError("Page already returned to pool".to_string())
        })
    }
}

impl Drop for PooledPage {
    fn drop(&mut self) {
        if let Some(page) = self.page.take() {
            // `tokio::spawn` panics if no runtime is active for this thread —
            // normally guaranteed for the program's whole lifetime, but not
            // during process shutdown/panic-unwind after the runtime itself
            // has already been torn down (#QA-041). Skip the spawn rather
            // than panic in that case; the page (and its Chrome tab) is
            // simply leaked, same as if the process were killed outright.
            if tokio::runtime::Handle::try_current().is_ok() {
                let pool = Arc::clone(&self.pool);
                tokio::spawn(async move {
                    pool.return_page(page).await;
                });
            } else {
                warn!("No active tokio runtime; dropping page without returning it to the pool");
            }
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
    /// Upper bound for creating one new page (the browser's CDP timeout)
    page_create_timeout: Duration,
}

impl BrowserPoolInner {
    /// Return a page to the pool
    async fn return_page(&self, page: Page) {
        // Reset the page for reuse. Guard with a short timeout — some pages become
        // unresponsive after navigating to heavy sites and the goto call would hang
        // indefinitely, starving the semaphore.
        let reset = tokio::time::timeout(PAGE_RESET_TIMEOUT, page.goto("about:blank")).await;

        match reset {
            Ok(Ok(_)) => {
                let mut pages = self.pages.lock().await;
                pages.push(page);
                self.semaphore.add_permits(1);
                debug!("Page returned to pool ({} available)", pages.len());
            }
            Ok(Err(e)) => {
                warn!("Failed to reset page: {}", e);
                self.discard_page(page).await;
            }
            Err(_) => {
                warn!(
                    "Page reset timed out after {}s, discarding page",
                    PAGE_RESET_TIMEOUT.as_secs()
                );
                self.discard_page(page).await;
            }
        }
    }

    /// Discard a page that couldn't be reset for reuse: close its Chrome tab
    /// and give back its slot in `pages_created`, not just its semaphore
    /// permit (#QA-040). Without the `pages_created` decrement, every page
    /// lost this way permanently shrinks the pool's effective capacity —
    /// `acquire()`'s "shouldn't happen" exhaustion branch was actually
    /// reachable once enough pages were discarded on a long batch run,
    /// collapsing concurrency to zero with free semaphore permits sitting
    /// unused. `page.close()` is itself timeout-guarded since a page
    /// unresponsive enough to fail the reset above may also fail to close;
    /// the bookkeeping happens either way so a hung close can't reintroduce
    /// the same exhaustion bug.
    async fn discard_page(&self, page: Page) {
        match tokio::time::timeout(PAGE_CLOSE_TIMEOUT, page.close()).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!("Failed to close discarded page: {}", e),
            Err(_) => warn!(
                "Closing discarded page timed out after {}s",
                PAGE_CLOSE_TIMEOUT.as_secs()
            ),
        }
        self.pages_created.fetch_sub(1, Ordering::SeqCst);
        self.semaphore.add_permits(1);
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

        let page_create_timeout = Duration::from_secs(config.browser_options.timeout_secs);
        // Launch the browser
        let browser = BrowserManager::with_options(config.browser_options).await?;

        let inner = Arc::new(BrowserPoolInner {
            browser,
            pages: Mutex::new(Vec::with_capacity(config.max_pages)),
            semaphore: Semaphore::new(config.max_pages),
            pages_created: AtomicUsize::new(0),
            max_pages: config.max_pages,
            acquire_timeout: Duration::from_secs(config.acquire_timeout_secs),
            page_create_timeout,
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
        // The permit was forgotten and the slot counted above; a failed page
        // creation has to give both back, or the pool shrinks by one slot for
        // the rest of the run and later pages wait for a slot that never
        // frees up (#651). Bounded, because a Chrome saturated by heavy pages
        // can leave the page creation unanswered for good.
        let created = tokio::time::timeout(
            self.inner.page_create_timeout,
            self.inner.browser.new_page(),
        )
        .await
        .unwrap_or_else(|_| {
            Err(AuditError::BrowserLaunchFailed {
                reason: format!(
                    "Failed to create new page: no answer after {} seconds",
                    self.inner.page_create_timeout.as_secs()
                ),
            })
        });
        let page = match created {
            Ok(page) => page,
            Err(e) => {
                self.inner.pages_created.fetch_sub(1, Ordering::SeqCst);
                self.inner.semaphore.add_permits(1);
                return Err(e);
            }
        };

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
        {
            let mut pages = self.inner.pages.lock().await;
            for page in pages.drain(..) {
                if let Err(e) = page.close().await {
                    warn!("Failed to close pooled page: {}", e);
                }
            }
        }

        // Actually close the underlying browser process gracefully instead
        // of just letting the Arc drop implicitly at the end of this
        // function. A plain drop only relies on chromiumoxide's
        // `Browser::drop` "kill_on_drop", which schedules a *background*
        // task on the Tokio runtime to deliver the kill signal -- fine for
        // long-running processes, but callers that tear the process down
        // right after this returns (batch mode returns straight into
        // `main()`'s `std::process::exit`) don't give that background task
        // a chance to run, orphaning the Chrome process. Confirmed live
        // (2026-09-01): the old code logged "Browser pool closed"
        // successfully every time, yet the Chrome process was still running
        // afterward every time too -- this was the missing piece, not just
        // the caller never invoking `close()` at all.
        //
        // `Arc::try_unwrap` can still legitimately fail right here: the last
        // audited page's `PooledPage` was likely just dropped by the caller
        // moments ago, and `PooledPage::drop` spawns a detached
        // "return-to-pool" task holding its own clone of this same Arc (see
        // `PooledPage::drop`) -- that task may not have even been scheduled
        // yet. Retry briefly (bounded, ~2s total) instead of giving up on
        // the very first attempt; also confirmed live to be the actual
        // failure mode (2026-09-01), not a hypothetical.
        let mut inner = Arc::try_unwrap(self.inner);
        for _ in 0..20 {
            if inner.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            inner = match inner {
                Ok(i) => Ok(i),
                Err(arc) => Arc::try_unwrap(arc),
            };
        }
        match inner {
            Ok(inner) => {
                if let Err(e) = inner.browser.close().await {
                    warn!("Failed to close browser: {}", e);
                }
            }
            Err(_) => {
                warn!(
                    "Browser pool inner state still had outstanding references after \
                     waiting; the browser process may not shut down cleanly"
                );
            }
        }

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
