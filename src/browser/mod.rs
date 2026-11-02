//! Browser management module
//!
//! Handles browser detection, installation, resolution, launch, and lifecycle.
//!
//! Architecture:
//! - types: Core type definitions (BrowserKind, DetectedBrowser, etc.)
//! - platform: Platform contract (macOS ARM64, Linux x64/ARM64, Windows Edge only)
//! - registry: Platform-specific browser paths
//! - detection: System scanning for installed browsers
//! - probe: Bounded version query and headless CDP probe
//! - resolver: Priority-based browser selection
//! - provenance: Which browser measured (report record, no paths)
//! - installer: Explicit browser installation (no auto-download); facade over
//!   install_plan (what is trusted) and install_txn (transactional install)
//! - bootstrap: Consent policy for the one-time managed install
//! - manager: Browser launch and CDP connection
//! - pool: Concurrent page management

pub mod bootstrap;
pub mod consent;
mod detection;
pub mod install_plan;
#[cfg(test)]
mod install_testkit;
pub mod install_txn;
pub mod installer;
mod manager;
#[doc(hidden)]
pub mod mock;
pub mod platform;
mod pool;
pub mod probe;
pub mod provenance;
mod registry;
pub mod resolver;
pub mod state;
pub mod throttle;
pub mod types;

// New API
pub use consent::{handle_post_navigation, inject_consent_cookies, ConsentResult};
pub use detection::detect_all_browsers;
pub use installer::BrowserInstaller;
pub use platform::{Arch, Os, Platform};
pub use probe::{probe_cdp, query_version, ProbeFailure, VersionOutcome};
pub use provenance::{
    BrowserOrigin, BrowserProduct, BrowserProvenance, HeadlessModeKind, PlatformArch, PlatformOs,
};
pub use resolver::{
    diagnose_browsers, diagnose_browsers_static, resolve_browser, BrowserDiagnosis,
    BrowserResolveOptions,
};
pub use types::{
    BrowserKind, BrowserMode, BrowserSource, CandidateReport, CandidateVerdict, DetectedBrowser,
    InstallTarget, RejectionKind, ResolvedBrowser,
};

// Legacy API (still used by main.rs and manager.rs)
pub use detection::{detect_chrome, find_chrome, ChromeInfo};
pub use manager::{BrowserManager, BrowserOptions, MetaRefresh, MetaRefreshOutcome, SessionPolicy};
pub(crate) use pool::PAGE_RETURN_BUDGET_SECS;
pub use pool::{BrowserPool, PoolConfig, PoolStats, PooledPage};
pub use state::StorageState;
pub use throttle::ThrottleProfile;
