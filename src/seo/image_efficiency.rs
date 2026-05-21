//! Image efficiency analysis for SEO (#131).
//!
//! Checks whether images use modern formats (WebP/AVIF) and are not
//! served at a resolution significantly larger than their display size.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// An image that is served at a higher resolution than it is displayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversizedImage {
    /// Image src URL (truncated to 120 chars)
    pub src: String,
    /// Natural (intrinsic) width in pixels
    pub natural_width: u32,
    /// Natural (intrinsic) height in pixels
    pub natural_height: u32,
    /// Rendered display width in CSS pixels
    pub display_width: u32,
    /// Rendered display height in CSS pixels
    pub display_height: u32,
}

/// Image efficiency analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEfficiencyAnalysis {
    /// Total number of `<img>` elements found on the page
    pub total_images: usize,
    /// Images using modern formats (WebP, AVIF, SVG)
    pub modern_format_count: usize,
    /// Images using legacy formats (JPEG, PNG, GIF, BMP, TIFF)
    pub legacy_format_count: usize,
    /// Percentage of images using modern formats (0–100)
    pub modern_format_pct: f64,
    /// Images with a natural resolution more than 2× larger than display size
    pub oversized_images: Vec<OversizedImage>,
    /// URLs of images using legacy formats (max 20)
    pub legacy_format_urls: Vec<String>,
    /// Score contribution to SEO (0–100)
    pub score: u32,
}

/// Analyze image efficiency on the page.
pub async fn analyze_image_efficiency(page: &Page) -> Result<ImageEfficiencyAnalysis> {
    info!("Analyzing image efficiency...");

    let js = r#"
    (() => {
        const imgs = Array.from(document.querySelectorAll('img'));
        return JSON.stringify(imgs.map(img => ({
            src: (img.currentSrc || img.src || '').substring(0, 120),
            naturalWidth: img.naturalWidth || 0,
            naturalHeight: img.naturalHeight || 0,
            displayWidth: img.width || img.clientWidth || 0,
            displayHeight: img.height || img.clientHeight || 0,
        })));
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Image efficiency JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");

    #[derive(serde::Deserialize)]
    struct RawImg {
        src: String,
        #[serde(rename = "naturalWidth")]
        natural_width: u32,
        #[serde(rename = "naturalHeight")]
        natural_height: u32,
        #[serde(rename = "displayWidth")]
        display_width: u32,
        #[serde(rename = "displayHeight")]
        display_height: u32,
    }

    let raw: Vec<RawImg> = serde_json::from_str(json_str).unwrap_or_default();

    let total_images = raw.len();
    let mut modern_format_count = 0usize;
    let mut legacy_format_count = 0usize;
    let mut oversized_images: Vec<OversizedImage> = Vec::new();
    let mut legacy_format_urls: Vec<String> = Vec::new();

    for img in &raw {
        if img.src.is_empty() || img.src.starts_with("data:") || img.src.starts_with("blob:") {
            continue;
        }

        let format = detect_format(&img.src);
        match format {
            ImageFormat::Modern => modern_format_count += 1,
            ImageFormat::Legacy => {
                legacy_format_count += 1;
                if legacy_format_urls.len() < 20 {
                    legacy_format_urls.push(img.src.clone());
                }
            }
            ImageFormat::Unknown => {}
        }

        // Flag images served more than 2× larger than displayed
        if img.display_width > 0
            && img.display_height > 0
            && img.natural_width > 0
            && img.natural_height > 0
        {
            let width_ratio = img.natural_width as f64 / img.display_width as f64;
            let height_ratio = img.natural_height as f64 / img.display_height as f64;
            if width_ratio > 2.0 || height_ratio > 2.0 {
                oversized_images.push(OversizedImage {
                    src: img.src.clone(),
                    natural_width: img.natural_width,
                    natural_height: img.natural_height,
                    display_width: img.display_width,
                    display_height: img.display_height,
                });
            }
        }
    }

    // Sort oversized by worst ratio first (largest natural vs display gap)
    oversized_images.sort_by(|a, b| {
        let ratio_a = if a.display_width > 0 {
            a.natural_width as f64 / a.display_width as f64
        } else {
            0.0
        };
        let ratio_b = if b.display_width > 0 {
            b.natural_width as f64 / b.display_width as f64
        } else {
            0.0
        };
        ratio_b
            .partial_cmp(&ratio_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let categorized = modern_format_count + legacy_format_count;
    let modern_format_pct = if categorized > 0 {
        (modern_format_count as f64 / categorized as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        100.0 // no images = no problem
    };

    let score = calculate_score(modern_format_pct, oversized_images.len(), total_images);

    info!(
        "Image efficiency: {}/{} modern format ({:.0}%), {} oversized",
        modern_format_count,
        categorized,
        modern_format_pct,
        oversized_images.len()
    );

    Ok(ImageEfficiencyAnalysis {
        total_images,
        modern_format_count,
        legacy_format_count,
        modern_format_pct,
        oversized_images,
        legacy_format_urls,
        score,
    })
}

enum ImageFormat {
    Modern,  // webp, avif, svg
    Legacy,  // jpeg, jpg, png, gif, bmp, tiff
    Unknown, // can't determine from URL (e.g. query-string only)
}

fn detect_format(src: &str) -> ImageFormat {
    let lower = src.to_lowercase();
    // Strip query string for extension detection
    let path = lower.split('?').next().unwrap_or(&lower);
    if path.ends_with(".webp") || path.ends_with(".avif") || path.ends_with(".svg") {
        ImageFormat::Modern
    } else if path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".png")
        || path.ends_with(".gif")
        || path.ends_with(".bmp")
        || path.ends_with(".tiff")
        || path.ends_with(".tif")
    {
        ImageFormat::Legacy
    } else {
        ImageFormat::Unknown
    }
}

fn calculate_score(modern_pct: f64, oversized_count: usize, total: usize) -> u32 {
    if total == 0 {
        return 100;
    }
    let mut score = 100u32;

    // Penalize for low modern format adoption
    if modern_pct < 50.0 {
        score = score.saturating_sub(20);
    } else if modern_pct < 80.0 {
        score = score.saturating_sub(10);
    } else if modern_pct < 95.0 {
        score = score.saturating_sub(5);
    }

    // Penalize for oversized images (cap at 25)
    let oversized_pct = oversized_count as f64 / total as f64;
    let oversized_penalty = if oversized_pct > 0.5 {
        25
    } else if oversized_pct > 0.25 {
        15
    } else if oversized_count > 0 {
        8
    } else {
        0
    };
    score = score.saturating_sub(oversized_penalty);

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_modern() {
        assert!(matches!(
            detect_format("https://cdn.example.com/hero.webp"),
            ImageFormat::Modern
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/photo.avif"),
            ImageFormat::Modern
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/logo.svg"),
            ImageFormat::Modern
        ));
    }

    #[test]
    fn test_detect_format_legacy() {
        assert!(matches!(
            detect_format("https://cdn.example.com/photo.jpg"),
            ImageFormat::Legacy
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/photo.jpeg"),
            ImageFormat::Legacy
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/image.png"),
            ImageFormat::Legacy
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/anim.gif"),
            ImageFormat::Legacy
        ));
    }

    #[test]
    fn test_detect_format_with_query() {
        assert!(matches!(
            detect_format("https://cdn.example.com/photo.webp?v=2"),
            ImageFormat::Modern
        ));
        assert!(matches!(
            detect_format("https://cdn.example.com/photo.png?w=800"),
            ImageFormat::Legacy
        ));
    }

    #[test]
    fn test_calculate_score_no_images() {
        assert_eq!(calculate_score(100.0, 0, 0), 100);
    }

    #[test]
    fn test_calculate_score_all_modern_no_oversized() {
        assert_eq!(calculate_score(100.0, 0, 10), 100);
    }

    #[test]
    fn test_calculate_score_all_legacy() {
        let score = calculate_score(0.0, 0, 10);
        assert!(score <= 80);
    }
}
