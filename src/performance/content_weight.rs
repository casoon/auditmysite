//! Content weight analyzer
//!
//! Analyzes page resources by type and provides optimization recommendations.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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
    /// Optimization recommendations
    pub recommendations: Vec<String>,
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

    // Get resource timing entries via JavaScript
    let js_code = r#"
    (() => {
        const resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(r => ({
            name: r.name,
            type: r.initiatorType,
            transferSize: r.transferSize || 0,
            decodedSize: r.decodedBodySize || 0,
            duration: r.duration
        })));
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Resource analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("[]");

    let resources: Vec<ResourceEntry> = serde_json::from_str(json_str).unwrap_or_else(|e| {
        warn!("Failed to parse resource entries JSON: {}", e);
        Vec::new()
    });

    let mut breakdown = ResourceBreakdown::default();
    let mut total_bytes: u64 = 0;
    let mut transfer_bytes: u64 = 0;
    let mut request_count: u32 = 0;

    for resource in &resources {
        request_count += 1;
        total_bytes += resource.decoded_size;
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
        stats.bytes += resource.decoded_size;
        stats.transfer_bytes += resource.transfer_size;

        if resource.decoded_size > stats.largest_bytes {
            stats.largest_bytes = resource.decoded_size;
            stats.largest_url = Some(truncate_url(&resource.name));
        }
    }

    // Generate recommendations
    let recommendations = generate_recommendations(&breakdown, total_bytes, transfer_bytes);

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

    // Check by extension first
    if url_lower.ends_with(".css") || initiator_type == "css" {
        ResourceCategory::Css
    } else if url_lower.ends_with(".js") || initiator_type == "script" {
        ResourceCategory::JavaScript
    } else if url_lower.ends_with(".html") || url_lower.ends_with(".htm") {
        ResourceCategory::Html
    } else if url_lower.contains(".woff")
        || url_lower.contains(".ttf")
        || url_lower.contains(".otf")
        || url_lower.contains(".eot")
    {
        ResourceCategory::Font
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
    if url.len() > 80 {
        format!("{}...", &url[..77])
    } else {
        url.to_string()
    }
}

fn generate_recommendations(
    breakdown: &ResourceBreakdown,
    total_bytes: u64,
    transfer_bytes: u64,
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

    recommendations
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
            recommendations: vec![],
        };

        assert!((weight.compression_ratio() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_is_heavy() {
        let light = ContentWeight {
            total_bytes: 1_000_000,
            transfer_bytes: 500_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 10,
            recommendations: vec![],
        };
        assert!(!light.is_heavy());

        let heavy = ContentWeight {
            total_bytes: 5_000_000,
            transfer_bytes: 3_000_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 50,
            recommendations: vec![],
        };
        assert!(heavy.is_heavy());
    }
}
