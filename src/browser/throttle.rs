//! Network and CPU throttling via CDP.
//!
//! Network: `Network.emulateNetworkConditions` (deprecated in CDP but still functional).
//! CPU: `Emulation.setCPUThrottlingRate` — required to simulate realistic mobile LCP/TBT.

use chromiumoxide::cdp::browser_protocol::emulation::SetCpuThrottlingRateParams;
#[allow(deprecated)]
use chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams;
use chromiumoxide::cdp::browser_protocol::network::SetCacheDisabledParams;
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
    pub const AUTO_PROFILES: &'static [ThrottleProfile] = &[
        ThrottleProfile::Slow3G,
        ThrottleProfile::Fast3G,
        ThrottleProfile::LhMobile,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThrottleProfile::Unthrottled => "unthrottled",
            ThrottleProfile::Slow3G => "slow-3g",
            ThrottleProfile::Fast3G => "fast-3g",
            ThrottleProfile::LhMobile => "lh-mobile",
        }
    }

    /// CPU slowdown factor matching Lighthouse's mobile simulation.
    ///
    /// Without CPU throttling, LCP/TBT values measured under network throttle are
    /// unrealistically low because the browser executes JS at full dev-machine speed.
    pub fn cpu_slowdown(self) -> f64 {
        match self {
            ThrottleProfile::Unthrottled => 1.0,
            ThrottleProfile::Slow3G => 6.0,
            ThrottleProfile::Fast3G => 4.0,
            ThrottleProfile::LhMobile => 4.0,
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

/// Apply CPU throttling for a given profile.
///
/// Must be called before navigation; resets to 1x with `disable_cpu_throttling`.
pub async fn apply_cpu_throttling(page: &Page, profile: ThrottleProfile) -> Result<()> {
    let rate = profile.cpu_slowdown();
    debug!("Applying CPU throttle {:?}: {:.0}x slowdown", profile, rate);
    page.execute(SetCpuThrottlingRateParams::new(rate))
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "cpu-throttle".to_string(),
            reason: e.to_string(),
        })?;
    Ok(())
}

/// Reset CPU throttling to 1x (no slowdown).
pub async fn disable_cpu_throttling(page: &Page) -> Result<()> {
    page.execute(SetCpuThrottlingRateParams::new(1.0_f64))
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "cpu-throttle-disable".to_string(),
            reason: e.to_string(),
        })?;
    Ok(())
}

/// Disable the browser cache so subsequent navigations fetch all resources
/// over the (throttled) network rather than serving from cache.
pub async fn disable_cache(page: &Page) -> Result<()> {
    page.execute(SetCacheDisabledParams::new(true))
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "cache-disable".to_string(),
            reason: e.to_string(),
        })?;
    Ok(())
}

/// Re-enable the browser cache after a throttled measurement pass.
pub async fn enable_cache(page: &Page) -> Result<()> {
    page.execute(SetCacheDisabledParams::new(false))
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "cache-enable".to_string(),
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

#[cfg(test)]
mod tests {
    // Verify the cache control functions are reachable (compile-time check).
    // Full behaviour requires a live browser; referencing the symbols suffices.
    #[test]
    fn cache_fns_exist() {
        // If these names don't exist the file won't compile.
        let _disable = super::disable_cache;
        let _enable = super::enable_cache;
    }
}
