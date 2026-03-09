//! Browser management module
//!
//! Handles browser detection, installation, resolution, launch, and lifecycle.
//!
//! Architecture:
//! - types: Core type definitions (BrowserKind, DetectedBrowser, etc.)
//! - registry: Platform-specific browser paths
//! - detection: System scanning for installed browsers
//! - resolver: Priority-based browser selection
//! - installer: Explicit browser installation (no auto-download)
//! - manager: Browser launch and CDP connection
//! - pool: Concurrent page management

pub mod types;
mod registry;
mod detection;
pub mod resolver;
pub mod installer;
mod manager;
mod pool;

// New API
pub use types::{BrowserKind, BrowserSource, BrowserMode, DetectedBrowser, ResolvedBrowser, InstallTarget};
pub use detection::detect_all_browsers;
pub use resolver::{resolve_browser, BrowserResolveOptions};
pub use installer::BrowserInstaller;

// Legacy API (still used by main.rs and manager.rs)
pub use detection::{detect_chrome, find_chrome, ChromeInfo};
pub use installer::ChromiumInstaller;
pub use manager::{BrowserManager, BrowserOptions};
pub use pool::{BrowserPool, PoolConfig, PoolStats, PooledPage};
