//! Network throttling via CDP `Network.emulateNetworkConditions`.
//!
//! EmulateNetworkConditionsParams is deprecated in CDP in favour of emulateNetworkConditionsByRule,
//! but chromiumoxide 0.8 does not expose the replacement yet.

#[allow(deprecated)]
use chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::Result;

/// Fixed throttle profiles used for automatic performance comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThrottleProfile {
    /// No throttling — baseline measurement
    Unthrottled,
    /// Slow 3G — 500 kbps / 400 ms latency
    Slow3G,
    /// Fast 3G — 1.5 Mbps / 40 ms latency
    Fast3G,
    /// Lighthouse mobile preset — 1.6 Mbps / 150 ms latency
    LhMobile,
}

impl ThrottleProfile {
    /// All profiles run automatically in every single-page audit.
    pub const AUTO_PROFILES: &'static [ThrottleProfile] =
        &[ThrottleProfile::Slow3G, ThrottleProfile::Fast3G, ThrottleProfile::LhMobile];

    pub fn label(self) -> &'static str {
        match self {
            ThrottleProfile::Unthrottled => "unthrottled",
            ThrottleProfile::Slow3G => "slow-3g",
            ThrottleProfile::Fast3G => "fast-3g",
            ThrottleProfile::LhMobile => "lh-mobile",
        }
    }
}

/// Apply a network throttle profile to the given page.
///
/// Must be called before navigation so the initial page load is throttled.
#[allow(deprecated)]
pub async fn apply_throttling(page: &Page, profile: ThrottleProfile) -> Result<()> {
    // All values in bytes/sec (1 kbps = 125 B/s, 1 Mbps = 125 000 B/s)
    let (download, upload, latency_ms) = match profile {
        ThrottleProfile::Unthrottled => (-1.0, -1.0, 0.0),
        ThrottleProfile::Slow3G => (62_500.0, 62_500.0, 400.0),
        ThrottleProfile::Fast3G => (187_500.0, 93_750.0, 40.0),
        ThrottleProfile::LhMobile => (200_000.0, 96_000.0, 150.0),
    };

    debug!(
        "Applying network throttle {:?}: down={:.0} B/s, up={:.0} B/s, latency={:.0} ms",
        profile, download, upload, latency_ms
    );

    page.execute(
        EmulateNetworkConditionsParams::builder()
            .offline(false)
            .latency(latency_ms)
            .download_throughput(download)
            .upload_throughput(upload)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| crate::error::AuditError::NavigationFailed {
        url: "network-throttle".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Disable any active network throttling on the page.
#[allow(deprecated)]
pub async fn disable_throttling(page: &Page) -> Result<()> {
    page.execute(
        EmulateNetworkConditionsParams::builder()
            .offline(false)
            .latency(0.0)
            .download_throughput(-1.0_f64)
            .upload_throughput(-1.0_f64)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| crate::error::AuditError::NavigationFailed {
        url: "network-throttle-disable".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}
