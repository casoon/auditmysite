//! Content weight analyzer
//!
//! Analyzes page resources by type and provides optimization recommendations.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// Content weight analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentWeight {
    /// Total page size in bytes
    pub total_bytes: u64,
    /// Total transfer size (compressed) in bytes
    pub transfer_bytes: u64,
    /// Breakdown by resource type
    pub breakdown: ResourceBreakdown,
    /// Number of requests
    pub request_count: u32,
    /// Estimated carbon footprint for one page view based on transfer bytes.
    #[serde(default)]
    pub carbon: CarbonEstimate,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Sustainable Web Design-style transfer-byte carbon estimate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarbonEstimate {
    /// Estimated grams CO₂e for one page view.
    pub grams_co2e_per_view: f64,
    /// Human-readable efficiency rating.
    pub rating: String,
    /// Model description used for the estimate.
    pub model: String,
}

impl CarbonEstimate {
    /// Formats `grams_co2e_per_view` for display. Truncating a genuinely small
    /// but nonzero footprint to two decimals reads as "0.00 g" — i.e. "no
    /// footprint" — rather than "very small footprint" (same failure class as
    /// the root-cause share-percentage truncation, see
    /// `output/pdf/single_report.rs`). Near-zero values get a "< 0.01"
    /// qualifier instead; all other values keep the normal two-decimal format.
    pub fn format_grams(&self) -> String {
        if self.grams_co2e_per_view > 0.0 && self.grams_co2e_per_view < 0.005 {
            "< 0.01".to_string()
        } else {
            format!("{:.2}", self.grams_co2e_per_view)
        }
    }
}

/// Resource breakdown by type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceBreakdown {
    /// HTML documents
    pub html: ResourceStats,
    /// CSS stylesheets
    pub css: ResourceStats,
    /// JavaScript files
    pub javascript: ResourceStats,
    /// Images (all formats)
    pub images: ResourceStats,
    /// Fonts
    pub fonts: ResourceStats,
    /// Media (video/audio)
    pub media: ResourceStats,
    /// Other resources
    pub other: ResourceStats,
}

/// Statistics for a resource type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Number of resources
    pub count: u32,
    /// Total size in bytes
    pub bytes: u64,
    /// Transfer size (compressed) in bytes
    pub transfer_bytes: u64,
    /// Largest resource size
    pub largest_bytes: u64,
    /// Largest resource URL
    pub largest_url: Option<String>,
}

impl ContentWeight {
    /// Get compression ratio (transfer/total)
    pub fn compression_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            1.0
        } else {
            self.transfer_bytes as f64 / self.total_bytes as f64
        }
    }

    /// Check if page is considered "heavy"
    pub fn is_heavy(&self) -> bool {
        // > 3MB total is considered heavy
        self.total_bytes > 3_000_000
    }

    /// Get formatted total size
    pub fn formatted_total(&self) -> String {
        format_bytes(self.total_bytes)
    }

    /// Get formatted transfer size
    pub fn formatted_transfer(&self) -> String {
        format_bytes(self.transfer_bytes)
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Analyze content weight of a page
///
/// # Arguments
/// * `page` - The chromiumoxide Page to analyze
///
/// # Returns
/// * `Ok(ContentWeight)` - The analysis results
/// * `Err(AuditError)` - If analysis fails
pub async fn analyze_content_weight(page: &Page) -> Result<ContentWeight> {
    info!("Analyzing content weight...");

    // Get resource timing entries via JavaScript.
    // Include the navigation entry (main HTML document) which is not in 'resource' entries.
    let js_code = r#"
    (() => {
        const resources = performance.getEntriesByType('resource');
        const nav = performance.getEntriesByType('navigation')[0];
        const navEntry = nav ? [{
            name: nav.name,
            type: 'navigation',
            transferSize: nav.transferSize || 0,
            decodedSize: nav.decodedBodySize || 0,
            duration: nav.duration
        }] : [];
        return JSON.stringify([...navEntry, ...resources.map(r => ({
            name: r.name,
            type: r.initiatorType,
            transferSize: r.transferSize || 0,
            decodedSize: r.decodedBodySize || 0,
            duration: r.duration
        }))]);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Resource analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("[]");

    let resources: Vec<ResourceEntry> = serde_json::from_str(json_str).unwrap_or_default();

    let mut breakdown = ResourceBreakdown::default();
    let mut total_bytes: u64 = 0;
    let mut transfer_bytes: u64 = 0;
    let mut request_count: u32 = 0;

    for resource in &resources {
        request_count += 1;
        // decodedBodySize is 0 for cached resources and some font/image types;
        // fall back to transferSize so the breakdown doesn't silently show zero.
        let effective_size = if resource.decoded_size > 0 {
            resource.decoded_size
        } else {
            resource.transfer_size
        };
        total_bytes += effective_size;
        transfer_bytes += resource.transfer_size;

        let stats = match categorize_resource(&resource.name, &resource.resource_type) {
            ResourceCategory::Html => &mut breakdown.html,
            ResourceCategory::Css => &mut breakdown.css,
            ResourceCategory::JavaScript => &mut breakdown.javascript,
            ResourceCategory::Image => &mut breakdown.images,
            ResourceCategory::Font => &mut breakdown.fonts,
            ResourceCategory::Media => &mut breakdown.media,
            ResourceCategory::Other => &mut breakdown.other,
        };

        stats.count += 1;
        stats.bytes += effective_size;
        stats.transfer_bytes += resource.transfer_size;

        if effective_size > stats.largest_bytes {
            stats.largest_bytes = effective_size;
            stats.largest_url = Some(truncate_url(&resource.name));
        }
    }

    // Generate recommendations
    let carbon = estimate_carbon(transfer_bytes);
    let recommendations =
        generate_recommendations(&breakdown, total_bytes, transfer_bytes, &carbon);

    info!(
        "Content weight: {} total, {} transfer, {} requests",
        format_bytes(total_bytes),
        format_bytes(transfer_bytes),
        request_count
    );

    Ok(ContentWeight {
        total_bytes,
        transfer_bytes,
        breakdown,
        request_count,
        carbon,
        recommendations,
    })
}

#[derive(Debug, Deserialize)]
struct ResourceEntry {
    name: String,
    #[serde(rename = "type")]
    resource_type: String,
    #[serde(rename = "transferSize")]
    transfer_size: u64,
    #[serde(rename = "decodedSize")]
    decoded_size: u64,
}

enum ResourceCategory {
    Html,
    Css,
    JavaScript,
    Image,
    Font,
    Media,
    Other,
}

fn categorize_resource(url: &str, initiator_type: &str) -> ResourceCategory {
    let url_lower = url.to_lowercase();

    // Font files loaded via CSS @font-face carry initiatorType="css" in
    // PerformanceResourceTiming. Check font extensions before the CSS branch
    // so they are counted in the correct category.
    if url_lower.contains(".woff")
        || url_lower.contains(".ttf")
        || url_lower.contains(".otf")
        || url_lower.contains(".eot")
    {
        return ResourceCategory::Font;
    }

    if initiator_type == "navigation" {
        ResourceCategory::Html
    } else if url_lower.ends_with(".css") || initiator_type == "css" {
        ResourceCategory::Css
    } else if url_lower.ends_with(".js") || initiator_type == "script" {
        ResourceCategory::JavaScript
    } else if url_lower.ends_with(".html") || url_lower.ends_with(".htm") {
        ResourceCategory::Html
    } else if url_lower.ends_with(".png")
        || url_lower.ends_with(".jpg")
        || url_lower.ends_with(".jpeg")
        || url_lower.ends_with(".gif")
        || url_lower.ends_with(".webp")
        || url_lower.ends_with(".svg")
        || url_lower.ends_with(".ico")
        || initiator_type == "img"
    {
        ResourceCategory::Image
    } else if url_lower.ends_with(".mp4")
        || url_lower.ends_with(".webm")
        || url_lower.ends_with(".mp3")
        || url_lower.ends_with(".wav")
    {
        ResourceCategory::Media
    } else {
        ResourceCategory::Other
    }
}

fn truncate_url(url: &str) -> String {
    crate::util::truncate_url(url, 80)
}

fn generate_recommendations(
    breakdown: &ResourceBreakdown,
    total_bytes: u64,
    transfer_bytes: u64,
    carbon: &CarbonEstimate,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check total size
    if total_bytes > 5_000_000 {
        recommendations.push(format!(
            "Page size ({}) exceeds 5MB. Consider lazy loading and code splitting.",
            format_bytes(total_bytes)
        ));
    } else if total_bytes > 3_000_000 {
        recommendations.push(format!(
            "Page size ({}) is heavy. Consider optimizing resources.",
            format_bytes(total_bytes)
        ));
    }

    // Check compression
    if total_bytes > 0 {
        let ratio = transfer_bytes as f64 / total_bytes as f64;
        if ratio > 0.8 {
            recommendations.push("Enable gzip/brotli compression for text resources.".to_string());
        }
    }

    // Check JavaScript
    if breakdown.javascript.bytes > 1_000_000 {
        recommendations.push(format!(
            "JavaScript ({}) is large. Consider code splitting and tree shaking.",
            format_bytes(breakdown.javascript.bytes)
        ));
    }

    // Check images
    if breakdown.images.bytes > 2_000_000 {
        recommendations.push(format!(
            "Images ({}) are heavy. Use WebP format and responsive images.",
            format_bytes(breakdown.images.bytes)
        ));
    }

    // Check CSS
    if breakdown.css.bytes > 500_000 {
        recommendations.push(format!(
            "CSS ({}) is large. Remove unused styles and consider critical CSS.",
            format_bytes(breakdown.css.bytes)
        ));
    }

    // Check fonts
    if breakdown.fonts.count > 3 {
        recommendations.push(format!(
            "Using {} font files. Consider reducing font variations.",
            breakdown.fonts.count
        ));
    }

    // Check request count
    if breakdown.javascript.count + breakdown.css.count > 20 {
        recommendations
            .push("Many JS/CSS files. Consider bundling to reduce HTTP requests.".to_string());
    }

    if carbon.grams_co2e_per_view > 1.0 {
        recommendations.push(format!(
            "Estimated carbon footprint is {} g CO2e per view ({}). Reduce transfer bytes to improve sustainability.",
            carbon.format_grams(), carbon.rating
        ));
    }

    recommendations
}

pub fn estimate_carbon(transfer_bytes: u64) -> CarbonEstimate {
    // Conservative Sustainable Web Design-style transfer estimate:
    // 0.81 kWh / GB transfer × 442 gCO2e / kWh = 358.02 gCO2e / GB.
    const GRAMS_PER_GB: f64 = 0.81 * 442.0;
    let gb = transfer_bytes as f64 / 1_000_000_000.0;
    let grams = gb * GRAMS_PER_GB;
    CarbonEstimate {
        grams_co2e_per_view: grams,
        rating: carbon_rating(grams).to_string(),
        model: "transfer_bytes * 0.81 kWh/GB * 442 gCO2e/kWh".to_string(),
    }
}

fn carbon_rating(grams: f64) -> &'static str {
    if grams <= 0.095 {
        "A+"
    } else if grams <= 0.186 {
        "A"
    } else if grams <= 0.341 {
        "B"
    } else if grams <= 0.493 {
        "C"
    } else if grams <= 0.656 {
        "D"
    } else if grams <= 0.846 {
        "E"
    } else {
        "F"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
        assert_eq!(format_bytes(1_500_000), "1.5 MB");
    }

    #[test]
    fn test_categorize_resource() {
        assert!(matches!(
            categorize_resource("style.css", "css"),
            ResourceCategory::Css
        ));
        assert!(matches!(
            categorize_resource("app.js", "script"),
            ResourceCategory::JavaScript
        ));
        assert!(matches!(
            categorize_resource("photo.jpg", "img"),
            ResourceCategory::Image
        ));
    }

    #[test]
    fn test_content_weight_compression_ratio() {
        let weight = ContentWeight {
            total_bytes: 1000,
            transfer_bytes: 300,
            breakdown: ResourceBreakdown::default(),
            request_count: 5,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };

        assert!((weight.compression_ratio() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_estimate_carbon_from_transfer_bytes() {
        let estimate = estimate_carbon(1_000_000_000);

        assert!((estimate.grams_co2e_per_view - 358.02).abs() < 0.01);
        assert_eq!(estimate.rating, "F");
    }

    #[test]
    fn test_carbon_rating_for_light_page() {
        let estimate = estimate_carbon(100_000);

        assert_eq!(estimate.rating, "A+");
        assert!(estimate.grams_co2e_per_view < 0.1);
    }

    #[test]
    fn test_format_grams_discloses_near_zero_instead_of_truncating() {
        let near_zero = CarbonEstimate {
            grams_co2e_per_view: 0.003,
            rating: "A+".to_string(),
            model: String::new(),
        };
        assert_eq!(near_zero.format_grams(), "< 0.01");

        let normal = CarbonEstimate {
            grams_co2e_per_view: 0.42,
            rating: "A".to_string(),
            model: String::new(),
        };
        assert_eq!(normal.format_grams(), "0.42");

        let zero = CarbonEstimate {
            grams_co2e_per_view: 0.0,
            rating: "A+".to_string(),
            model: String::new(),
        };
        assert_eq!(zero.format_grams(), "0.00");
    }

    #[test]
    fn test_is_heavy() {
        let light = ContentWeight {
            total_bytes: 1_000_000,
            transfer_bytes: 500_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 10,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };
        assert!(!light.is_heavy());

        let heavy = ContentWeight {
            total_bytes: 5_000_000,
            transfer_bytes: 3_000_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 50,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };
        assert!(heavy.is_heavy());
    }
}
