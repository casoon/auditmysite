//! Browser management module
//!
//! Handles Chrome/Chromium detection, launch, and lifecycle management.

mod detection;
mod installer;
mod manager;
mod pool;

pub use detection::{detect_chrome, find_chrome, ChromeInfo};
pub use installer::ChromiumInstaller;
pub use manager::{BrowserManager, BrowserOptions};
pub use pool::{BrowserPool, PoolConfig, PoolStats, PooledPage};
