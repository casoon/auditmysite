//! SEO analysis module
//!
//! Provides meta tags validation, heading structure, social tags, and technical SEO checks.

pub(crate) mod headings;
pub mod image_efficiency;
pub mod interpretation;
pub(crate) mod meta;
pub mod module;
pub mod page_health;
pub mod profile;
pub mod robots;
pub mod schema;
pub mod schema_fit;
pub mod schema_parity;
pub mod schema_rules;
pub mod serp;
pub(crate) mod social;
pub mod technical;
pub mod topics;

pub use module::SeoModule;

pub use headings::{analyze_heading_structure, HeadingInfo, HeadingIssue, HeadingStructure};
pub use image_efficiency::{analyze_image_efficiency, ImageEfficiencyAnalysis, OversizedImage};
pub use meta::{extract_meta_tags, MetaTags, MetaValidation};
pub use page_health::{
    analyze_page_health, collect_issues, HtmlValidationIssue, HtmlValidationKind,
    PageHealthAnalysis, PageHealthIssue, UrlCanonicalizationCheck, WwwConsolidation,
};
pub use profile::{build_content_profile, SeoContentProfile};
pub use robots::{audit_robots_txt, infer_robots_policy, BotClass, RobotsAudit, RobotsGroup};
pub use schema::{detect_structured_data, SchemaType, StructuredData};
pub use schema_fit::{
    assess_schema_fit, assess_schema_fit_with_facts, SchemaCoverageStatus, SchemaFitAssessment,
    SchemaFitStatus, SchemaPageKind,
};
pub use schema_parity::{ContentParityAssessment, ContentParityStatus, VisibleSchemaFacts};
pub use schema_rules::{
    SchemaFeature, SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment,
};
pub use serp::{build_serp_analysis, SerpAnalysis, SerpSignal, SerpSignalStatus};
pub use social::{extract_social_tags, OpenGraph, SocialTags, TwitterCard};
pub use technical::{analyze_technical_seo, collect_technical_issues, TechnicalSeo};
pub use topics::{
    average_page_semantic_score, derive_domain_topics, derive_topic_overlap_pairs,
    extract_page_topics,
};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

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
    /// Whether the page is meant to be indexed and is (#780).
    #[serde(default)]
    pub indexability: IndexabilityCheck,
}

/// Upper bound of the SEO score of a page that is meant to be indexed but
/// carries `noindex`: the lowest band. Nothing else on the page reaches
/// search users while the directive stands (#780).
pub const INDEXABILITY_BLOCKER_CAP: u32 = 39;

/// The page's indexability against what it is meant to have (#780). A
/// `noindex` without a declared or implied expectation stays undecided: an
/// intended noindex page is not an SEO defect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexabilityCheck {
    pub expectation: crate::audit::finding_contract::IndexabilityExpectation,
    /// The robots meta tag carries `noindex`.
    pub noindex: bool,
    /// Meant to be indexed, but `noindex`: a technical blocker that caps the
    /// score at [`INDEXABILITY_BLOCKER_CAP`].
    pub blocked: bool,
}

impl IndexabilityCheck {
    pub fn assess(
        technical: &TechnicalSeo,
        expectation: crate::audit::finding_contract::IndexabilityExpectation,
    ) -> Self {
        let noindex = technical
            .robots_meta
            .as_deref()
            .is_some_and(|robots| robots.to_lowercase().contains("noindex"));
        Self {
            expectation,
            noindex,
            blocked: noindex && expectation.expected == Some(true),
        }
    }
}

/// Run complete SEO analysis.
///
/// All derived text is baked in canonical English so the stored struct (and the
/// JSON report) is language-independent. The PDF re-derives localized text from
/// the same pure builders at presentation time (#406).
pub async fn analyze_seo(
    page: &Page,
    url: &str,
    log: &crate::audit::collection::CollectionLog,
) -> Result<SeoAnalysis> {
    // Extract all SEO data in parallel where possible
    let meta = extract_meta_tags(page, log).await?;
    let mut meta_issues = meta.validate();
    let headings = analyze_heading_structure(page, log).await?;
    let social = extract_social_tags(page, log).await?;
    meta_issues.extend(social.validate());
    let technical = analyze_technical_seo(page, url, log).await?;
    let structured_data = detect_structured_data(page, log).await?;

    // robots.txt — HTTP fetch, independent of browser
    let is_noindex = technical
        .robots_meta
        .as_deref()
        .map(|r| r.to_lowercase().contains("noindex"))
        .unwrap_or(false);
    let robots = audit_robots_txt(
        url,
        technical.canonical_url.as_deref(),
        is_noindex,
        "en",
        log,
    )
    .await;

    // Page health analysis — HTTP probes + DOM inspection
    let page_health = analyze_page_health(page, url, log).await.map(|mut ph| {
        // Copy in the declared <meta charset> so charset-consistency
        // checking (#543) stays a pure function of PageHealthAnalysis
        // without page_health.rs depending on the meta module.
        ph.declared_charset = meta.charset.clone();
        ph.issues = collect_issues(&ph, true);
        ph
    });

    // Image efficiency analysis
    let image_efficiency = analyze_image_efficiency(page, log).await;

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
        indexability: IndexabilityCheck::default(),
    };

    // Build content profile from collected data (baked English; PDF re-derives)
    analysis.content_profile = Some(build_content_profile(&analysis, "en"));

    // SERP pass — pure aggregation, no additional CDP calls needed
    analysis.serp = Some(build_serp_analysis(&analysis, url, "en"));

    Ok(analysis)
}

/// Recomputes the SEO score after the heading issues were replaced by the
/// shared findings (#724) and the page's indexability expectation is known
/// (#780).
pub(crate) fn rescore(
    seo: &mut SeoAnalysis,
    expectation: crate::audit::finding_contract::IndexabilityExpectation,
) {
    seo.indexability = IndexabilityCheck::assess(&seo.technical, expectation);
    let score = calculate_seo_score(
        &seo.meta,
        &seo.meta_issues,
        &seo.headings,
        &seo.social,
        &seo.technical,
        seo.image_efficiency.as_ref(),
    );
    seo.score = if seo.indexability.blocked {
        score.min(INDEXABILITY_BLOCKER_CAP)
    } else {
        score
    };
}

fn calculate_seo_score(
    meta: &MetaTags,
    meta_issues: &[MetaValidation],
    headings: &HeadingStructure,
    social: &SocialTags,
    technical: &TechnicalSeo,
    image_efficiency: Option<&ImageEfficiencyAnalysis>,
) -> u32 {
    let mut score = 100u32;

    // Meta tags — cap total penalty at 20 to avoid collapse from many minor issues.
    // A missing lang or canonical is charged once, in the technical section
    // below; the meta list reported it a second time. A missing viewport is
    // charged by Mobile and Accessibility, not here as well (plan 105).
    let meta_penalty: u32 = meta_issues
        .iter()
        .filter(|issue| !matches!(issue.field.as_str(), "lang" | "canonical" | "viewport"))
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

    // Baseline-signal floor (#486): a page with the core meta/technical SEO
    // signals in place must not collapse from a couple of structural heading
    // issues. Each present baseline signal raises the floor by 10 points, so
    // well-configured pages stay in a credible range even with a missing H1 —
    // a high-profile site with complete meta tags, canonical, lang and Open
    // Graph no longer scores 44 from two heading findings.
    let baseline_signals = [
        meta.title.as_ref().is_some_and(|t| !t.trim().is_empty()),
        meta.description
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty()),
        technical.https,
        technical.has_canonical,
        technical.has_lang,
        social.open_graph.is_some(),
    ];
    let floor = baseline_signals.iter().filter(|present| **present).count() as u32 * 10;

    score.max(floor).min(100)
}

#[cfg(test)]
mod score_tests {
    use super::*;

    fn issue(field: &str, severity: crate::taxonomy::Severity) -> MetaValidation {
        MetaValidation {
            field: field.to_string(),
            message: String::new(),
            severity,
            suggestion: None,
        }
    }

    /// Plan 105: a missing lang or canonical showed up in the meta list and
    /// in the technical checks, and cost points in both. It counts once.
    #[test]
    fn missing_lang_and_canonical_are_charged_once() {
        use crate::taxonomy::Severity;
        let technical = TechnicalSeo {
            https: true,
            has_canonical: false,
            has_lang: false,
            ..Default::default()
        };
        let social = SocialTags {
            open_graph: Some(Default::default()),
            twitter_card: Some(Default::default()),
            ..Default::default()
        };
        let headings = HeadingStructure {
            h1_count: 1,
            ..Default::default()
        };
        let meta = MetaTags::default();
        let without_meta_issues =
            calculate_seo_score(&meta, &[], &headings, &social, &technical, None);
        let with_meta_issues = calculate_seo_score(
            &meta,
            &[
                issue("lang", Severity::Medium),
                issue("canonical", Severity::Low),
            ],
            &headings,
            &social,
            &technical,
            None,
        );
        assert_eq!(with_meta_issues, without_meta_issues);
        assert_eq!(without_meta_issues, 100 - 5 - 3);
    }

    /// Plan 105: a missing viewport is charged by Mobile and Accessibility;
    /// SEO lists it without a deduction.
    #[test]
    fn missing_viewport_costs_seo_nothing() {
        let technical = TechnicalSeo {
            https: true,
            has_canonical: true,
            has_lang: true,
            ..Default::default()
        };
        let social = SocialTags {
            open_graph: Some(Default::default()),
            twitter_card: Some(Default::default()),
            ..Default::default()
        };
        let headings = HeadingStructure {
            h1_count: 1,
            ..Default::default()
        };
        let meta = MetaTags::default();
        assert_eq!(
            calculate_seo_score(
                &meta,
                &[issue("viewport", crate::taxonomy::Severity::High)],
                &headings,
                &social,
                &technical,
                None,
            ),
            100
        );
    }
}
