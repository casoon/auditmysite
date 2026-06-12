//! SEO analysis module
//!
//! Provides meta tags validation, heading structure, social tags, and technical SEO checks.

mod headings;
pub mod image_efficiency;
mod meta;
pub mod module;
pub mod page_health;
pub mod profile;
pub mod robots;
pub mod schema;
pub mod serp;
mod social;
pub mod technical;

pub use module::SeoModule;

pub use headings::{analyze_heading_structure, HeadingIssue, HeadingStructure};
pub use image_efficiency::{analyze_image_efficiency, ImageEfficiencyAnalysis, OversizedImage};
pub use meta::{extract_meta_tags, MetaTags, MetaValidation};
pub use page_health::{
    analyze_page_health, HtmlValidationIssue, PageHealthAnalysis, PageHealthIssue, WwwConsolidation,
};
pub use profile::{build_content_profile, SeoContentProfile};
pub use robots::{audit_robots_txt, BotClass, RobotsAudit, RobotsGroup};
pub use schema::{detect_structured_data, SchemaType, StructuredData};
pub use serp::{build_serp_analysis, SerpAnalysis, SerpSignal, SerpSignalStatus};
pub use social::{extract_social_tags, OpenGraph, SocialTags, TwitterCard};
pub use technical::{analyze_technical_seo, TechnicalSeo};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::Result;

/// Complete SEO analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoAnalysis {
    /// Meta tags analysis
    pub meta: MetaTags,
    /// Meta validation issues
    pub meta_issues: Vec<MetaValidation>,
    /// Heading structure
    pub headings: HeadingStructure,
    /// Social media tags
    pub social: SocialTags,
    /// Technical SEO
    pub technical: TechnicalSeo,
    /// Structured data
    pub structured_data: StructuredData,
    /// Overall SEO score (0-100)
    pub score: u32,
    /// Content profile analysis
    pub content_profile: Option<SeoContentProfile>,
    /// robots.txt audit (informational, no score impact)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robots: Option<RobotsAudit>,
    /// Page health analysis (HTTP probes, DOM checks, URL analysis)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_health: Option<PageHealthAnalysis>,
    /// SERP pass — aggregated search-result-page readiness signals
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serp: Option<SerpAnalysis>,
    /// Image efficiency analysis (format + oversizing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_efficiency: Option<ImageEfficiencyAnalysis>,
}

/// Run complete SEO analysis
pub async fn analyze_seo(page: &Page, url: &str, locale: &str) -> Result<SeoAnalysis> {
    // Extract all SEO data in parallel where possible
    let meta = extract_meta_tags(page).await?;
    let meta_issues = meta.validate();
    let headings = analyze_heading_structure(page).await?;
    let social = extract_social_tags(page).await?;
    let technical = analyze_technical_seo(page, url, locale).await?;
    let structured_data = detect_structured_data(page).await?;

    // robots.txt — HTTP fetch, independent of browser
    let is_noindex = technical
        .robots_meta
        .as_deref()
        .map(|r| r.to_lowercase().contains("noindex"))
        .unwrap_or(false);
    let robots =
        Some(audit_robots_txt(url, technical.canonical_url.as_deref(), is_noindex, locale).await);

    // Page health analysis — HTTP probes + DOM inspection
    let page_health = match analyze_page_health(page, url, locale).await {
        Ok(ph) => Some(ph),
        Err(e) => {
            warn!("Page health analysis failed: {}", e);
            None
        }
    };

    // Image efficiency analysis
    let image_efficiency = match analyze_image_efficiency(page).await {
        Ok(ie) => Some(ie),
        Err(e) => {
            warn!("Image efficiency analysis failed: {}", e);
            None
        }
    };

    // Calculate score
    let score = calculate_seo_score(
        &meta,
        &meta_issues,
        &headings,
        &social,
        &technical,
        image_efficiency.as_ref(),
    );

    let mut analysis = SeoAnalysis {
        meta,
        meta_issues,
        headings,
        social,
        technical,
        structured_data,
        score,
        content_profile: None,
        robots,
        page_health,
        serp: None,
        image_efficiency,
    };

    // Build content profile from collected data
    analysis.content_profile = Some(build_content_profile(&analysis, locale));

    // SERP pass — pure aggregation, no additional CDP calls needed
    analysis.serp = Some(build_serp_analysis(&analysis, url, locale));

    Ok(analysis)
}

fn calculate_seo_score(
    _meta: &MetaTags,
    meta_issues: &[MetaValidation],
    headings: &HeadingStructure,
    social: &SocialTags,
    technical: &TechnicalSeo,
    image_efficiency: Option<&ImageEfficiencyAnalysis>,
) -> u32 {
    let mut score = 100u32;

    // Meta tags — cap total penalty at 20 to avoid collapse from many minor issues
    let meta_penalty: u32 = meta_issues
        .iter()
        .map(|issue| match issue.severity {
            crate::taxonomy::Severity::Critical => 15,
            crate::taxonomy::Severity::High => 10,
            crate::taxonomy::Severity::Medium => 5,
            crate::taxonomy::Severity::Low => 2,
        })
        .sum::<u32>()
        .min(20);
    score = score.saturating_sub(meta_penalty);

    // Heading structure
    if headings.h1_count == 0 {
        score = score.saturating_sub(15);
    } else if headings.h1_count > 1 {
        score = score.saturating_sub(5);
    }
    if !headings.issues.is_empty() {
        // Cap at 15 — reflects severity, not the count of repeated instances
        let heading_penalty = (headings.issues.len() as u32 * 3).min(15);
        score = score.saturating_sub(heading_penalty);
    }

    // Social tags
    if social.open_graph.is_none() {
        score = score.saturating_sub(5);
    }
    if social.twitter_card.is_none() {
        score = score.saturating_sub(5);
    }

    // Technical SEO
    if !technical.https {
        score = score.saturating_sub(10);
    }
    if !technical.has_canonical {
        score = score.saturating_sub(5);
    }
    if !technical.has_lang {
        score = score.saturating_sub(3);
    }

    // Image efficiency penalty
    if let Some(ie) = image_efficiency {
        let ie_penalty = (100 - ie.score).min(15);
        score = score.saturating_sub(ie_penalty);
    }

    score.min(100)
}
