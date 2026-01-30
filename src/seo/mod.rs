//! SEO analysis module
//!
//! Provides meta tags validation, heading structure, social tags, and technical SEO checks.

mod meta;
mod headings;
mod social;
mod technical;
mod schema;

pub use meta::{extract_meta_tags, MetaTags, MetaValidation};
pub use headings::{analyze_heading_structure, HeadingStructure, HeadingIssue};
pub use social::{extract_social_tags, SocialTags, OpenGraph, TwitterCard};
pub use technical::{analyze_technical_seo, TechnicalSeo};
pub use schema::{detect_structured_data, StructuredData, SchemaType};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Complete SEO analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Run complete SEO analysis
pub async fn analyze_seo(page: &Page, url: &str) -> Result<SeoAnalysis> {
    // Extract all SEO data in parallel where possible
    let meta = extract_meta_tags(page).await?;
    let meta_issues = meta.validate();
    let headings = analyze_heading_structure(page).await?;
    let social = extract_social_tags(page).await?;
    let technical = analyze_technical_seo(page, url).await?;
    let structured_data = detect_structured_data(page).await?;

    // Calculate score
    let score = calculate_seo_score(&meta, &meta_issues, &headings, &social, &technical);

    Ok(SeoAnalysis {
        meta,
        meta_issues,
        headings,
        social,
        technical,
        structured_data,
        score,
    })
}

fn calculate_seo_score(
    meta: &MetaTags,
    meta_issues: &[MetaValidation],
    headings: &HeadingStructure,
    social: &SocialTags,
    technical: &TechnicalSeo,
) -> u32 {
    let mut score = 100u32;

    // Meta tags (-5 to -20 per issue)
    for issue in meta_issues {
        score = score.saturating_sub(match issue.severity.as_str() {
            "error" => 10,
            "warning" => 5,
            _ => 2,
        });
    }

    // Heading structure
    if headings.h1_count == 0 {
        score = score.saturating_sub(15);
    } else if headings.h1_count > 1 {
        score = score.saturating_sub(5);
    }
    if !headings.issues.is_empty() {
        score = score.saturating_sub(headings.issues.len() as u32 * 3);
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

    score.min(100)
}
